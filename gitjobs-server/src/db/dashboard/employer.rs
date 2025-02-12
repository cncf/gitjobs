//! This module defines some database functionality for the employer dashboard.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    templates::dashboard::employer::{
        employers::{Employer, EmployerSummary},
        jobs::{Job, JobBoard, JobSummary},
    },
    PgDB,
};

/// Trait that defines some database operations used in the employer dashboard.
#[async_trait]
pub(crate) trait DBDashBoardEmployer {
    /// Add employer.
    async fn add_employer(&self, job_board_id: &Uuid, user_id: &Uuid, employer: &Employer) -> Result<Uuid>;

    /// Add job.
    async fn add_job(&self, employer_id: &Uuid, job: &Job) -> Result<()>;

    /// Archive job.
    async fn archive_job(&self, job_id: &Uuid) -> Result<()>;

    /// Delete job.
    async fn delete_job(&self, job_id: &Uuid) -> Result<()>;

    /// Get employer.
    async fn get_employer(&self, employer_id: &Uuid) -> Result<Employer>;

    /// Get job board.
    async fn get_job_board(&self, job_board_id: &Uuid) -> Result<JobBoard>;

    /// Get job.
    async fn get_job(&self, job_id: &Uuid) -> Result<Job>;

    /// List employer jobs.
    async fn list_employer_jobs(&self, employer_id: &Uuid) -> Result<Vec<JobSummary>>;

    /// List employers where the user is part of the team.
    async fn list_employers(&self, user_id: &Uuid) -> Result<Vec<EmployerSummary>>;

    /// Publish job.
    async fn publish_job(&self, job_id: &Uuid) -> Result<()>;

    /// Update employer.
    async fn update_employer(&self, employer_id: &Uuid, employer: &Employer) -> Result<()>;

    /// Update job.
    async fn update_job(&self, job_id: &Uuid, job: &Job) -> Result<()>;
}

#[async_trait]
impl DBDashBoardEmployer for PgDB {
    /// [DBDashBoardEmployer::add_employer]
    #[instrument(skip(self), err)]
    async fn add_employer(&self, job_board_id: &Uuid, user_id: &Uuid, employer: &Employer) -> Result<Uuid> {
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Insert employer
        let employer_id: Uuid = tx
            .query_one(
                "
                insert into employer (
                    job_board_id,
                    company,
                    description,
                    public,
                    location_id,
                    logo_id,
                    website_url
                ) values (
                    $1::uuid,
                    $2::text,
                    $3::text,
                    $4::bool,
                    $5::uuid,
                    $6::uuid,
                    $7::text
                ) returning employer_id;
                ",
                &[
                    &job_board_id,
                    &employer.company,
                    &employer.description,
                    &employer.public,
                    &employer.location_id,
                    &employer.logo_id,
                    &employer.website_url,
                ],
            )
            .await?
            .get("employer_id");

        // Add user to employer team
        tx.execute(
            "
            insert into employer_team (
                employer_id,
                user_id
            ) values (
                $1::uuid,
                $2::uuid
            );
            ",
            &[&employer_id, &user_id],
        )
        .await?;

        tx.commit().await?;

        Ok(employer_id)
    }

    /// [DBDashBoardEmployer::add_job]
    #[instrument(skip(self), err)]
    async fn add_job(&self, employer_id: &Uuid, job: &Job) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            insert into job (
                employer_id,
                type,
                status,
                location_id,
                workplace,
                title,
                description,
                apply_instructions,
                apply_url,
                benefits,
                open_source,
                salary,
                salary_currency,
                salary_min,
                salary_max,
                salary_period,
                skills,
                upstream_commitment,
                published_at
            )
            select
                $1::uuid,
                $2::text,
                $3::text,
                $4::uuid,
                $5::text,
                $6::text,
                $7::text,
                $8::text,
                $9::text,
                $10::text[],
                $11::int,
                $12::bigint,
                $13::text,
                $14::bigint,
                $15::bigint,
                $16::text,
                $17::text[],
                $18::int,
                case when $3::text = 'published' then current_timestamp else null end;
            ",
            &[
                &employer_id,
                &job.type_.to_string(),
                &job.status.to_string(),
                &job.location_id,
                &job.workplace.to_string(),
                &job.title,
                &job.description,
                &job.apply_instructions,
                &job.apply_url,
                &job.benefits,
                &job.open_source,
                &job.salary,
                &job.salary_currency,
                &job.salary_min,
                &job.salary_max,
                &job.salary_period,
                &job.skills,
                &job.upstream_commitment,
            ],
        )
        .await?;

        Ok(())
    }

    /// [DBDashBoardEmployer::archive_job]
    #[instrument(skip(self), err)]
    async fn archive_job(&self, job_id: &Uuid) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
                    update job
                    set
                        status = 'archived',
                        archived_at = current_timestamp,
                        updated_at = current_timestamp
                    where job_id = $1::uuid
                    and status = 'published';
                    ",
            &[&job_id],
        )
        .await?;

        Ok(())
    }

    /// [DBDashBoardEmployer::delete_job]
    #[instrument(skip(self), err)]
    async fn delete_job(&self, job_id: &Uuid) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute("delete from job where job_id = $1::uuid;", &[&job_id])
            .await?;

        Ok(())
    }

    /// [DBDashBoardEmployer::get_employer]
    #[instrument(skip(self), err)]
    async fn get_employer(&self, employer_id: &Uuid) -> Result<Employer> {
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select
                    e.company,
                    e.description,
                    e.public,
                    e.location_id,
                    e.logo_id,
                    e.website_url,
                    l.city,
                    l.country,
                    l.state
                from employer e
                left join location l using (location_id)
                where employer_id = $1::uuid;
                ",
                &[&employer_id],
            )
            .await?;
        let employer = Employer {
            company: row.get("company"),
            description: row.get("description"),
            public: row.get("public"),
            city: row.get("city"),
            country: row.get("country"),
            location_id: row.get("location_id"),
            logo_id: row.get("logo_id"),
            state: row.get("state"),
            website_url: row.get("website_url"),
        };

        Ok(employer)
    }

    /// [DBDashBoardEmployer::get_job_board]
    #[instrument(skip(self), err)]
    async fn get_job_board(&self, job_board_id: &Uuid) -> Result<JobBoard> {
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select
                    benefits,
                    skills
                from job_board
                where job_board_id = $1::uuid;
                ",
                &[&job_board_id],
            )
            .await?;
        let job_board = JobBoard {
            benefits: row.get("benefits"),
            skills: row.get("skills"),
        };

        Ok(job_board)
    }

    /// [DBDashBoardEmployer::get_job]
    #[instrument(skip(self), err)]
    async fn get_job(&self, job_id: &Uuid) -> Result<Job> {
        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select
                    j.description,
                    j.status,
                    j.title,
                    j.type,
                    j.workplace,
                    j.apply_instructions,
                    j.apply_url,
                    j.benefits,
                    j.job_id,
                    j.location_id,
                    j.open_source,
                    j.salary,
                    j.salary_currency,
                    j.salary_min,
                    j.salary_max,
                    j.salary_period,
                    j.skills,
                    j.upstream_commitment,
                    l.city,
                    l.country,
                    l.state
                from job j
                left join location l using (location_id)
                where job_id = $1::uuid;
                ",
                &[&job_id],
            )
            .await?;
        let job = Job {
            description: row.get("description"),
            status: row.get::<_, String>("status").parse().expect("valid job status"),
            title: row.get("title"),
            type_: row.get::<_, String>("type").parse().expect("valid job type"),
            workplace: row.get::<_, String>("workplace").parse().expect("valid workplace"),
            apply_instructions: row.get("apply_instructions"),
            apply_url: row.get("apply_url"),
            benefits: row.get("benefits"),
            city: row.get("city"),
            country: row.get("country"),
            job_id: row.get("job_id"),
            location_id: row.get("location_id"),
            open_source: row.get("open_source"),
            salary: row.get("salary"),
            salary_currency: row.get("salary_currency"),
            salary_min: row.get("salary_min"),
            salary_max: row.get("salary_max"),
            salary_period: row.get("salary_period"),
            skills: row.get("skills"),
            state: row.get("state"),
            upstream_commitment: row.get("upstream_commitment"),
        };

        Ok(job)
    }

    /// [DBDashBoardEmployer::list_employer_jobs]
    #[instrument(skip(self), err)]
    async fn list_employer_jobs(&self, employer_id: &Uuid) -> Result<Vec<JobSummary>> {
        let db = self.pool.get().await?;
        let jobs = db
            .query(
                "
                select
                    j.job_id,
                    j.created_at,
                    j.title,
                    j.status,
                    j.archived_at,
                    j.published_at,
                    l.city,
                    l.country
                from job j
                left join location l using (location_id)
                where employer_id = $1::uuid
                order by published_at desc, created_at desc;
                ",
                &[&employer_id],
            )
            .await?
            .into_iter()
            .map(|row| JobSummary {
                created_at: row.get("created_at"),
                job_id: row.get("job_id"),
                title: row.get("title"),
                status: row.get::<_, String>("status").parse().expect("valid job status"),
                city: row.get("city"),
                country: row.get("country"),
                archived_at: row.get("archived_at"),
                published_at: row.get("published_at"),
            })
            .collect();

        Ok(jobs)
    }

    /// [DBAuth::list_employers]
    #[instrument(skip(self), err)]
    async fn list_employers(&self, user_id: &Uuid) -> Result<Vec<EmployerSummary>> {
        let db = self.pool.get().await?;
        let employers = db
            .query(
                "
                select employer_id, company, logo_id
                from employer e
                join employer_team et using (employer_id)
                where et.user_id = $1::uuid;
                ",
                &[&user_id],
            )
            .await?
            .into_iter()
            .map(|row| EmployerSummary {
                employer_id: row.get("employer_id"),
                company: row.get("company"),
                logo_id: row.get("logo_id"),
            })
            .collect();

        Ok(employers)
    }

    /// [DBDashBoardEmployer::publish_job]
    #[instrument(skip(self), err)]
    async fn publish_job(&self, job_id: &Uuid) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'published',
                published_at = current_timestamp,
                updated_at = current_timestamp,
                archived_at = null
            where job_id = $1::uuid
            and status <> 'published';
            ",
            &[&job_id],
        )
        .await?;

        Ok(())
    }

    /// [DBDashBoardEmployer::update_employer]
    #[instrument(skip(self), err)]
    async fn update_employer(&self, employer_id: &Uuid, employer: &Employer) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            update employer
            set
                company = $2::text,
                description = $3::text,
                public = $4::bool,
                location_id = $5::uuid,
                logo_id = $6::uuid,
                website_url = $7::text,
                updated_at = current_timestamp
            where employer_id = $1::uuid;
            ",
            &[
                &employer_id,
                &employer.company,
                &employer.description,
                &employer.public,
                &employer.location_id,
                &employer.logo_id,
                &employer.website_url,
            ],
        )
        .await?;

        Ok(())
    }

    /// [DBDashBoardEmployer::update_job]
    #[instrument(skip(self), err)]
    async fn update_job(&self, job_id: &Uuid, job: &Job) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                type = $2::text,
                status = $3::text,
                location_id = $4::uuid,
                workplace = $5::text,
                title = $6::text,
                description = $7::text,
                apply_instructions = $8::text,
                apply_url = $9::text,
                benefits = $10::text[],
                open_source = $11::int,
                salary = $12::bigint,
                salary_currency = $13::text,
                salary_min = $14::bigint,
                salary_max = $15::bigint,
                salary_period = $16::text,
                skills = $17::text[],
                upstream_commitment = $18::int,
                updated_at = current_timestamp
            where job_id = $1::uuid;
            ",
            &[
                &job_id,
                &job.type_.to_string(),
                &job.status.to_string(),
                &job.location_id,
                &job.workplace.to_string(),
                &job.title,
                &job.description,
                &job.apply_instructions,
                &job.apply_url,
                &job.benefits,
                &job.open_source,
                &job.salary,
                &job.salary_currency,
                &job.salary_min,
                &job.salary_max,
                &job.salary_period,
                &job.skills,
                &job.upstream_commitment,
            ],
        )
        .await?;

        Ok(())
    }
}
