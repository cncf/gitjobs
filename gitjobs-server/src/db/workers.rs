//! This module defines some database functionality used in tasks workers.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::db::PgDB;

/// Trait that defines some database operations used in tasks workers.
#[async_trait]
pub(crate) trait DBWorkers {
    /// Archive expired jobs.
    async fn archive_expired_jobs(&self) -> Result<()>;
}

#[async_trait]
impl DBWorkers for PgDB {
    #[instrument(skip(self), err)]
    async fn archive_expired_jobs(&self) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'archived',
                archived_at = current_timestamp,
                updated_at = current_timestamp
            where status = 'published'
            and published_at + '30 days'::interval < current_timestamp;
            ",
            &[],
        )
        .await?;

        Ok(())
    }
}
