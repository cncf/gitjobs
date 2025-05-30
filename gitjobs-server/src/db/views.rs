//! This module defines database functionality used in the views tracker, including
//! operations for updating job view counts.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
#[cfg(test)]
use mockall::automock;
use tokio_postgres::types::Json;
use tracing::{instrument, trace};

use crate::{
    db::PgDB,
    views::{Day, JobId, Total},
};

/// Lock key used to synchronize updates to job views in the database.
const LOCK_KEY_UPDATE_JOBS_VIEWS: i64 = 1;

/// Trait that defines database operations used in the views tracker.
#[async_trait]
#[cfg_attr(test, automock)]
pub(crate) trait DBViews {
    /// Updates the number of views for the provided jobs and days.
    async fn update_jobs_views(&self, data: Vec<(JobId, Day, Total)>) -> Result<()>;
}

/// Type alias for a thread-safe, reference-counted `DBViews` trait object.
pub(crate) type DynDBViews = Arc<dyn DBViews + Send + Sync>;

#[async_trait]
impl DBViews for PgDB {
    #[instrument(skip(self), err)]
    async fn update_jobs_views(&self, data: Vec<(JobId, Day, Total)>) -> Result<()> {
        trace!("db: update jobs views");

        let db = self.pool.get().await?;
        db.execute(
            "select update_jobs_views($1::bigint, $2::jsonb)",
            &[&LOCK_KEY_UPDATE_JOBS_VIEWS, &Json(&data)],
        )
        .await?;

        Ok(())
    }
}
