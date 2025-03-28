//! This module defines some database functionality used for authentication and
//! authorization.

use anyhow::{Result, anyhow};
use async_trait::async_trait;
use axum_login::tower_sessions::session;
use cached::proc_macro::cached;
use deadpool_postgres::Object;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    auth::{User, UserSummary},
    db::PgDB,
};

/// Trait that defines some database operations used for authentication and
/// authorization.
#[async_trait]
pub(crate) trait DBAuth {
    /// Create a new session.
    async fn create_session(&self, record: &session::Record) -> Result<()>;

    /// Delete session.
    async fn delete_session(&self, session_id: &session::Id) -> Result<()>;

    /// Get session.
    async fn get_session(&self, session_id: &session::Id) -> Result<Option<session::Record>>;

    /// Get user by email.
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Get user by id.
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>>;

    /// Get user by username.
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Get user password.
    async fn get_user_password(&self, user_id: &Uuid) -> Result<Option<String>>;

    /// Check if the image is public.
    async fn is_image_public(&self, image_id: &Uuid) -> Result<bool>;

    /// Sign up a new user.
    async fn sign_up_user(
        &self,
        user_summary: &UserSummary,
        email_verified: bool,
    ) -> Result<(User, Option<VerificationCode>)>;

    /// Update session.
    async fn update_session(&self, record: &session::Record) -> Result<()>;

    /// Update user details.
    async fn update_user_details(&self, user_id: &Uuid, user_summary: &UserSummary) -> Result<()>;

    /// Update user password.
    async fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<()>;

    /// Check if the user has access to the image.
    async fn user_has_image_access(&self, user_id: &Uuid, image_id: &Uuid) -> Result<bool>;

    /// Check if the user has access to the profile.
    async fn user_has_profile_access(&self, user_id: &Uuid, job_seeker_profile_id: &Uuid) -> Result<bool>;

    /// Check if the user owns the employer.
    async fn user_owns_employer(&self, user_id: &Uuid, employer_id: &Uuid) -> Result<bool>;

    /// Check if the user owns the job
    async fn user_owns_job(&self, user_id: &Uuid, job_id: &Uuid) -> Result<bool>;

    /// Verify email.
    async fn verify_email(&self, code: &Uuid) -> Result<()>;
}

#[async_trait]
impl DBAuth for PgDB {
    #[instrument(skip(self, record), err)]
    async fn create_session(&self, record: &session::Record) -> Result<()> {
        trace!("db: create session");

        let db = self.pool.get().await?;
        db.execute(
            "
            insert into session (
                session_id,
                data,
                expires_at
            ) values (
                $1::text,
                $2::jsonb,
                $3::timestamptz
            );
            ",
            &[
                &record.id.to_string(),
                &serde_json::to_value(&record.data)?,
                &record.expiry_date,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, session_id), err)]
    async fn delete_session(&self, session_id: &session::Id) -> Result<()> {
        trace!("db: delete session");

        let db = self.pool.get().await?;
        db.execute(
            "delete from session where session_id = $1::text;",
            &[&session_id.to_string()],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, session_id), err)]
    async fn get_session(&self, session_id: &session::Id) -> Result<Option<session::Record>> {
        trace!("db: get session");

        let db = self.pool.get().await?;
        let row = db
            .query_opt(
                "select data, expires_at from session where session_id = $1::text;",
                &[&session_id.to_string()],
            )
            .await?;

        if let Some(row) = row {
            let record = session::Record {
                id: *session_id,
                data: serde_json::from_value(row.get("data"))?,
                expiry_date: row.get("expires_at"),
            };
            return Ok(Some(record));
        }

        Ok(None)
    }

    #[instrument(skip(self, email), err)]
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>> {
        trace!("db: get user (by email)");

        let db = self.pool.get().await?;
        let user = db
            .query_opt(
                r#"
                select
                    u.user_id,
                    u.auth_hash,
                    u.email,
                    u.email_verified,
                    u.password is not null as has_password,
                    u.name,
                    u.username,
                    p.job_seeker_profile_id is not null as has_profile
                from "user" u
                left join job_seeker_profile p on u.user_id = p.user_id
                where u.email = $1::text
                and u.email_verified = true;
                "#,
                &[&email],
            )
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                has_password: row.get("has_password"),
                has_profile: row.get("has_profile"),
                name: row.get("name"),
                password: None,
                username: row.get("username"),
            });

        Ok(user)
    }

    #[instrument(skip(self), err)]
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>> {
        trace!("db: get user (by id)");

        let db = self.pool.get().await?;
        let user = db
            .query_opt(
                r#"
                select
                    u.user_id,
                    u.auth_hash,
                    u.email,
                    u.email_verified,
                    u.password is not null as has_password,
                    u.name,
                    u.username,
                    p.job_seeker_profile_id is not null as has_profile
                from "user" u
                left join job_seeker_profile p on u.user_id = p.user_id
                where u.user_id = $1::uuid
                and email_verified = true;
                "#,
                &[&user_id],
            )
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                has_password: row.get("has_password"),
                has_profile: row.get("has_profile"),
                name: row.get("name"),
                password: None,
                username: row.get("username"),
            });

        Ok(user)
    }

    #[instrument(skip(self), err)]
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>> {
        trace!("db: get user (by username)");

        let db = self.pool.get().await?;
        let user = db
            .query_opt(
                r#"
                select
                    u.user_id,
                    u.auth_hash,
                    u.email,
                    u.email_verified,
                    u.password is not null as has_password,
                    u.name,
                    u.password,
                    u.username,
                    p.job_seeker_profile_id is not null as has_profile
                from "user" u
                left join job_seeker_profile p on u.user_id = p.user_id
                where u.username = $1::text
                and password is not null
                and email_verified = true;
                "#,
                &[&username],
            )
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                has_password: row.get("has_password"),
                has_profile: row.get("has_profile"),
                name: row.get("name"),
                password: row.get("password"),
                username: row.get("username"),
            });

        Ok(user)
    }

    #[instrument(skip(self), err)]
    async fn get_user_password(&self, user_id: &Uuid) -> Result<Option<String>> {
        trace!("db: get user password");

        let db = self.pool.get().await?;
        let password = db
            .query_opt(
                r#"select password from "user" where user_id = $1::uuid;"#,
                &[&user_id],
            )
            .await?
            .map(|row| row.get("password"));

        Ok(password)
    }

    #[instrument(skip(self), err)]
    async fn is_image_public(&self, image_id: &Uuid) -> Result<bool> {
        #[cached(
            key = "Uuid",
            convert = r#"{ image_id.clone() }"#,
            sync_writes = "by_key",
            result = true
        )]
        async fn inner(db: Object, image_id: &Uuid) -> Result<bool> {
            trace!("db: check if image is public");

            let row = db
                .query_one(
                    "
                    select exists (
                        select 1
                        from employer e
                        join job j using (employer_id)
                        where e.logo_id = $1::uuid
                        and j.status = 'published'
                    ) as is_public;
                    ",
                    &[&image_id],
                )
                .await?;

            Ok(row.get("is_public"))
        }

        let db = self.pool.get().await?;
        inner(db, image_id).await
    }

    #[instrument(skip(self, user_summary, email_verified), err)]
    async fn sign_up_user(
        &self,
        user_summary: &UserSummary,
        email_verified: bool,
    ) -> Result<(User, Option<VerificationCode>)> {
        trace!("db: sign up user");

        // Start a transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Add user to the database
        let row = tx
            .query_one(
                r#"
                insert into "user" (
                    auth_hash,
                    email,
                    email_verified,
                    name,
                    password,
                    username
                ) values (
                    gen_random_bytes(32),
                    $1::text,
                    $2::boolean,
                    $3::text,
                    $4::text,
                    $5::text
                ) returning
                    user_id,
                    auth_hash,
                    email,
                    email_verified,
                    name,
                    username;
                "#,
                &[
                    &user_summary.email,
                    &email_verified,
                    &user_summary.name,
                    &user_summary.password,
                    &user_summary.username,
                ],
            )
            .await?;
        let user = User {
            user_id: row.get("user_id"),
            auth_hash: row.get("auth_hash"),
            email: row.get("email"),
            email_verified: row.get("email_verified"),
            has_password: Some(true),
            has_profile: false,
            name: row.get("name"),
            password: None,
            username: row.get("username"),
        };

        // Create email verification code if the email is not yet verified
        let mut email_verification_code = None;
        if !email_verified {
            let row = tx
                .query_one(
                    "
                    insert into email_verification_code (user_id)
                    values ($1::uuid)
                    returning email_verification_code_id;
                    ",
                    &[&user.user_id],
                )
                .await?;
            email_verification_code = Some(row.get("email_verification_code_id"));
        }

        // Commit the transaction
        tx.commit().await?;

        Ok((user, email_verification_code))
    }

    #[instrument(skip(self, record), err)]
    async fn update_session(&self, record: &session::Record) -> Result<()> {
        trace!("db: update session");

        let db = self.pool.get().await?;
        db.execute(
            "
            update session set
                data = $2::jsonb,
                expires_at = $3::timestamptz
            where session_id = $1::text;
            ",
            &[
                &record.id.to_string(),
                &serde_json::to_value(&record.data)?,
                &record.expiry_date,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, user_summary), err)]
    async fn update_user_details(&self, user_id: &Uuid, user_summary: &UserSummary) -> Result<()> {
        trace!("db: update user details");

        let db = self.pool.get().await?;
        db.execute(
            r#"
            update "user" set
                email = $2::text,
                name = $3::text,
                username = $4::text
            where user_id = $1::uuid;
            "#,
            &[
                &user_id,
                &user_summary.email,
                &user_summary.name,
                &user_summary.username,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, new_password), err)]
    async fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<()> {
        trace!("db: update user password");

        let db = self.pool.get().await?;
        db.execute(
            r#"
            update "user" set
                auth_hash = gen_random_bytes(32), -- Invalidate existing sessions
                password = $2::text
            where user_id = $1::uuid;
            "#,
            &[&user_id, &new_password],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn user_has_image_access(&self, user_id: &Uuid, image_id: &Uuid) -> Result<bool> {
        trace!("db: check if user has access to image");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select user_has_image_access($1::uuid, $2::uuid);",
                &[&user_id, &image_id],
            )
            .await?;

        Ok(row.get(0))
    }

    #[instrument(skip(self), err)]
    async fn user_has_profile_access(&self, user_id: &Uuid, job_seeker_profile_id: &Uuid) -> Result<bool> {
        trace!("db: check if user has access to profile");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select exists (
                    select 1
                    from job_seeker_profile p
                    join application a on p.job_seeker_profile_id = a.job_seeker_profile_id
                    join job j on a.job_id = j.job_id
                    join employer_team et on j.employer_id = et.employer_id
                    where et.user_id = $1::uuid
                    and p.job_seeker_profile_id = $2::uuid
                ) as has_access;
                ",
                &[&user_id, &job_seeker_profile_id],
            )
            .await?;

        Ok(row.get(0))
    }

    #[instrument(skip(self), err)]
    async fn user_owns_employer(&self, user_id: &Uuid, employer_id: &Uuid) -> Result<bool> {
        trace!("db: check if user owns employer");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select exists (
                    select 1
                    from employer_team
                    where user_id = $1::uuid
                    and employer_id = $2::uuid
                ) as owns_employer;
                ",
                &[&user_id, &employer_id],
            )
            .await?;

        Ok(row.get("owns_employer"))
    }

    #[instrument(skip(self), err)]
    async fn user_owns_job(&self, user_id: &Uuid, job_id: &Uuid) -> Result<bool> {
        trace!("db: check if user owns job");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "
                select exists (
                    select 1
                    from job j
                    join employer_team et using (employer_id)
                    where et.user_id = $1::uuid
                    and j.job_id = $2::uuid
                ) as owns_job;
                ",
                &[&user_id, &job_id],
            )
            .await?;

        Ok(row.get("owns_job"))
    }

    #[instrument(skip(self, code), err)]
    async fn verify_email(&self, code: &Uuid) -> Result<()> {
        trace!("db: verify email");

        // Start a transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Verify email
        let user_id: Uuid = tx
            .query_opt(
                "
                delete from email_verification_code
                where email_verification_code_id = $1::uuid
                and created_at > current_timestamp - interval '1 day'
                returning user_id;
                ",
                &[&code],
            )
            .await?
            .map(|row| row.get("user_id"))
            .ok_or_else(|| anyhow!("invalid email verification code"))?;

        // Mark email as verified
        tx.execute(
            r#"update "user" set email_verified = true where user_id = $1::uuid;"#,
            &[&user_id],
        )
        .await?;

        // Commit the transaction
        tx.commit().await?;

        Ok(())
    }
}

/// Type alias for the email verification code.
pub(crate) type VerificationCode = Uuid;
