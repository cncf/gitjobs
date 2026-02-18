//! This module defines an abstraction layer over the database.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::{instrument, trace};

use crate::syncer::{Foundation, Member, Project};

/// Abstraction layer over the database. Trait that defines operations a `DB` must support.
#[async_trait]
pub(crate) trait DB {
    /// Adds a new member to a foundation.
    async fn add_member(&self, member: &Member) -> Result<()>;

    /// Adds a new project to a foundation.
    async fn add_project(&self, project: &Project) -> Result<()>;

    /// Lists all foundations present in the database.
    async fn list_foundations(&self) -> Result<Vec<Foundation>>;

    /// Lists all members of a given foundation.
    async fn list_members(&self, foundation: &str) -> Result<Vec<Member>>;

    /// Lists all projects of a given foundation.
    async fn list_projects(&self, foundation: &str) -> Result<Vec<Project>>;

    /// Removes a member from a foundation.
    async fn remove_member(&self, foundation: &str, member_name: &str) -> Result<()>;

    /// Removes a project from a foundation.
    async fn remove_project(&self, foundation: &str, project_name: &str) -> Result<()>;

    /// Updates an existing member's information.
    async fn update_member(&self, member: &Member) -> Result<()>;

    /// Updates an existing project's information.
    async fn update_project(&self, project: &Project) -> Result<()>;
}

/// Type alias for a thread-safe, reference-counted `DB` trait object.
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

/// DB implementation backed by `PostgreSQL`.
pub(crate) struct PgDB {
    /// Connection pool for `PostgreSQL` database access.
    pool: Pool,
}

impl PgDB {
    /// Creates a new `PgDB` instance.
    pub(crate) fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DB for PgDB {
    #[instrument(skip(self), err)]
    async fn add_member(&self, member: &Member) -> Result<()> {
        trace!("db: add member");

        let db = self.pool.get().await?;
        db.execute(
            "select add_member($1::text, $2::text, $3::text, $4::text)",
            &[&member.foundation, &member.name, &member.level, &member.logo_url],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn add_project(&self, project: &Project) -> Result<()> {
        trace!("db: add project");

        let db = self.pool.get().await?;
        db.execute(
            "select add_project($1::text, $2::text, $3::text, $4::text)",
            &[
                &project.foundation,
                &project.name,
                &project.maturity,
                &project.logo_url,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn list_foundations(&self) -> Result<Vec<Foundation>> {
        trace!("db: list foundations");

        let db = self.pool.get().await?;
        let row = db.query_one("select syncer_list_foundations()::text", &[]).await?;
        let foundations = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(foundations)
    }

    #[instrument(skip(self), err)]
    async fn list_members(&self, foundation: &str) -> Result<Vec<Member>> {
        trace!("db: list members");

        let db = self.pool.get().await?;
        let row = db
            .query_one("select list_members($1::text)::text", &[&foundation])
            .await?;
        let members = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn list_projects(&self, foundation: &str) -> Result<Vec<Project>> {
        trace!("db: list projects");

        let db = self.pool.get().await?;
        let row = db
            .query_one("select list_projects($1::text)::text", &[&foundation])
            .await?;
        let projects = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(projects)
    }

    #[instrument(skip(self), err)]
    async fn remove_member(&self, foundation: &str, member_name: &str) -> Result<()> {
        trace!("db: remove member");

        let db = self.pool.get().await?;
        db.execute(
            "select remove_member($1::text, $2::text);",
            &[&foundation, &member_name],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn remove_project(&self, foundation: &str, project_name: &str) -> Result<()> {
        trace!("db: remove project");

        let db = self.pool.get().await?;
        db.execute(
            "select remove_project($1::text, $2::text);",
            &[&foundation, &project_name],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn update_member(&self, member: &Member) -> Result<()> {
        trace!("db: update member");

        let db = self.pool.get().await?;
        db.execute(
            "select update_member($1::text, $2::text, $3::text, $4::text);",
            &[&member.foundation, &member.name, &member.level, &member.logo_url],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn update_project(&self, project: &Project) -> Result<()> {
        trace!("db: update project");

        let db = self.pool.get().await?;
        db.execute(
            "select update_project($1::text, $2::text, $3::text, $4::text);",
            &[
                &project.foundation,
                &project.name,
                &project.maturity,
                &project.logo_url,
            ],
        )
        .await?;

        Ok(())
    }
}
