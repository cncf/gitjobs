//! This module defines some database functionality for the job seeker
//! dashboard.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{PgDB, templates::dashboard::job_seeker::profile::JobSeekerProfile};

/// Trait that defines some database operations used in the job seeker
/// dashboard.
#[async_trait]
pub(crate) trait DBDashBoardJobSeeker {
    /// Get job seeker profile.
    async fn get_job_seeker_profile(&self, user_id: &Uuid) -> Result<Option<JobSeekerProfile>>;

    /// Update job seeker profile.
    async fn update_job_seeker_profile(&self, user_id: &Uuid, profile: &JobSeekerProfile) -> Result<()>;
}

#[async_trait]
impl DBDashBoardJobSeeker for PgDB {
    /// [DBDashBoardJobSeeker::get_job_seeker_profile]
    #[instrument(skip(self), err)]
    async fn get_job_seeker_profile(&self, user_id: &Uuid) -> Result<Option<JobSeekerProfile>> {
        let db = self.pool.get().await?;
        let profile = db
            .query_opt(
                "
                select
                    p.email,
                    p.name,
                    p.public,
                    p.summary,
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
                    l.city,
                    l.country,
                    l.state
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
                location_id: row.get("location_id"),
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
                city: row.get("city"),
                country: row.get("country"),
                state: row.get("state"),
            });

        Ok(profile)
    }

    /// [DBDashBoardJobSeeker::update_job_seeker_profile]
    #[instrument(skip(self), err)]
    async fn update_job_seeker_profile(&self, user_id: &Uuid, profile: &JobSeekerProfile) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            insert into job_seeker_profile (
                user_id,
                email,
                name,
                public,
                summary,
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
                nullif($6::jsonb, 'null'::jsonb),
                nullif($7::jsonb, 'null'::jsonb),
                nullif($8::jsonb, 'null'::jsonb),
                $9::text,
                $10::text,
                $11::text,
                $12::uuid,
                $13::boolean,
                $14::boolean,
                $15::text,
                $16::uuid,
                nullif($17::jsonb, 'null'::jsonb),
                $18::text[],
                $19::text,
                $20::text
            )
            on conflict (user_id) do update set
                email = excluded.email,
                name = excluded.name,
                public = excluded.public,
                summary = excluded.summary,
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
                &serde_json::to_value(&profile.certifications).expect("certifications should be valid json"),
                &serde_json::to_value(&profile.education).expect("education should be valid json"),
                &serde_json::to_value(&profile.experience).expect("experience should be valid json"),
                &profile.facebook_url,
                &profile.github_url,
                &profile.linkedin_url,
                &profile.location_id,
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
