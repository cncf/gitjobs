//! This module defines database operations for the moderator dashboard.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    templates::dashboard::{employer::jobs::JobStatus, moderator::jobs::JobSummary},
};

/// Trait for moderator dashboard database operations.
#[async_trait]
pub(crate) trait DBDashBoardModerator {
    /// Approves a job and updates its status and review metadata.
    async fn approve_job(&self, job_id: &Uuid, reviewer: &Uuid) -> Result<Option<DateTime<Utc>>>;

    /// Lists jobs for moderation filtered by the given status.
    async fn list_jobs_for_moderation(&self, status: JobStatus) -> Result<Vec<JobSummary>>;

    /// Rejects a job, optionally adding review notes and updating review metadata.
    async fn reject_job(&self, job_id: &Uuid, reviewer: &Uuid, review_notes: Option<String>) -> Result<()>;
}

#[async_trait]
impl DBDashBoardModerator for PgDB {
    #[instrument(skip(self), err)]
    async fn approve_job(&self, job_id: &Uuid, reviewer: &Uuid) -> Result<Option<DateTime<Utc>>> {
        trace!("db: approve job");

        let db = self.pool.get().await?;
        let first_published_at = db
            .query_one(
                "select dashboard_moderator_approve_job($1::uuid, $2::uuid)",
                &[job_id, reviewer],
            )
            .await?
            .get(0);

        Ok(first_published_at)
    }

    #[instrument(skip(self), err)]
    async fn list_jobs_for_moderation(&self, status: JobStatus) -> Result<Vec<JobSummary>> {
        trace!("db: list jobs for moderation");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select dashboard_moderator_list_jobs_for_moderation($1::text)::text",
                &[&status.to_string()],
            )
            .await?;
        let jobs = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(jobs)
    }

    #[instrument(skip(self), err)]
    async fn reject_job(&self, job_id: &Uuid, reviewer: &Uuid, review_notes: Option<String>) -> Result<()> {
        trace!("db: reject job");

        let db = self.pool.get().await?;
        db.execute(
            "select dashboard_moderator_reject_job($1::uuid, $2::uuid, $3::text);",
            &[job_id, reviewer, &review_notes],
        )
        .await?;

        Ok(())
    }
}
