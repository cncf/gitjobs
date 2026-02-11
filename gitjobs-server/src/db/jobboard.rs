//! This module defines some database functionality for the job board.

use std::time::Duration;

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
    templates::jobboard::{
        jobs::{Filters, FiltersOptions, Job, JobSummary},
        stats::Stats,
    },
};

/// Trait for database operations used by the job board, such as applying and searching jobs.
#[async_trait]
pub(crate) trait DBJobBoard {
    /// Records a user's application to a job. Returns `true` if the
    /// application was successfully recorded or `false` otherwise.
    async fn apply_to_job(&self, job_id: &Uuid, user_id: &Uuid) -> Result<bool>;

    /// Fetches a job for the job board by its unique identifier.
    async fn get_job_jobboard(&self, job_id: &Uuid) -> Result<Option<Job>>;

    /// Retrieves available filter options for job searches.
    async fn get_jobs_filters_options(&self) -> Result<FiltersOptions>;

    /// Retrieves statistics about the job board.
    async fn get_stats(&self) -> Result<Stats>;

    /// Searches for jobs using the provided filter criteria.
    async fn search_jobs(&self, filters: &Filters) -> Result<JobsSearchOutput>;
}

/// Implementation of `DBJobBoard` for the `PostgreSQL` database backend.
#[async_trait]
impl DBJobBoard for PgDB {
    #[instrument(skip(self), err)]
    async fn apply_to_job(&self, job_id: &Uuid, user_id: &Uuid) -> Result<bool> {
        trace!("db: apply to job");

        let db = self.pool.get().await?;
        let applied = db
            .query_one(
                "select jobboard_apply_to_job($1::uuid, $2::uuid);",
                &[&job_id, &user_id],
            )
            .await?;

        Ok(applied.get(0))
    }

    #[instrument(skip(self), err)]
    async fn get_job_jobboard(&self, job_id: &Uuid) -> Result<Option<Job>> {
        trace!("db: get job for jobboard");

        let db = self.pool.get().await?;
        let json_data: Option<String> = db
            .query_one("select jobboard_get_job_jobboard($1::uuid)::text;", &[&job_id])
            .await?
            .get(0);
        let job = json_data.map(|data| serde_json::from_str(&data)).transpose()?;

        Ok(job)
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

            let row = db
                .query_one("select jobboard_get_jobs_filters_options()::text;", &[])
                .await?;
            let filters_options = serde_json::from_str(&row.get::<_, String>(0))?;

            Ok(filters_options)
        }

        let db = self.pool.get().await?;
        inner(db).await
    }

    #[instrument(skip(self))]
    async fn get_stats(&self) -> Result<Stats> {
        trace!("db: get stats");

        // Query database
        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select jobboard_get_stats()::text as stats;", &[])
            .await?
            .get("stats");
        let stats = serde_json::from_str(&json_data)?;

        Ok(stats)
    }

    #[instrument(skip(self))]
    async fn search_jobs(&self, filters: &Filters) -> Result<JobsSearchOutput> {
        trace!("db: search jobs");

        // Query database
        let db = self.pool.get().await?;
        let row = db
            .query_one("select jobboard_search_jobs($1::jsonb)::text", &[&Json(filters)])
            .await?;
        let output = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(output)
    }
}

/// Output for job search, including job summaries and total count.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobsSearchOutput {
    /// List of jobs matching the search criteria.
    pub jobs: Vec<JobSummary>,
    /// Total number of jobs matching the search criteria.
    pub total: usize,
}
