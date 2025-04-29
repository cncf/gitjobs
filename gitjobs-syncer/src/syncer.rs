//! This module defines some types and the logic to synchronize the members and
//! projects of the foundations with the `GitJobs` database.

use std::time::Duration;

use anyhow::{Context, Error, Result, format_err};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::time::timeout;
use tracing::{info, instrument};

use crate::db::DynDB;

/// Maximum time that can take synchronizing a foundation.
const FOUNDATION_TIMEOUT: u64 = 300;

/// A syncer is responsible for synchronizing the members and projects of all
/// registered foundations. It feeds from the landscape API.
pub(crate) struct Syncer {
    db: DynDB,
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
                        Err(final_err) => Err(format_err!("{:#}\n{:#}", final_err, task_err)),
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
        let members_in_landscape: Vec<LandscapeMember> =
            self.http_client.get(&url).send().await?.json().await?;

        // Get members from database
        let members_in_db = self.db.list_members(&foundation.name).await?;

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
        let projects_in_landscape: Vec<LandscapeProject> =
            self.http_client.get(&url).send().await?.json().await?;

        // Get projects from database
        let projects_in_db = self.db.list_projects(&foundation.name).await?;

        Ok(())
    }
}

// Types.

/// Foundation details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Foundation {
    pub name: String,
    pub landscape_url: String,
}

/// Landscape member details.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LandscapeMember {
    name: String,
    level: String,
    logo_url: String,
}

/// Landscape project details.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct LandscapeProject {
    name: String,
    logo_url: String,
    maturity: String,
}

/// Member details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Member {
    pub foundation: String,
    pub name: String,
    pub level: String,
    pub logo_url: String,
}

/// Project details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Project {
    pub foundation: String,
    pub name: String,
    pub maturity: String,
    pub logo_url: String,
}
