//! This module defines database operations for the employer dashboard.

use anyhow::{Result, bail};
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
            "
            update employer_team
            set approved = true
            where employer_id = $1::uuid
            and user_id = $2::uuid;
            ",
            &[&employer_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, employer), err)]
    async fn add_employer(&self, user_id: &Uuid, employer: &Employer) -> Result<Uuid> {
        trace!("db: add employer");

        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Insert employer
        let employer_id: Uuid = tx
            .query_one(
                "
                insert into employer (
                    company,
                    description,
                    public,
                    location_id,
                    logo_id,
                    member_id,
                    website_url
                ) values (
                    $1::text,
                    $2::text,
                    $3::bool,
                    $4::uuid,
                    $5::uuid,
                    $6::uuid,
                    $7::text
                ) returning employer_id;
                ",
                &[
                    &employer.company,
                    &employer.description,
                    &employer.public,
                    &employer.location.as_ref().map(|l| l.location_id),
                    &employer.logo_id,
                    &employer.member.as_ref().map(|m| m.member_id),
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
                user_id,
                approved
            ) values (
                $1::uuid,
                $2::uuid,
                true
            );
            ",
            &[&employer_id, &user_id],
        )
        .await?;

        // Commit transaction
        tx.commit().await?;

        Ok(employer_id)
    }

    #[instrument(skip(self, job), err)]
    async fn add_job(&self, employer_id: &Uuid, job: &Job) -> Result<()> {
        trace!("db: add job");

        // Begin transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Insert job
        let row = tx
            .query_one(
                "
                insert into job (
                    employer_id,
                    kind,
                    status,
                    location_id,
                    workplace,
                    title,
                    description,
                    apply_instructions,
                    apply_url,
                    benefits,
                    open_source,
                    qualifications,
                    responsibilities,
                    salary,
                    salary_usd_year,
                    salary_currency,
                    salary_min,
                    salary_min_usd_year,
                    salary_max,
                    salary_max_usd_year,
                    salary_period,
                    seniority,
                    skills,
                    tz_end,
                    tz_start,
                    upstream_commitment
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
                    $12::text,
                    $13::text,
                    $14::bigint,
                    $15::bigint,
                    $16::text,
                    $17::bigint,
                    $18::bigint,
                    $19::bigint,
                    $20::bigint,
                    $21::text,
                    $22::text,
                    $23::text[],
                    $24::text,
                    $25::text,
                    $26::int
                returning job_id;
                ",
                &[
                    &employer_id,
                    &job.kind.to_string(),
                    &job.status.to_string(),
                    &job.location.as_ref().map(|l| l.location_id),
                    &job.workplace.to_string(),
                    &job.title,
                    &job.description,
                    &job.apply_instructions,
                    &job.apply_url,
                    &job.benefits,
                    &job.open_source,
                    &job.qualifications,
                    &job.responsibilities,
                    &job.salary,
                    &job.salary_usd_year,
                    &job.salary_currency,
                    &job.salary_min,
                    &job.salary_min_usd_year,
                    &job.salary_max,
                    &job.salary_max_usd_year,
                    &job.salary_period,
                    &job.seniority.as_ref().map(ToString::to_string),
                    &job.skills,
                    &job.tz_end,
                    &job.tz_start,
                    &job.upstream_commitment,
                ],
            )
            .await?;
        let job_id: Uuid = row.get("job_id");

        // Insert job projects
        if let Some(projects) = &job.projects {
            for project in projects {
                tx.execute(
                    "
                    insert into job_project (job_id, project_id)
                    values ($1::uuid, $2::uuid);
                    ",
                    &[&job_id, &project.project_id],
                )
                .await?;
            }
        }

        // Insert job certifications
        if let Some(certifications) = &job.certifications {
            for certification in certifications {
                tx.execute(
                    "
                    insert into job_certification (job_id, certification_id)
                    values ($1::uuid, $2::uuid);
                    ",
                    &[&job_id, &certification.certification_id],
                )
                .await?;
            }
        }

        // Commit transaction
        tx.commit().await?;

        Ok(())
    }

    #[instrument(skip(self, email), err)]
    async fn add_team_member(&self, employer_id: &Uuid, email: &str) -> Result<Option<Uuid>> {
        trace!("db: add team member");

        let db = self.pool.get().await?;
        let user_id = db
            .query_opt(
                r#"
                insert into employer_team (
                    employer_id,
                    user_id,
                    approved
                )
                select
                    $1::uuid,
                    user_id,
                    false
                from "user"
                where email = $2::text
                on conflict do nothing
                returning user_id;
                "#,
                &[&employer_id, &email],
            )
            .await?
            .map(|row| row.get("user_id"));

        Ok(user_id)
    }

    #[instrument(skip(self), err)]
    async fn archive_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: archive job");

        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'archived',
                archived_at = current_timestamp,
                updated_at = current_timestamp
            where job_id = $1::uuid
            and (status = 'pending-approval' or status = 'published');
            ",
            &[&job_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn delete_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: delete job");

        let db = self.pool.get().await?;
        db.execute(
            "
            update job
            set
                status = 'deleted',
                deleted_at = current_timestamp
            where job_id = $1::uuid;
            ",
            &[&job_id],
        )
        .await?;

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

        let approved: bool = db
            .query_one(
                "
                select approved
                from employer_team
                where employer_id = $1::uuid
                and user_id = $2::uuid;
                ",
                &[&employer_id, &user_id],
            )
            .await?
            .get("approved");

        if approved {
            let n = db
                .execute(
                    "
                    delete from employer_team
                    where employer_id = $1::uuid
                    and user_id = $2::uuid
                    and
                        (
                            select count(*)
                            from employer_team
                            where employer_id = $1::uuid
                            and approved = true
                        ) > 1;
                    ",
                    &[&employer_id, &user_id],
                )
                .await?;
            if n == 0 {
                bail!("cannot delete last approved team member");
            }
        } else {
            db.execute(
                "
                delete from employer_team
                where employer_id = $1::uuid
                and user_id = $2::uuid;
                ",
                &[&employer_id, &user_id],
            )
            .await?;
        }

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_applications_filters_options(
        &self,
        employer_id: &Uuid,
    ) -> Result<applications::FiltersOptions> {
        trace!("db: get applications filters options");

        // Query database
        let db = self.pool.get().await?;
        let rows = db
            .query(
                "
                select
                    j.job_id,
                    j.created_at,
                    j.title,
                    j.status,
                    l.city,
                    l.country
                from job j
                left join location l using (location_id)
                where employer_id = $1::uuid
                and j.status <> 'deleted'
                order by created_at desc;
                ",
                &[&employer_id],
            )
            .await?;

        // Prepare filters options
        let mut jobs = Vec::new();
        for row in rows {
            let job = JobSummary {
                job_id: row.get("job_id"),
                city: row.get("city"),
                country: row.get("country"),
                created_at: row.get("created_at"),
                title: row.get("title"),
                status: row.get::<_, String>("status").parse().expect("valid job status"),
                ..Default::default()
            };
            jobs.push(job);
        }
        let filters_options = applications::FiltersOptions { jobs };

        Ok(filters_options)
    }

    #[instrument(skip(self), err)]
    async fn get_employer(&self, employer_id: &Uuid) -> Result<Employer> {
        trace!("db: get employer");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select
                    e.company,
                    e.description,
                    e.public,
                    e.logo_id,
                    e.website_url,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'location_id', l.location_id,
                            'city', l.city,
                            'country', l.country,
                            'state', l.state
                        )), '{}'::jsonb)
                    ) as location,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'member_id', m.member_id,
                            'foundation', m.foundation,
                            'level', m.level,
                            'logo_url', m.logo_url,
                            'name', m.name
                        )), '{}'::jsonb)
                    ) as member
                from employer e
                left join location l using (location_id)
                left join member m using (member_id)
                where employer_id = $1::uuid;
                ",
                &[&employer_id],
            )
            .await?;
        let employer = Employer {
            company: row.get("company"),
            description: row.get("description"),
            public: row.get("public"),
            location: row
                .get::<_, Option<serde_json::Value>>("location")
                .map(|v| serde_json::from_value(v).expect("location should be valid json")),
            logo_id: row.get("logo_id"),
            member: row
                .get::<_, Option<serde_json::Value>>("member")
                .map(|v| serde_json::from_value(v).expect("member should be valid json")),
            website_url: row.get("website_url"),
        };

        Ok(employer)
    }

    #[instrument(skip(self), err)]
    async fn get_job_dashboard(&self, job_id: &Uuid) -> Result<Job> {
        trace!("db: get job dashboard");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select
                    j.description,
                    j.status,
                    j.title,
                    j.kind,
                    j.workplace,
                    j.apply_instructions,
                    j.apply_url,
                    j.benefits,
                    j.job_id,
                    j.location_id,
                    j.open_source,
                    j.published_at,
                    j.qualifications,
                    j.responsibilities,
                    j.review_notes,
                    j.salary,
                    j.salary_currency,
                    j.salary_min,
                    j.salary_max,
                    j.salary_period,
                    j.seniority,
                    j.skills,
                    j.tz_end,
                    j.tz_start,
                    j.updated_at,
                    j.upstream_commitment,
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
                    ) as projects,
                    (
                        select json_agg(json_build_object(
                            'certification_id', c.certification_id,
                            'name', c.name,
                            'provider', c.provider,
                            'short_name', c.short_name,
                            'description', c.description,
                            'url', c.url,
                            'logo_url', c.logo_url
                        ))
                        from certification c
                        left join job_certification jc using (certification_id)
                        left join job j using (job_id)
                        where j.job_id = $1::uuid
                    ) as certifications
                from job j
                left join location l using (location_id)
                where job_id = $1::uuid
                and status <> 'deleted';
                ",
                &[&job_id],
            )
            .await?;

        let job = Job {
            description: row.get("description"),
            status: row.get::<_, String>("status").parse().expect("valid job status"),
            title: row.get("title"),
            kind: row.get::<_, String>("kind").parse().expect("valid job kind"),
            workplace: row.get::<_, String>("workplace").parse().expect("valid workplace"),
            apply_instructions: row.get("apply_instructions"),
            apply_url: row.get("apply_url"),
            benefits: row.get("benefits"),
            certifications: row
                .get::<_, Option<serde_json::Value>>("certifications")
                .map(|v| serde_json::from_value(v).expect("certifications should be valid json")),
            job_id: row.get("job_id"),
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
            review_notes: row.get("review_notes"),
            salary: row.get("salary"),
            salary_usd_year: None,
            salary_currency: row.get("salary_currency"),
            salary_min: row.get("salary_min"),
            salary_min_usd_year: None,
            salary_max: row.get("salary_max"),
            salary_max_usd_year: None,
            salary_period: row.get("salary_period"),
            seniority: row
                .get::<_, Option<String>>("seniority")
                .map(|s| s.parse().expect("valid seniority")),
            skills: row.get("skills"),
            tz_end: row.get("tz_end"),
            tz_start: row.get("tz_start"),
            updated_at: row.get("updated_at"),
            upstream_commitment: row.get("upstream_commitment"),
        };

        Ok(job)
    }

    #[instrument(skip(self), err)]
    async fn get_job_seeker_user_id(&self, job_seeker_profile_id: &Uuid) -> Result<Option<Uuid>> {
        trace!("db: get job seeker user id");

        let db = self.pool.get().await?;
        let user_id = db
            .query_opt(
                "
                select user_id
                from job_seeker_profile
                where job_seeker_profile_id = $1::uuid;
                ",
                &[&job_seeker_profile_id],
            )
            .await?
            .map(|row| row.get("user_id"));

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
            .query_one(
                "
                select count(*) as count
                from employer_team et
                where et.user_id = $1::uuid
                and et.approved = false;
                ",
                &[&user_id],
            )
            .await?
            .get::<_, i64>("count") as usize;

        Ok(count)
    }

    #[instrument(skip(self), err)]
    async fn list_employer_jobs(&self, employer_id: &Uuid) -> Result<Vec<JobSummary>> {
        trace!("db: list employer jobs");

        let db = self.pool.get().await?;
        let jobs = db
            .query(
                "
                select
                    j.job_id,
                    j.created_at,
                    j.title,
                    j.status,
                    j.workplace,
                    j.archived_at,
                    j.published_at,
                    j.review_notes,
                    l.city,
                    l.country
                from job j
                left join location l using (location_id)
                where employer_id = $1::uuid
                and j.status <> 'deleted'
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
                workplace: row.get::<_, String>("workplace").parse().expect("valid workplace"),
                city: row.get("city"),
                country: row.get("country"),
                archived_at: row.get("archived_at"),
                published_at: row.get("published_at"),
                review_notes: row.get("review_notes"),
            })
            .collect();

        Ok(jobs)
    }

    #[instrument(skip(self), err)]
    async fn list_employers(&self, user_id: &Uuid) -> Result<Vec<EmployerSummary>> {
        trace!("db: list employers");

        let db = self.pool.get().await?;
        let employers = db
            .query(
                "
                select employer_id, company, logo_id
                from employer e
                join employer_team et using (employer_id)
                where et.user_id = $1::uuid
                and et.approved = true
                order by company asc;
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

    #[instrument(skip(self), err)]
    async fn list_certifications(&self) -> Result<Vec<Certification>> {
        trace!("db: list certifications");

        let db = self.pool.get().await?;
        let certifications = db
            .query(
                "
                select
                    certification_id,
                    name,
                    provider,
                    short_name,
                    description,
                    logo_url,
                    url
                from certification
                order by name asc;
                ",
                &[],
            )
            .await?
            .into_iter()
            .map(|row| Certification {
                certification_id: row.get("certification_id"),
                name: row.get("name"),
                provider: row.get("provider"),
                short_name: row.get("short_name"),
                description: row.get("description"),
                logo_url: row.get("logo_url"),
                url: row.get("url"),
            })
            .collect();

        Ok(certifications)
    }

    #[instrument(skip(self), err)]
    async fn list_foundations(&self) -> Result<Vec<Foundation>> {
        trace!("db: list foundations");

        let db = self.pool.get().await?;
        let foundations = db
            .query("select name from foundation order by name asc;", &[])
            .await?
            .into_iter()
            .map(|row| Foundation {
                name: row.get("name"),
            })
            .collect();

        Ok(foundations)
    }

    #[instrument(skip(self), err)]
    async fn list_team_members(&self, employer_id: &Uuid) -> Result<Vec<TeamMember>> {
        trace!("db: list team members");

        let db = self.pool.get().await?;
        let members = db
            .query(
                r#"
                select
                    et.approved,
                    u.email,
                    u.name,
                    u.user_id,
                    u.username
                from employer_team et
                join "user" u using (user_id)
                where employer_id = $1::uuid
                order by name asc;
                "#,
                &[&employer_id],
            )
            .await?
            .into_iter()
            .map(|row| TeamMember {
                approved: row.get("approved"),
                email: row.get("email"),
                name: row.get("name"),
                user_id: row.get("user_id"),
                username: row.get("username"),
            })
            .collect();

        Ok(members)
    }

    #[instrument(skip(self), err)]
    async fn list_user_invitations(&self, user_id: &Uuid) -> Result<Vec<TeamInvitation>> {
        trace!("db: list user invitations");

        let db = self.pool.get().await?;
        let invitations = db
            .query(
                "
                select
                    e.company,
                    e.created_at,
                    e.employer_id
                from employer_team et
                join employer e using (employer_id)
                where et.user_id = $1::uuid
                and et.approved = false
                order by created_at desc;
                ",
                &[&user_id],
            )
            .await?
            .into_iter()
            .map(|row| TeamInvitation {
                company: row.get("company"),
                created_at: row.get("created_at"),
                employer_id: row.get("employer_id"),
            })
            .collect();

        Ok(invitations)
    }

    #[instrument(skip(self), err)]
    async fn publish_job(&self, job_id: &Uuid) -> Result<()> {
        trace!("db: publish job");

        let db = self.pool.get().await?;

        // Get current job salary details to refresh the usd/year versions
        let row = db
            .query_one(
                "
                select
                    salary,
                    salary_currency,
                    salary_min,
                    salary_max,
                    salary_period
                from job
                where job_id = $1::uuid;
                ",
                &[&job_id],
            )
            .await?;
        let salary: Option<i64> = row.get("salary");
        let salary_min: Option<i64> = row.get("salary_min");
        let salary_max: Option<i64> = row.get("salary_max");
        let currency: Option<String> = row.get("salary_currency");
        let period: Option<String> = row.get("salary_period");

        // Update job
        db.execute(
            "
            update job
            set
                status = 'pending-approval',
                updated_at = current_timestamp,
                archived_at = null,
                salary_usd_year = $2::bigint,
                salary_min_usd_year = $3::bigint,
                salary_max_usd_year = $4::bigint
            where job_id = $1::uuid
            and (status = 'archived' or status = 'draft' or status = 'rejected');
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
                "select applications::text, total from search_applications($1::uuid, $2::jsonb)",
                &[&employer_id, &Json(filters)],
            )
            .await?;

        // Prepare search output
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let output = ApplicationsSearchOutput {
            applications: serde_json::from_str(&row.get::<_, String>("applications"))?,
            total: row.get::<_, i64>("total") as usize,
        };

        Ok(output)
    }

    #[instrument(skip(self, employer), err)]
    async fn update_employer(&self, employer_id: &Uuid, employer: &Employer) -> Result<()> {
        trace!("db: update employer");

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
                member_id = $7::uuid,
                website_url = $8::text,
                updated_at = current_timestamp
            where employer_id = $1::uuid;
            ",
            &[
                &employer_id,
                &employer.company,
                &employer.description,
                &employer.public,
                &employer.location.as_ref().map(|l| l.location_id),
                &employer.logo_id,
                &employer.member.as_ref().map(|m| m.member_id),
                &employer.website_url,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, job), err)]
    async fn update_job(&self, job_id: &Uuid, job: &Job) -> Result<()> {
        trace!("db: update job");

        // Begin transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Update job
        let rows_updated = tx
            .execute(
                "
                update job
                set
                    kind = $2::text,
                    status = $3::text,
                    location_id = $4::uuid,
                    workplace = $5::text,
                    title = $6::text,
                    description = $7::text,
                    apply_instructions = $8::text,
                    apply_url = $9::text,
                    benefits = $10::text[],
                    open_source = $11::int,
                    qualifications = $12::text,
                    responsibilities = $13::text,
                    salary = $14::bigint,
                    salary_usd_year = $15::bigint,
                    salary_currency = $16::text,
                    salary_min = $17::bigint,
                    salary_min_usd_year = $18::bigint,
                    salary_max = $19::bigint,
                    salary_max_usd_year = $20::bigint,
                    salary_period = $21::text,
                    seniority = $22::text,
                    skills = $23::text[],
                    tz_end = $24::text,
                    tz_start = $25::text,
                    upstream_commitment = $26::int,
                    updated_at = current_timestamp
                where job_id = $1::uuid
                and status <> 'deleted';
                ",
                &[
                    &job_id,
                    &job.kind.to_string(),
                    &job.status.to_string(),
                    &job.location.as_ref().map(|l| l.location_id),
                    &job.workplace.to_string(),
                    &job.title,
                    &job.description,
                    &job.apply_instructions,
                    &job.apply_url,
                    &job.benefits,
                    &job.open_source,
                    &job.qualifications,
                    &job.responsibilities,
                    &job.salary,
                    &job.salary_usd_year,
                    &job.salary_currency,
                    &job.salary_min,
                    &job.salary_min_usd_year,
                    &job.salary_max,
                    &job.salary_max_usd_year,
                    &job.salary_period,
                    &job.seniority.as_ref().map(ToString::to_string),
                    &job.skills,
                    &job.tz_end,
                    &job.tz_start,
                    &job.upstream_commitment,
                ],
            )
            .await?;

        if rows_updated == 1 {
            // Update job projects
            tx.execute("delete from job_project where job_id = $1::uuid;", &[&job_id])
                .await?;
            if let Some(projects) = &job.projects {
                for project in projects {
                    tx.execute(
                        "
                        insert into job_project (job_id, project_id)
                        values ($1::uuid, $2::uuid);
                        ",
                        &[&job_id, &project.project_id],
                    )
                    .await?;
                }
            }

            // Update job certifications
            tx.execute(
                "delete from job_certification where job_id = $1::uuid;",
                &[&job_id],
            )
            .await?;
            if let Some(certifications) = &job.certifications {
                for certification in certifications {
                    tx.execute(
                        "
                        insert into job_certification (job_id, certification_id)
                        values ($1::uuid, $2::uuid);
                        ",
                        &[&job_id, &certification.certification_id],
                    )
                    .await?;
                }
            }
        }

        // Commit transaction
        tx.commit().await?;

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
