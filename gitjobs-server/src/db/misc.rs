//! This module defines some database functionality used across the site.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    PgDB,
    templates::misc::{Location, Member, Project},
};

/// Trait that defines some common database operations used across the site.
#[async_trait]
pub(crate) trait DBMisc {
    /// Get the job board id from the host provided.
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>>;

    /// Search locations.
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>>;

    /// Search members.
    async fn search_members(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Member>>;

    /// Search projects.
    async fn search_projects(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Project>>;
}

#[async_trait]
impl DBMisc for PgDB {
    #[instrument(skip(self), err)]
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>> {
        let db = self.pool.get().await?;
        let job_board_id = db
            .query_opt(
                "
                select job_board_id
                from job_board
                where host = $1::text
                and active = true;
                ",
                &[&host],
            )
            .await?
            .map(|row| row.get("job_board_id"));

        Ok(job_board_id)
    }

    #[instrument(skip(self), err)]
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>> {
        let db = self.pool.get().await?;
        let locations = db
            .query(
                "
                select
                    location_id,
                    city,
                    country,
                    state
                from search_locations($1::text);
                ",
                &[&ts_query],
            )
            .await?
            .into_iter()
            .map(|row| Location {
                location_id: row.get("location_id"),
                city: row.get("city"),
                country: row.get("country"),
                state: row.get("state"),
            })
            .collect();

        Ok(locations)
    }

    #[instrument(skip(self), err)]
    async fn search_members(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Member>> {
        let db = self.pool.get().await?;
        let members = db
            .query(
                "
                select
                    member_id,
                    name,
                    level,
                    logo_url
                from member
                where job_board_id = $1::uuid
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&job_board_id, &name],
            )
            .await?
            .into_iter()
            .map(|row| Member {
                member_id: row.get("member_id"),
                name: row.get("name"),
                level: row.get("level"),
                logo_url: row.get("logo_url"),
            })
            .collect();

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn search_projects(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Project>> {
        let db = self.pool.get().await?;
        let projects = db
            .query(
                "
                select
                    project_id,
                    name,
                    maturity,
                    logo_url
                from project
                where job_board_id = $1::uuid
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&job_board_id, &name],
            )
            .await?
            .into_iter()
            .map(|row| Project {
                project_id: row.get("project_id"),
                name: row.get("name"),
                maturity: row.get("maturity"),
                logo_url: row.get("logo_url"),
            })
            .collect();

        Ok(projects)
    }
}

/// Type alias to represent the total count.
pub(crate) type Total = usize;
