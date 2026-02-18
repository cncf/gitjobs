//! This module defines database operations for the employer dashboard.

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tokio_postgres::types::Json;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    templates::{
        dashboard::employer::{
            applications::{self, Application},
            employers::{Employer, EmployerSummary},
            jobs::{Job, JobStats, JobSummary},
            team::{TeamInvitation, TeamMember},
        },
        helpers::normalize_salary,
        misc::{Certification, Foundation},
    },
};

/// Trait for employer dashboard database operations.
#[async_trait]
pub(crate) trait DBDashBoardEmployer {
    /// Accepts a team member invitation for an employer.
    async fn accept_team_member_invitation(&self, employer_id: &Uuid, user_id: &Uuid) -> Result<()>;

    /// Adds a new employer to the database.
    async fn add_employer(&self, user_id: &Uuid, employer: &Employer) -> Result<Uuid>;

    /// Adds a new job for an employer.
    async fn add_job(&self, employer_id: &Uuid, job: &Job) -> Result<()>;

    /// Adds a team member to an employer's team.
    async fn add_team_member(&self, employer_id: &Uuid, email: &str) -> Result<Option<Uuid>>;

    /// Archives a job, marking it as no longer active.
    async fn archive_job(&self, job_id: &Uuid) -> Result<()>;

    /// Mark a job as deleted in the database (soft delete).
    async fn delete_job(&self, job_id: &Uuid) -> Result<()>;

    /// Deletes a team member from an employer's team.
    ///
    /// There must be at least one approved team member left after deletion.
    async fn delete_team_member(&self, employer_id: &Uuid, user_id: &Uuid) -> Result<()>;

    /// Retrieves available filter options for applications.
    async fn get_applications_filters_options(
        &self,
        employer_id: &Uuid,
    ) -> Result<applications::FiltersOptions>;

    /// Retrieves an employer's details.
    async fn get_employer(&self, employer_id: &Uuid) -> Result<Employer>;

    /// Retrieves a job's details for the dashboard.
    async fn get_job_dashboard(&self, job_id: &Uuid) -> Result<Job>;

    /// Retrieves the user ID for a job seeker profile.
    async fn get_job_seeker_user_id(&self, job_seeker_profile_id: &Uuid) -> Result<Option<Uuid>>;

    /// Retrieves a job's statistics.
    async fn get_job_stats(&self, job_id: &Uuid) -> Result<JobStats>;

    /// Retrieves the count of invitations for a user.
    async fn get_user_invitations_count(&self, user_id: &Uuid) -> Result<usize>;

    /// Lists all jobs for an employer.
    async fn list_employer_jobs(&self, employer_id: &Uuid) -> Result<Vec<JobSummary>>;

    /// Lists all employers where the user is a team member.
    async fn list_employers(&self, user_id: &Uuid) -> Result<Vec<EmployerSummary>>;

    /// Lists all available certifications.
    async fn list_certifications(&self) -> Result<Vec<Certification>>;

    /// Lists all available foundations.
    async fn list_foundations(&self) -> Result<Vec<Foundation>>;

    /// Lists all team members for an employer.
    async fn list_team_members(&self, employer_id: &Uuid) -> Result<Vec<TeamMember>>;

    /// Lists all invitations for a user.
    async fn list_user_invitations(&self, user_id: &Uuid) -> Result<Vec<TeamInvitation>>;

    /// Publishes a job, setting it to pending approval.
    async fn publish_job(&self, job_id: &Uuid) -> Result<()>;

    /// Searches applications for an employer with filters.
    async fn search_applications(
        &self,
        employer_id: &Uuid,
        filters: &applications::Filters,
    ) -> Result<ApplicationsSearchOutput>;

    /// Updates an employer's details.
    async fn update_employer(&self, employer_id: &Uuid, employer: &Employer) -> Result<()>;

    /// Updates a job's details.
    async fn update_job(&self, job_id: &Uuid, job: &Job) -> Result<()>;
}

#[async_trait]
impl DBDashBoardEmployer for PgDB {
    #[instrument(skip(self), err)]
    async fn accept_team_member_invitation(&self, employer_id: &Uuid, user_id: &Uuid) -> Result<()> {
        trace!("db: accept team member invitation");

        let db = self.pool.get().await?;
        db.execute(
            "select accept_team_member_invitation($1::uuid, $2::uuid);",
            &[&employer_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, employer), err)]
    async fn add_employer(&self, user_id: &Uuid, employer: &Employer) -> Result<Uuid> {
        trace!("db: add employer");

        let db = self.pool.get().await?;
        let employer_id = db
            .query_one(
                "select add_employer($1::uuid, $2::jsonb);",
                &[&user_id, &Json(employer)],
            )
            .await?
            .get(0);

        Ok(employer_id)
    }

    #[instrument(skip(self, job), err)]
    async fn add_job(&self, employer_id: &Uuid, job: &Job) -> Result<()> {
        trace!("db: add job");

        let db = self.pool.get().await?;
        db.execute(
            "select add_job($1::uuid, $2::jsonb);",
            &[&employer_id, &Json(job)],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, email), err)]
    async fn add_team_member(&self, employer_id: &Uuid, email: &str) -> Result<Option<Uuid>> {
        trace!("db: add team member");

        let db = self.pool.get().await?;
        let user_id = db
            .query_one(
                "select add_team_member($1::uuid, $2::text) as user_id;",
                &[&employer_id, &email],
            )
            .await?
            .get("user_id");

        Ok(user_id)
    }

    #[instrument(skip(self), err)]
    async fn archive_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: archive job");

        let db = self.pool.get().await?;
        db.execute("select archive_job($1::uuid);", &[&job_id]).await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn delete_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: delete job");

        let db = self.pool.get().await?;
        db.execute("select delete_job($1::uuid);", &[&job_id]).await?;

        Ok(())
    }

    /// Delete team member.
    ///
    /// There must be at least one approved team member left on the team.
    ///
    /// - If the team member is approved, we can only delete it if there is at
    ///   least one other approved team member left on the team.
    ///
    /// - If the team member is not approved, we can delete it directly.
    ///
    #[instrument(skip(self), err)]
    async fn delete_team_member(&self, employer_id: &Uuid, user_id: &Uuid) -> Result<()> {
        trace!("db: delete team member");

        let db = self.pool.get().await?;
        db.execute(
            "select delete_team_member($1::uuid, $2::uuid);",
            &[&employer_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_applications_filters_options(
        &self,
        employer_id: &Uuid,
    ) -> Result<applications::FiltersOptions> {
        trace!("db: get applications filters options");

        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one(
                "select get_applications_filters_options($1::uuid)::text;",
                &[&employer_id],
            )
            .await?
            .get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn get_employer(&self, employer_id: &Uuid) -> Result<Employer> {
        trace!("db: get employer");

        let db = self.pool.get().await?;
        let json_data: Option<String> = db
            .query_one("select get_employer($1::uuid)::text;", &[&employer_id])
            .await?
            .get(0);
        let json_data = json_data.context("employer not found")?;

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn get_job_dashboard(&self, job_id: &Uuid) -> Result<Job> {
        trace!("db: get job dashboard");

        let db = self.pool.get().await?;
        let json_data: Option<String> = db
            .query_one("select get_job_dashboard($1::uuid)::text;", &[&job_id])
            .await?
            .get(0);
        let json_data = json_data.context("job not found or deleted")?;

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn get_job_seeker_user_id(&self, job_seeker_profile_id: &Uuid) -> Result<Option<Uuid>> {
        trace!("db: get job seeker user id");

        let db = self.pool.get().await?;
        let user_id = db
            .query_one(
                "select get_job_seeker_user_id($1::uuid);",
                &[&job_seeker_profile_id],
            )
            .await?
            .get(0);

        Ok(user_id)
    }

    #[instrument(skip(self), err)]
    async fn get_job_stats(&self, job_id: &Uuid) -> Result<JobStats> {
        trace!("db: get job stats");

        // Query database
        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select get_job_stats($1::uuid)::text as stats", &[&job_id])
            .await?
            .get("stats");
        let stats: JobStats = serde_json::from_str(&json_data)?;

        Ok(stats)
    }

    #[instrument(skip(self), err)]
    async fn get_user_invitations_count(&self, user_id: &Uuid) -> Result<usize> {
        trace!("db: get user invitations count");

        let db = self.pool.get().await?;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let count = db
            .query_one("select get_user_invitations_count($1::uuid);", &[&user_id])
            .await?
            .get::<_, i64>(0) as usize;

        Ok(count)
    }

    #[instrument(skip(self), err)]
    async fn list_employer_jobs(&self, employer_id: &Uuid) -> Result<Vec<JobSummary>> {
        trace!("db: list employer jobs");

        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select list_employer_jobs($1::uuid)::text;", &[&employer_id])
            .await?
            .get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn list_employers(&self, user_id: &Uuid) -> Result<Vec<EmployerSummary>> {
        trace!("db: list employers");

        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select list_employers($1::uuid)::text;", &[&user_id])
            .await?
            .get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn list_certifications(&self) -> Result<Vec<Certification>> {
        trace!("db: list certifications");

        let db = self.pool.get().await?;
        let json_data: String = db.query_one("select list_certifications()::text;", &[]).await?.get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn list_foundations(&self) -> Result<Vec<Foundation>> {
        trace!("db: list foundations");

        let db = self.pool.get().await?;
        let json_data: String = db.query_one("select list_foundations()::text;", &[]).await?.get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn list_team_members(&self, employer_id: &Uuid) -> Result<Vec<TeamMember>> {
        trace!("db: list team members");

        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select list_team_members($1::uuid)::text;", &[&employer_id])
            .await?
            .get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn list_user_invitations(&self, user_id: &Uuid) -> Result<Vec<TeamInvitation>> {
        trace!("db: list user invitations");

        let db = self.pool.get().await?;
        let json_data: String = db
            .query_one("select list_user_invitations($1::uuid)::text;", &[&user_id])
            .await?
            .get(0);

        Ok(serde_json::from_str(&json_data)?)
    }

    #[instrument(skip(self), err)]
    async fn publish_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: publish job");

        let db = self.pool.get().await?;

        // Read salary fields needed to refresh normalized yearly values
        let row = db
            .query_one("select * from get_job_salary($1::uuid);", &[&job_id])
            .await?;
        let salary: Option<i64> = row.get("salary");
        let salary_min: Option<i64> = row.get("salary_min");
        let salary_max: Option<i64> = row.get("salary_max");
        let currency: Option<String> = row.get("salary_currency");
        let period: Option<String> = row.get("salary_period");

        // Publish job and persist normalized salary values
        db.execute(
            "
            select publish_job($1::uuid, $2::bigint, $3::bigint, $4::bigint);
            ",
            &[
                &job_id,
                &normalize_salary(salary, currency.as_ref(), period.as_ref()).await,
                &normalize_salary(salary_min.or(salary), currency.as_ref(), period.as_ref()).await,
                &normalize_salary(salary_max.or(salary), currency.as_ref(), period.as_ref()).await,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self))]
    async fn search_applications(
        &self,
        employer_id: &Uuid,
        filters: &applications::Filters,
    ) -> Result<ApplicationsSearchOutput> {
        trace!("db: search applications");

        // Query database
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select search_applications($1::uuid, $2::jsonb)::text",
                &[&employer_id, &Json(filters)],
            )
            .await?;
        let output = serde_json::from_str(&row.get::<_, String>(0))?;

        Ok(output)
    }

    #[instrument(skip(self, employer), err)]
    async fn update_employer(&self, employer_id: &Uuid, employer: &Employer) -> Result<()> {
        trace!("db: update employer");

        let db = self.pool.get().await?;
        db.execute(
            "select update_employer($1::uuid, $2::jsonb);",
            &[&employer_id, &Json(employer)],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, job), err)]
    async fn update_job(&self, job_id: &Uuid, job: &Job) -> Result<()> {
        trace!("db: update job");

        let db = self.pool.get().await?;
        db.execute("select update_job($1::uuid, $2::jsonb);", &[&job_id, &Json(job)])
            .await?;

        Ok(())
    }
}

/// Applications search results.
#[derive(Debug, Clone, Serialize, Deserialize)]
/// Output for applications search.
pub(crate) struct ApplicationsSearchOutput {
    /// List of applications matching the search.
    pub applications: Vec<Application>,
    /// Total number of applications found.
    pub total: usize,
}
