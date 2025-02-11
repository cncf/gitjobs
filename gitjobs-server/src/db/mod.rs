//! This module defines an abstraction layer over the database.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use auth::DBAuth;
use dashboard::DBDashBoard;
use deadpool_postgres::Pool;
use img::DBImage;
use tracing::instrument;
use uuid::Uuid;

use crate::templates::misc::Location;

mod auth;
mod dashboard;
pub(crate) mod img;

/// Abstraction layer over the database. Trait that defines some operations a
/// DB implementation must support.
#[async_trait]
pub(crate) trait DB: DBAuth + DBDashBoard + DBImage {
    /// Get the job board id from the host provided.
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>>;

    /// Search locations.
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>>;
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
    /// [DB::get_job_board_id]
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

    /// [DB::search_locations]
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
