//! This module defines database operations used by background task workers, such as
//! archiving expired jobs.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::db::PgDB;

/// Trait for database operations required by background tasks workers.
#[async_trait]
pub(crate) trait DBWorkers {
    /// Archives jobs that have expired based on their published date.
    async fn archive_expired_jobs(&self) -> Result<()>;
}

#[async_trait]
impl DBWorkers for PgDB {
    #[instrument(skip(self), err)]
    async fn archive_expired_jobs(&self) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute("select archive_expired_jobs()", &[]).await?;

        Ok(())
    }
}
