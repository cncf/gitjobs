//! This module defines some database functionality for the job board.

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::Json;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    PgDB,
    templates::{
        dashboard::employer::jobs::Job,
        jobboard::jobs::{Filters, FiltersOptions},
    },
};

/// Trait that defines some database operations used in the job board.
#[async_trait]
pub(crate) trait DBJobBoard {
    /// Get filters options used to search jobs.
    async fn get_jobs_filters_options(&self) -> Result<FiltersOptions>;

    /// Search jobs.
    async fn search_jobs(&self, job_board_id: &Uuid, filters: &Filters) -> Result<JobsSearchOutput>;
}

#[async_trait]
impl DBJobBoard for PgDB {
    /// [DBJobBoard::get_jobs_filters_options]
    #[instrument(skip(self))]
    async fn get_jobs_filters_options(&self) -> Result<FiltersOptions> {
        // Query database
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select json_build_object(
                    'kind', (
                        select coalesce(json_agg(json_build_object(
                            'name', name,
                            'value', name
                        )), '[]')
                        from (
                            select name
                            from job_kind
                            order by name asc
                        ) as kinds
                    ),
                    'workplace', (
                        select coalesce(json_agg(json_build_object(
                            'name', name,
                            'value', name
                        )), '[]')
                        from (
                            select name
                            from workplace
                            order by name asc
                        ) as workplaces
                    )
                )::text as filters_options;
                ",
                &[],
            )
            .await?;

        // Prepare filters options
        let filters_options = serde_json::from_str(&row.get::<_, String>("filters_options"))?;

        Ok(filters_options)
    }

    /// [DBJobBoard::search_jobs]
    #[instrument(skip(self))]
    async fn search_jobs(&self, job_board_id: &Uuid, filters: &Filters) -> Result<JobsSearchOutput> {
        // Query database
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select jobs::text, total from search_jobs($1::uuid, $2::jsonb)",
                &[&job_board_id, &Json(filters)],
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
    pub jobs: Vec<Job>,
    pub total: Total,
}

/// Type alias to represent the total count.
pub(crate) type Total = usize;
