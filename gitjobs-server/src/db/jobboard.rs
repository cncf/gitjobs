//! This module defines some database functionality for the job board.

use anyhow::Result;
use async_trait::async_trait;
use cached::proc_macro::cached;
use deadpool_postgres::Object;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::Json;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    db::misc::Total,
    templates::jobboard::jobs::{Filters, FiltersOptions, Job, JobSummary},
};

/// Trait that defines some database operations used in the job board.
#[async_trait]
pub(crate) trait DBJobBoard {
    /// Apply to a job.
    async fn apply_to_job(&self, job_id: &Uuid, user_id: &Uuid) -> Result<()>;

    /// Get job.
    async fn get_job_jobboard(&self, job_id: &Uuid) -> Result<Option<Job>>;

    /// Get filters options used to search jobs.
    async fn get_jobs_filters_options(&self) -> Result<FiltersOptions>;

    /// Search jobs.
    async fn search_jobs(&self, filters: &Filters) -> Result<JobsSearchOutput>;
}

#[async_trait]
impl DBJobBoard for PgDB {
    #[instrument(skip(self), err)]
    async fn apply_to_job(&self, job_id: &Uuid, user_id: &Uuid) -> Result<()> {
        trace!("db: apply to job");

        let db = self.pool.get().await?;
        db.execute(
            "
            insert into application (
                job_id,
                job_seeker_profile_id
            ) values (
                $1::uuid,
                (select job_seeker_profile_id from job_seeker_profile where user_id = $2::uuid)
            )
            on conflict (job_seeker_profile_id, job_id) do nothing;
            ",
            &[&job_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_job_jobboard(&self, job_id: &Uuid) -> Result<Option<Job>> {
        trace!("db: get job for jobboard");

        let db = self.pool.get().await?;
        let row = db
            .query_opt(
                "
                select
                    j.description,
                    j.job_id,
                    j.kind,
                    j.title,
                    j.workplace,
                    j.apply_instructions,
                    j.apply_url,
                    j.benefits,
                    j.location_id,
                    j.open_source,
                    j.published_at,
                    j.qualifications,
                    j.responsibilities,
                    j.salary,
                    j.salary_currency,
                    j.salary_min,
                    j.salary_max,
                    j.salary_period,
                    j.seniority,
                    j.skills,
                    j.updated_at,
                    j.upstream_commitment,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'company', e.company,
                            'description', e.description,
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
                        )), '{}'::jsonb)
                    ) as employer,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'location_id', l.location_id,
                            'city', l.city,
                            'country', l.country,
                            'state', l.state
                        )), '{}'::jsonb)
                    ) as location,
                    (
                        select json_agg(json_build_object(
                            'project_id', p.project_id,
                            'foundation', p.foundation,
                            'logo_url', p.logo_url,
                            'maturity', p.maturity,
                            'name', p.name
                        ))
                        from project p
                        left join job_project jp using (project_id)
                        left join job j using (job_id)
                        where j.job_id = $1::uuid
                    ) as projects
                from job j
                join employer e on j.employer_id = e.employer_id
                left join location l on j.location_id = l.location_id
                left join member m on e.member_id = m.member_id
                where job_id = $1::uuid
                and status = 'published';
                ",
                &[&job_id],
            )
            .await?;

        if let Some(row) = row {
            let job = Job {
                description: row.get("description"),
                employer: serde_json::from_value(row.get::<_, serde_json::Value>("employer"))
                    .expect("employer should be valid json"),
                job_id: row.get("job_id"),
                kind: row.get::<_, String>("kind").parse().expect("valid job kind"),
                title: row.get("title"),
                workplace: row.get::<_, String>("workplace").parse().expect("valid workplace"),
                apply_instructions: row.get("apply_instructions"),
                apply_url: row.get("apply_url"),
                benefits: row.get("benefits"),
                location: row
                    .get::<_, Option<serde_json::Value>>("location")
                    .map(|v| serde_json::from_value(v).expect("location should be valid json")),
                open_source: row.get("open_source"),
                projects: row
                    .get::<_, Option<serde_json::Value>>("projects")
                    .map(|v| serde_json::from_value(v).expect("projects should be valid json")),
                published_at: row.get("published_at"),
                qualifications: row.get("qualifications"),
                responsibilities: row.get("responsibilities"),
                salary: row.get("salary"),
                salary_currency: row.get("salary_currency"),
                salary_min: row.get("salary_min"),
                salary_max: row.get("salary_max"),
                salary_period: row.get("salary_period"),
                seniority: row
                    .get::<_, Option<String>>("seniority")
                    .map(|s| s.parse().expect("valid seniority")),
                skills: row.get("skills"),
                updated_at: row.get("updated_at"),
                upstream_commitment: row.get("upstream_commitment"),
            };

            Ok(Some(job))
        } else {
            Ok(None)
        }
    }

    #[instrument(skip(self))]
    async fn get_jobs_filters_options(&self) -> Result<FiltersOptions> {
        #[cached(
            time = 3600,
            key = "&str",
            convert = r#"{ "jobs_filters_options" }"#,
            sync_writes = "by_key",
            result = true
        )]
        async fn inner(db: Object) -> Result<FiltersOptions> {
            trace!("db: get jobs filters options");

            // Query database
            let row = db
                .query_one(
                    "
                select
                    (
                        select json_agg(json_build_object(
                            'project_id', project_id,
                            'foundation', foundation,
                            'logo_url', logo_url,
                            'maturity', maturity,
                            'name', name
                        ))
                        from project
                    )::text as projects;
                ",
                    &[],
                )
                .await?;

            // Prepare filters options
            let filters_options = FiltersOptions {
                projects: serde_json::from_str(&row.get::<_, String>("projects"))?,
            };

            Ok(filters_options)
        }

        let db = self.pool.get().await?;
        inner(db).await
    }

    #[instrument(skip(self))]
    async fn search_jobs(&self, filters: &Filters) -> Result<JobsSearchOutput> {
        trace!("db: search jobs");

        // Query database
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select jobs::text, total from search_jobs($1::jsonb)",
                &[&Json(filters)],
            )
            .await?;

        // Prepare search output
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let output = JobsSearchOutput {
            jobs: serde_json::from_str(&row.get::<_, String>("jobs"))?,
            total: row.get::<_, i64>("total") as usize,
        };

        Ok(output)
    }
}

/// Jobs search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobsSearchOutput {
    pub jobs: Vec<JobSummary>,
    pub total: Total,
}
