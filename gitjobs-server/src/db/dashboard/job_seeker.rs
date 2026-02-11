//! This module defines database operations for the job seeker dashboard.

use anyhow::Result;
use async_trait::async_trait;
use tokio_postgres::types::Json;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    templates::dashboard::job_seeker::{applications::Application, profile::JobSeekerProfile},
};

/// Trait for job seeker dashboard database operations.
#[async_trait]
pub(crate) trait DBDashBoardJobSeeker {
    /// Cancels a job application for the given user.
    async fn cancel_application(&self, application_id: &Uuid, user_id: &Uuid) -> Result<()>;

    /// Retrieves the job seeker profile for the given user.
    async fn get_job_seeker_profile(&self, user_id: &Uuid) -> Result<Option<JobSeekerProfile>>;

    /// Lists all job applications for the given user.
    async fn list_job_seeker_applications(&self, user_id: &Uuid) -> Result<Vec<Application>>;

    /// Updates the job seeker profile for the given user.
    async fn update_job_seeker_profile(&self, user_id: &Uuid, profile: &JobSeekerProfile) -> Result<()>;
}

#[async_trait]
impl DBDashBoardJobSeeker for PgDB {
    #[instrument(skip(self), err)]
    async fn cancel_application(&self, application_id: &Uuid, user_id: &Uuid) -> Result<()> {
        trace!("db: cancel application");

        let db = self.pool.get().await?;
        db.execute(
            "select dashboard_job_seeker_cancel_application($1::uuid, $2::uuid)",
            &[&application_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_job_seeker_profile(&self, user_id: &Uuid) -> Result<Option<JobSeekerProfile>> {
        trace!("db: get job seeker profile");

        let db = self.pool.get().await?;
        let row = db
            .query_one("select dashboard_job_seeker_get_profile($1::uuid)", &[&user_id])
            .await?;
        let profile = row
            .get::<_, Option<serde_json::Value>>(0)
            .map(serde_json::from_value)
            .transpose()?;

        Ok(profile)
    }

    #[instrument(skip(self), err)]
    async fn list_job_seeker_applications(&self, user_id: &Uuid) -> Result<Vec<Application>> {
        trace!("db: list job seeker applications");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select dashboard_job_seeker_list_applications($1::uuid)::text",
                &[&user_id],
            )
            .await?;
        let applications = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(applications)
    }

    #[instrument(skip(self), err)]
    async fn update_job_seeker_profile(&self, user_id: &Uuid, profile: &JobSeekerProfile) -> Result<()> {
        trace!("db: update job seeker profile");

        let db = self.pool.get().await?;
        db.execute(
            "select dashboard_job_seeker_upsert_profile($1::uuid, $2::jsonb)",
            &[&user_id, &Json(profile)],
        )
        .await?;

        Ok(())
    }
}
