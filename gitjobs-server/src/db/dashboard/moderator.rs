//! This module defines some database functionality for the moderator
//! dashboard.

use anyhow::Result;
use async_trait::async_trait;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{PgDB, templates::dashboard::moderator::jobs::JobSummary};

/// Trait that defines some database operations used in the moderator
/// dashboard.
#[async_trait]
pub(crate) trait DBDashBoardModerator {
    /// Approve job.
    async fn approve_job(&self, job_id: &Uuid, reviewer: &Uuid) -> Result<()>;

    /// List moderation pending jobs.
    async fn list_moderation_pending_jobs(&self) -> Result<Vec<JobSummary>>;

    /// Reject job.
    async fn reject_job(&self, job_id: &Uuid, reviewer: &Uuid, review_notes: Option<&String>) -> Result<()>;
}

#[async_trait]
impl DBDashBoardModerator for PgDB {
    #[instrument(skip(self), err)]
    async fn approve_job(&self, job_id: &Uuid, reviewer: &Uuid) -> Result<()> {
        trace!("db: approve job");

        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'published',
                published_at = current_timestamp,
                reviewed_at = current_timestamp,
                reviewed_by = $2
            where job_id = $1
            ",
            &[job_id, reviewer],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn list_moderation_pending_jobs(&self) -> Result<Vec<JobSummary>> {
        trace!("db: list moderation pending jobs");

        let db = self.pool.get().await?;
        let jobs = db
            .query(
                "
                select
                    j.created_at,
                    j.job_id,
                    j.title,
                    (
                        select jsonb_strip_nulls(jsonb_build_object(
                            'company', e.company,
                            'logo_id', e.logo_id,
                            'website_url', e.website_url,
                            'member', (
                                select nullif(jsonb_strip_nulls(jsonb_build_object(
                                    'member_id', m.member_id,
                                    'foundation', m.foundation,
                                    'level', m.level,
                                    'logo_url', m.logo_url,
                                    'name', m.name
                                )), '{}'::jsonb)
                            )
                        ))
                    ) as employer
                from job j
                join employer e on j.employer_id = e.employer_id
                left join member m on e.member_id = m.member_id
                where j.status = 'pending-approval'
                order by j.created_at desc;
                ",
                &[],
            )
            .await?
            .into_iter()
            .map(|row| JobSummary {
                created_at: row.get("created_at"),
                job_id: row.get("job_id"),
                title: row.get("title"),
                employer: serde_json::from_value(row.get::<_, serde_json::Value>("employer"))
                    .expect("employer should be valid"),
            })
            .collect();

        Ok(jobs)
    }

    #[instrument(skip(self), err)]
    async fn reject_job(&self, job_id: &Uuid, reviewer: &Uuid, review_notes: Option<&String>) -> Result<()> {
        trace!("db: reject job");

        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'rejected',
                review_notes = $3,
                reviewed_at = current_timestamp,
                reviewed_by = $2
            where job_id = $1;
            ",
            &[job_id, reviewer, &review_notes],
        )
        .await?;

        Ok(())
    }
}
