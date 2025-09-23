//! This module defines types and logic to synchronize foundation members and projects
//! with the `GitJobs` database.

use std::{sync::LazyLock, time::Duration};

use anyhow::{Context, Error, Result, format_err};
use futures::stream::{self, StreamExt};
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{debug, info, instrument};

use crate::db::DynDB;

/// Maximum time, in seconds, allowed for synchronizing a foundation.
const FOUNDATION_TIMEOUT: u64 = 300;

/// Responsible for synchronizing members and projects of all registered foundations.
/// Feeds from the landscape API and updates the `GitJobs` database accordingly.
pub(crate) struct Syncer {
    /// Database handle for storing and retrieving foundation data.
    db: DynDB,
    /// HTTP client used to fetch data from the landscape API.
    http_client: reqwest::Client,
}

impl Syncer {
    /// Create a new `Syncer` instance.
    pub(crate) fn new(db: DynDB) -> Self {
        Self {
            db,
            http_client: reqwest::Client::new(),
        }
    }

    /// Run the syncer to synchronize all registered foundations.
    #[instrument(skip_all, err)]
    pub(crate) async fn run(&self) -> anyhow::Result<()> {
        info!("started");

        let foundations = self.db.list_foundations().await?;
        #[allow(clippy::manual_try_fold)]
        let result = stream::iter(foundations)
            .map(|foundation| async {
                let foundation_name = foundation.name.clone();
                match timeout(
                    Duration::from_secs(FOUNDATION_TIMEOUT),
                    self.sync_foundation(foundation),
                )
                .await
                {
                    Ok(result) => result,
                    Err(err) => Err(err.into()),
                }
                .context(format!("error synchronizing foundation {foundation_name}"))
            })
            .buffer_unordered(3)
            .collect::<Vec<Result<()>>>()
            .await
            .into_iter()
            .fold(
                Ok::<(), Error>(()),
                |final_result, task_result| match task_result {
                    Ok(()) => final_result,
                    Err(task_err) => match final_result {
                        Ok(()) => Err(task_err),
                        Err(final_err) => Err(format_err!("{final_err:#}\n{task_err:#}")),
                    },
                },
            );

        info!("finished");
        result
    }

    /// Synchronize the members and projects of the provided foundation.
    #[instrument(fields(foundation = foundation.name), skip_all, err)]
    async fn sync_foundation(&self, foundation: Foundation) -> Result<()> {
        info!("started");

        self.sync_members(foundation.clone()).await?;
        self.sync_projects(foundation).await?;

        info!("finished");
        Ok(())
    }

    /// Synchronize the members of the provided foundation.
    #[instrument(fields(foundation = foundation.name), skip_all, err)]
    async fn sync_members(&self, foundation: Foundation) -> Result<()> {
        // Get members from landscape
        let url = format!(
            "{}/api/members/all.json",
            foundation
                .landscape_url
                .strip_suffix('/')
                .unwrap_or(&foundation.landscape_url)
        );
        let mut members_in_landscape: Vec<LandscapeMember> = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("error fetching landscape members")?
            .json()
            .await?;
        for landscape_member in &mut members_in_landscape {
            // Remove the member kind from the name
            landscape_member.name = MEMBER_KIND.replace(&landscape_member.name, "").to_string();
        }

        // Get members from database
        let members_in_db = self.db.list_members(&foundation.name).await?;

        // Add new members (members in landscape but not in db)
        let members_added: Vec<Member> = members_in_landscape
            .iter()
            .filter(|landscape_member| {
                !members_in_db
                    .iter()
                    .any(|db_member| db_member.name == landscape_member.name)
                    && !landscape_member.name.to_lowercase().contains("non-public")
            })
            .map(|landscape_member| Member {
                foundation: foundation.name.clone(),
                name: landscape_member.name.clone(),
                level: landscape_member.subcategory.clone(),
                logo_url: landscape_member.logo_url.clone(),
            })
            .collect();
        for member in members_added {
            debug!(name = member.name, "adding member");
            self.db.add_member(&member).await?;
        }

        // Remove non-existing members (members in db but not in landscape)
        let members_removed: Vec<&String> = members_in_db
            .iter()
            .filter(|db_member| {
                !members_in_landscape
                    .iter()
                    .any(|landscape_member| landscape_member.name == db_member.name)
            })
            .map(|db_member| &db_member.name)
            .collect();
        for member_name in members_removed {
            debug!(name = member_name, "removing member");
            self.db.remove_member(&foundation.name, member_name).await?;
        }

        // Update existing members (members in both landscape and db)
        let members_updated: Vec<Member> = members_in_landscape
            .iter()
            .filter(|landscape_member| {
                members_in_db.iter().any(|db_member| {
                    db_member.name == landscape_member.name
                        && (db_member.level != landscape_member.subcategory
                            || db_member.logo_url != landscape_member.logo_url)
                })
            })
            .map(|landscape_member| Member {
                foundation: foundation.name.clone(),
                name: landscape_member.name.clone(),
                level: landscape_member.subcategory.clone(),
                logo_url: landscape_member.logo_url.clone(),
            })
            .collect();
        for member in members_updated {
            debug!(name = member.name, "updating member");
            self.db.update_member(&member).await?;
        }

        Ok(())
    }

    /// Synchronize the projects of the provided foundation.
    #[instrument(fields(foundation = foundation.name), skip_all, err)]
    async fn sync_projects(&self, foundation: Foundation) -> Result<()> {
        // Get projects from landscape
        let url = format!(
            "{}/api/projects/all.json",
            foundation
                .landscape_url
                .strip_suffix('/')
                .unwrap_or(&foundation.landscape_url)
        );
        let projects_in_landscape: Vec<LandscapeProject> = self
            .http_client
            .get(&url)
            .send()
            .await
            .context("error fetching landscape projects")?
            .json()
            .await?;

        // Get projects from database
        let projects_in_db = self.db.list_projects(&foundation.name).await?;

        // Add new projects (projects in landscape but not in db)
        let projects_added: Vec<Project> = projects_in_landscape
            .iter()
            .filter(|landscape_project| {
                !projects_in_db
                    .iter()
                    .any(|db_project| db_project.name == landscape_project.name)
                    && landscape_project.maturity != "archived"
            })
            .map(|landscape_project| Project {
                foundation: foundation.name.clone(),
                name: landscape_project.name.clone(),
                maturity: landscape_project.maturity.clone(),
                logo_url: landscape_project.logo_url.clone(),
            })
            .collect();
        for project in projects_added {
            debug!(name = project.name, "adding project");
            self.db.add_project(&project).await?;
        }

        // Remove non-existing projects (projects in db but not in landscape)
        let projects_removed: Vec<&String> = projects_in_db
            .iter()
            .filter(|db_project| {
                !projects_in_landscape
                    .iter()
                    .any(|landscape_project| landscape_project.name == db_project.name)
            })
            .map(|db_project| &db_project.name)
            .collect();
        for project_name in projects_removed {
            debug!(name = project_name, "removing project");
            self.db.remove_project(&foundation.name, project_name).await?;
        }

        // Update existing projects (projects in both landscape and db)
        let projects_updated: Vec<Project> = projects_in_landscape
            .iter()
            .filter(|landscape_project| {
                projects_in_db.iter().any(|db_project| {
                    db_project.name == landscape_project.name
                        && (db_project.maturity != landscape_project.maturity
                            || db_project.logo_url != landscape_project.logo_url)
                })
            })
            .map(|landscape_project| Project {
                foundation: foundation.name.clone(),
                name: landscape_project.name.clone(),
                maturity: landscape_project.maturity.clone(),
                logo_url: landscape_project.logo_url.clone(),
            })
            .collect();
        for project in projects_updated {
            debug!(name = project.name, "updating project");
            self.db.update_project(&project).await?;
        }

        Ok(())
    }
}

/// Regular expression that matches the member kind in the member name, e.g. " (Platinum)".
static MEMBER_KIND: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r" \(.*\)").expect("exprs in MEMBER_KIND should be valid"));

// Types.

/// Foundation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Foundation {
    /// Name of the foundation.
    pub name: String,
    /// Base URL of the foundation's landscape API.
    pub landscape_url: String,
}

/// Details of a member as returned by the landscape API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LandscapeMember {
    /// Name of the member, possibly including kind (e.g. " (Platinum)").
    name: String,
    /// Subcategory or level of the member (e.g. "Platinum", "Gold").
    subcategory: String,
    /// URL to the member's logo image.
    logo_url: String,
}

/// Details of a project as returned by the landscape API.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LandscapeProject {
    /// Name of the project.
    name: String,
    /// URL to the project's logo image.
    logo_url: String,
    /// Project maturity level (e.g. "sandbox", "incubating", "graduated", "archived").
    maturity: String,
}

/// Member details as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Member {
    /// Name of the foundation this member belongs to.
    pub foundation: String,
    /// Name of the member (without kind suffix).
    pub name: String,
    /// Level or subcategory of the member (e.g. "Platinum").
    pub level: String,
    /// URL to the member's logo image.
    pub logo_url: String,
}

/// Project details as stored in the database.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    /// Name of the foundation this project belongs to.
    pub foundation: String,
    /// Name of the project.
    pub name: String,
    /// Project maturity level (e.g. "sandbox", "incubating", "graduated", "archived").
    pub maturity: String,
    /// URL to the project's logo image.
    pub logo_url: String,
}
