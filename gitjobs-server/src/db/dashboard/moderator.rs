//! This module defines some database functionality for the moderator
//! dashboard.

use anyhow::Result;
use async_trait::async_trait;
use tracing::{instrument, trace};

use crate::{PgDB, templates::dashboard::moderator::jobs::JobSummary};

/// Trait that defines some database operations used in the moderator
/// dashboard.
#[async_trait]
pub(crate) trait DBDashBoardModerator {
    /// List moderation pending jobs.
    async fn list_moderation_pending_jobs(&self) -> Result<Vec<JobSummary>>;
}

#[async_trait]
impl DBDashBoardModerator for PgDB {
    #[instrument(skip(self), err)]
    async fn list_moderation_pending_jobs(&self) -> Result<Vec<JobSummary>> {
        trace!("db: list moderation pending jobs");

        let db = self.pool.get().await?;
        let jobs = db
            .query(
                "
                select
                    j.job_id,
                    j.title,
                    j.published_at,
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
                order by j.published_at desc;
                ",
                &[],
            )
            .await?
            .into_iter()
            .map(|row| JobSummary {
                job_id: row.get("job_id"),
                title: row.get("title"),
                published_at: row.get("published_at"),
                employer: serde_json::from_value(row.get::<_, serde_json::Value>("employer"))
                    .expect("employer should be valid"),
            })
            .collect();

        Ok(jobs)
    }
}
