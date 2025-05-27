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
    async fn reject_job(&self, job_id: &Uuid, reviewer: &Uuid, review_notes: Option<&String>) -> Result<()>;
}

#[async_trait]
impl DBDashBoardModerator for PgDB {
    #[instrument(skip(self), err)]
    async fn approve_job(&self, job_id: &Uuid, reviewer: &Uuid) -> Result<Option<DateTime<Utc>>> {
        trace!("db: approve job");

        let db = self.pool.get().await?;
        let first_published_at = db
            .query_opt(
                "
                with old as (
                    select first_published_at from job where job_id = $1
                )
                update job
                set
                    status = 'published',
                    first_published_at = coalesce(first_published_at, current_timestamp),
                    published_at = current_timestamp,
                    reviewed_at = current_timestamp,
                    reviewed_by = $2
                where job_id = $1
                returning (select first_published_at from old);
                ",
                &[job_id, reviewer],
            )
            .await?
            .map(|row| row.get("first_published_at"));

        Ok(first_published_at)
    }

    #[instrument(skip(self), err)]
    async fn list_jobs_for_moderation(&self, status: JobStatus) -> Result<Vec<JobSummary>> {
        trace!("db: list jobs for moderation");

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
                            'employer_id', e.employer_id,
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
                where j.status = $1
                order by j.created_at desc;
                ",
                &[&status.to_string()],
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
