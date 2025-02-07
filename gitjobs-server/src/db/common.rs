//! This module defines some common database functionality.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::templates::common::Location;

use crate::PgDB;

/// Trait that defines some common database operations.
#[async_trait]
pub(crate) trait DBCommon {
    /// Get the job board id from the host provided.
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>>;

    /// Search locations.
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>>;
}

#[async_trait]
impl DBCommon for PgDB {
    /// [DBCommon::get_job_board_id]
    #[instrument(skip(self), err)]
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>> {
        let db = self.pool.get().await?;
        let job_board_id = db
            .query_opt(
                "
                select job_board_id
                from job_board
                where host = $1::text
                and active = true
                ",
                &[&host],
            )
            .await?
            .map(|row| row.get("job_board_id"));

        Ok(job_board_id)
    }

    /// [DBCommon::search_locations]
    #[instrument(skip(self), err)]
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>> {
        let db = self.pool.get().await?;
        let locations = db
            .query("select * from search_locations($1::text)", &[&ts_query])
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
}
