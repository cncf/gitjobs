//! This module defines some database functionality used across the site.

use anyhow::Result;
use async_trait::async_trait;
use tracing::{instrument, trace};

use crate::{
    PgDB,
    templates::misc::{Location, Member, Project},
};

/// Trait that defines common database operations used across the site.
#[async_trait]
pub(crate) trait DBMisc {
    /// Searches for locations matching the provided query string.
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>>;

    /// Searches for members in a foundation matching the provided member name.
    async fn search_members(&self, foundation: &str, member: &str) -> Result<Vec<Member>>;

    /// Searches for projects in a foundation matching the provided project name.
    async fn search_projects(&self, foundation: &str, project: &str) -> Result<Vec<Project>>;
}

#[async_trait]
impl DBMisc for PgDB {
    #[instrument(skip(self), err)]
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>> {
        trace!("db: search locations");

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
    async fn search_members(&self, foundation: &str, member: &str) -> Result<Vec<Member>> {
        trace!("db: search members");

        let db = self.pool.get().await?;
        let members = db
            .query(
                "
                select
                    member_id,
                    foundation,
                    level,
                    logo_url,
                    name
                from member
                where foundation = $1::text
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&foundation, &member],
            )
            .await?
            .into_iter()
            .map(|row| Member {
                member_id: row.get("member_id"),
                foundation: row.get("foundation"),
                level: row.get("level"),
                logo_url: row.get("logo_url"),
                name: row.get("name"),
            })
            .collect();

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn search_projects(&self, foundation: &str, project: &str) -> Result<Vec<Project>> {
        trace!("db: search projects");

        let db = self.pool.get().await?;
        let projects = db
            .query(
                "
                select
                    project_id,
                    foundation,
                    logo_url,
                    maturity,
                    name
                from project
                where foundation = $1::text
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&foundation, &project],
            )
            .await?
            .into_iter()
            .map(|row| Project {
                project_id: row.get("project_id"),
                foundation: row.get("foundation"),
                logo_url: row.get("logo_url"),
                maturity: row.get("maturity"),
                name: row.get("name"),
            })
            .collect();

        Ok(projects)
    }
}
