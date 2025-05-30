//! This module defines database operations for the job seeker dashboard.

use anyhow::Result;
use async_trait::async_trait;
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
            "
            delete from application
            where application_id in (
                select application_id
                from application a
                join job_seeker_profile p using (job_seeker_profile_id)
                where application_id = $1::uuid
                and user_id = $2::uuid
            );
            ",
            &[&application_id, &user_id],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_job_seeker_profile(&self, user_id: &Uuid) -> Result<Option<JobSeekerProfile>> {
        trace!("db: get job seeker profile");

        let db = self.pool.get().await?;
        let profile = db
            .query_opt(
                "
                select
                    p.email,
                    p.name,
                    p.public,
                    p.summary,
                    p.bluesky_url,
                    p.certifications,
                    p.education,
                    p.experience,
                    p.facebook_url,
                    p.github_url,
                    p.linkedin_url,
                    p.location_id,
                    p.open_to_relocation,
                    p.open_to_remote,
                    p.phone,
                    p.photo_id,
                    p.projects,
                    p.skills,
                    p.twitter_url,
                    p.website_url,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'location_id', l.location_id,
                            'city', l.city,
                            'country', l.country,
                            'state', l.state
                        )), '{}'::jsonb)
                    ) as location
                from job_seeker_profile p
                left join location l using (location_id)
                where user_id = $1::uuid;
                ",
                &[&user_id],
            )
            .await?
            .map(|row| JobSeekerProfile {
                email: row.get("email"),
                name: row.get("name"),
                public: row.get("public"),
                summary: row.get("summary"),
                bluesky_url: row.get("bluesky_url"),
                certifications: row
                    .get::<_, Option<serde_json::Value>>("certifications")
                    .map(|v| serde_json::from_value(v).expect("certifications should be valid json")),
                education: row
                    .get::<_, Option<serde_json::Value>>("education")
                    .map(|v| serde_json::from_value(v).expect("education should be valid json")),
                experience: row
                    .get::<_, Option<serde_json::Value>>("experience")
                    .map(|v| serde_json::from_value(v).expect("experience should be valid json")),
                facebook_url: row.get("facebook_url"),
                github_url: row.get("github_url"),
                linkedin_url: row.get("linkedin_url"),
                location: row
                    .get::<_, Option<serde_json::Value>>("location")
                    .map(|v| serde_json::from_value(v).expect("location should be valid json")),
                open_to_relocation: row.get("open_to_relocation"),
                open_to_remote: row.get("open_to_remote"),
                phone: row.get("phone"),
                photo_id: row.get("photo_id"),
                projects: row
                    .get::<_, Option<serde_json::Value>>("projects")
                    .map(|v| serde_json::from_value(v).expect("projects should be valid json")),
                skills: row.get("skills"),
                twitter_url: row.get("twitter_url"),
                website_url: row.get("website_url"),
            });

        Ok(profile)
    }

    #[instrument(skip(self), err)]
    async fn list_job_seeker_applications(&self, user_id: &Uuid) -> Result<Vec<Application>> {
        trace!("db: list job seeker applications");

        let db = self.pool.get().await?;
        let applications = db
            .query(
                "
                select
                    a.application_id,
                    a.created_at as applied_at,
                    a.job_id,
                    j.title as job_title,
                    (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'location_id', l.location_id,
                            'city', l.city,
                            'country', l.country,
                            'state', l.state
                        )), '{}'::jsonb)
                    ) as job_location,
                    j.status as job_status,
                    j.workplace as job_workplace
                from application a
                join job j on a.job_id = j.job_id
                join job_seeker_profile p on a.job_seeker_profile_id = p.job_seeker_profile_id
                left join location l on j.location_id = l.location_id
                where p.user_id = $1::uuid
                order by applied_at desc;
                ",
                &[&user_id],
            )
            .await?
            .iter()
            .map(|row| Application {
                application_id: row.get("application_id"),
                applied_at: row.get("applied_at"),
                job_id: row.get("job_id"),
                job_location: row
                    .get::<_, Option<serde_json::Value>>("job_location")
                    .map(|v| serde_json::from_value(v).expect("job location should be valid json")),
                job_status: row.get::<_, String>("job_status").parse().expect("valid job status"),
                job_title: row.get("job_title"),
                job_workplace: row
                    .get::<_, String>("job_workplace")
                    .parse()
                    .expect("valid job workplace"),
            })
            .collect();

        Ok(applications)
    }

    #[instrument(skip(self), err)]
    async fn update_job_seeker_profile(&self, user_id: &Uuid, profile: &JobSeekerProfile) -> Result<()> {
        trace!("db: update job seeker profile");

        let db = self.pool.get().await?;
        db.execute(
            "
            insert into job_seeker_profile (
                user_id,
                email,
                name,
                public,
                summary,
                bluesky_url,
                certifications,
                education,
                experience,
                facebook_url,
                github_url,
                linkedin_url,
                location_id,
                open_to_relocation,
                open_to_remote,
                phone,
                photo_id,
                projects,
                skills,
                twitter_url,
                website_url
            ) values (
                $1::uuid,
                $2::text,
                $3::text,
                $4::boolean,
                $5::text,
                $6::text,
                nullif($7::jsonb, 'null'::jsonb),
                nullif($8::jsonb, 'null'::jsonb),
                nullif($9::jsonb, 'null'::jsonb),
                $10::text,
                $11::text,
                $12::text,
                $13::uuid,
                $14::boolean,
                $15::boolean,
                $16::text,
                $17::uuid,
                nullif($18::jsonb, 'null'::jsonb),
                $19::text[],
                $20::text,
                $21::text
            )
            on conflict (user_id) do update set
                email = excluded.email,
                name = excluded.name,
                public = excluded.public,
                summary = excluded.summary,
                bluesky_url = excluded.bluesky_url,
                certifications = excluded.certifications,
                education = excluded.education,
                experience = excluded.experience,
                facebook_url = excluded.facebook_url,
                github_url = excluded.github_url,
                linkedin_url = excluded.linkedin_url,
                location_id = excluded.location_id,
                open_to_relocation = excluded.open_to_relocation,
                open_to_remote = excluded.open_to_remote,
                phone = excluded.phone,
                photo_id = excluded.photo_id,
                projects = excluded.projects,
                skills = excluded.skills,
                twitter_url = excluded.twitter_url,
                website_url = excluded.website_url;
            ",
            &[
                &user_id,
                &profile.email,
                &profile.name,
                &profile.public,
                &profile.summary,
                &profile.bluesky_url,
                &serde_json::to_value(&profile.certifications).expect("certifications should be valid json"),
                &serde_json::to_value(&profile.education).expect("education should be valid json"),
                &serde_json::to_value(&profile.experience).expect("experience should be valid json"),
                &profile.facebook_url,
                &profile.github_url,
                &profile.linkedin_url,
                &profile.location.as_ref().map(|l| l.location_id),
                &profile.open_to_relocation,
                &profile.open_to_remote,
                &profile.phone,
                &profile.photo_id,
                &serde_json::to_value(&profile.projects).expect("projects should be valid json"),
                &profile.skills,
                &profile.twitter_url,
                &profile.website_url,
            ],
        )
        .await?;

        Ok(())
    }
}
