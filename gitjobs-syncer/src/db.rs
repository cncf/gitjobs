//! This module defines an abstraction layer over the database.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use deadpool_postgres::Pool;
use tracing::{instrument, trace};

use crate::syncer::{Foundation, Member, Project};

/// Abstraction layer over the database. Trait that defines some operations a
/// DB implementation must support.
#[async_trait]
pub(crate) trait DB {
    /// Add foundation member.
    async fn add_member(&self, member: &Member) -> Result<()>;

    /// Add foundation project.
    async fn add_project(&self, project: &Project) -> Result<()>;

    /// Delete foundation member.
    async fn delete_member(&self, foundation: &str, member_name: &str) -> Result<()>;

    /// Delete foundation project.
    async fn delete_project(&self, foundation: &str, project_name: &str) -> Result<()>;

    /// List foundations.
    async fn list_foundations(&self) -> Result<Vec<Foundation>>;

    /// List foundation members.
    async fn list_members(&self, foundation: &str) -> Result<Vec<Member>>;

    /// List foundation projects.
    async fn list_projects(&self, foundation: &str) -> Result<Vec<Project>>;

    /// Update foundation member.
    async fn update_member(&self, member: &Member) -> Result<()>;

    /// Update foundation project.
    async fn update_project(&self, project: &Project) -> Result<()>;
}

/// Type alias to represent a DB trait object.
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

/// DB implementation backed by `PostgreSQL`.
pub(crate) struct PgDB {
    pool: Pool,
}

impl PgDB {
    /// Create a new `PgDB` instance.
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
            "
            insert into member (
                foundation,
                name,
                level,
                logo_url
            ) values ($1, $2, $3, $4);
            ",
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
            "
            insert into project (
                foundation,
                name,
                maturity,
                logo_url
            ) values ($1, $2, $3, $4);
            ",
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
    async fn delete_member(&self, foundation: &str, member_name: &str) -> Result<()> {
        trace!("db: delete member");

        let db = self.pool.get().await?;
        db.execute(
            "delete from member where foundation = $1 and name = $2;",
            &[&foundation, &member_name],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn delete_project(&self, foundation: &str, project_name: &str) -> Result<()> {
        trace!("db: delete project");

        let db = self.pool.get().await?;
        db.execute(
            "delete from project where foundation = $1 and name = $2;",
            &[&foundation, &project_name],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn list_foundations(&self) -> Result<Vec<Foundation>> {
        trace!("db: list foundations");

        let db = self.pool.get().await?;
        let foundations = db
            .query(
                "
                select
                    name,
                    landscape_url
                from foundation
                where landscape_url is not null;
                ",
                &[],
            )
            .await?
            .into_iter()
            .map(|row| Foundation {
                name: row.get("name"),
                landscape_url: row.get("landscape_url"),
            })
            .collect();

        Ok(foundations)
    }

    #[instrument(skip(self), err)]
    async fn list_members(&self, foundation: &str) -> Result<Vec<Member>> {
        trace!("db: list members");

        let db = self.pool.get().await?;
        let members = db
            .query(
                "
                select
                    name,
                    level,
                    logo_url
                from member
                where foundation = $1;
                ",
                &[&foundation],
            )
            .await?
            .into_iter()
            .map(|row| Member {
                foundation: foundation.to_string(),
                name: row.get("name"),
                level: row.get("level"),
                logo_url: row.get("logo_url"),
            })
            .collect();

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn list_projects(&self, foundation: &str) -> Result<Vec<Project>> {
        trace!("db: list projects");

        let db = self.pool.get().await?;
        let projects = db
            .query(
                "
                select
                    name,
                    logo_url,
                    maturity
                from project
                where foundation = $1;
                ",
                &[&foundation],
            )
            .await?
            .into_iter()
            .map(|row| Project {
                foundation: foundation.to_string(),
                name: row.get("name"),
                logo_url: row.get("logo_url"),
                maturity: row.get("maturity"),
            })
            .collect();

        Ok(projects)
    }

    #[instrument(skip(self), err)]
    async fn update_member(&self, member: &Member) -> Result<()> {
        trace!("db: update member");

        let db = self.pool.get().await?;
        db.execute(
            "
            update member set
                level = $3,
                logo_url = $4
            where foundation = $1 and name = $2;
            ",
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
            "
            update project set
                maturity = $3,
                logo_url = $4
            where foundation = $1 and name = $2;
            ",
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
