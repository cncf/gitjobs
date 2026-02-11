//! This module defines database functionality used in the event tracker, including
//! operations for updating job view and search appearance counts.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::{instrument, trace};

use crate::{
    db::PgDB,
    event_tracker::{Day, JobId, Total},
};

/// Lock key used to synchronize updates to job views in the database.
const LOCK_KEY_UPDATE_JOBS_VIEWS: i64 = 1;

/// Lock key used to synchronize updates to search appearances in the database.
const LOCK_KEY_UPDATE_SEARCH_APPEARANCES: i64 = 2;

/// Trait that defines database operations used in the event tracker.
#[async_trait]
pub(crate) trait DBEventTracker {
    /// Updates the number of views for the provided jobs and days.
    async fn update_jobs_views(&self, data: Vec<(JobId, Day, Total)>) -> Result<()>;

    /// Updates the number of search appearances for the provided jobs and days.
    async fn update_search_appearances(&self, data: Vec<(JobId, Day, Total)>) -> Result<()>;
}

/// Type alias for a thread-safe, reference-counted `DBEventTracker` trait object.
pub(crate) type DynDBEventTracker = Arc<dyn DBEventTracker + Send + Sync>;

#[async_trait]
impl DBEventTracker for PgDB {
    #[instrument(skip(self), err)]
    async fn update_jobs_views(&self, data: Vec<(JobId, Day, Total)>) -> Result<()> {
        trace!("db: update jobs views");

        let db = self.pool.get().await?;
        db.execute(
            "select jobboard_update_jobs_views($1::bigint, $2::jsonb)",
            &[&LOCK_KEY_UPDATE_JOBS_VIEWS, &Json(&data)],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn update_search_appearances(&self, data: Vec<(JobId, Day, Total)>) -> Result<()> {
        trace!("db: update search appearances");

        let db = self.pool.get().await?;
        db.execute(
            "select jobboard_update_search_appearances($1::bigint, $2::jsonb)",
            &[&LOCK_KEY_UPDATE_SEARCH_APPEARANCES, &Json(&data)],
        )
        .await?;

        Ok(())
    }
}
