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
        let row = db
            .query_one("select search_locations_json($1::text)::text;", &[&ts_query])
            .await?;
        let locations = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(locations)
    }

    #[instrument(skip(self), err)]
    async fn search_members(&self, foundation: &str, member: &str) -> Result<Vec<Member>> {
        trace!("db: search members");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select search_members($1::text, $2::text)::text",
                &[&foundation, &member],
            )
            .await?;
        let members = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn search_projects(&self, foundation: &str, project: &str) -> Result<Vec<Project>> {
        trace!("db: search projects");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select search_projects($1::text, $2::text)::text",
                &[&foundation, &project],
            )
            .await?;
        let projects = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(projects)
    }
}
