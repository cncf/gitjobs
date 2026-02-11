//! This module provides database operations for authentication and authorization.

use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use axum_login::tower_sessions::session;
use cached::proc_macro::cached;
use deadpool_postgres::Object;
use tokio_postgres::types::Json;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    auth::{User, UserSummary},
    db::PgDB,
};

/// Trait for database operations related to authentication and authorization.
#[async_trait]
pub(crate) trait DBAuth {
    /// Creates a new session in the database.
    async fn create_session(&self, record: &session::Record) -> Result<()>;

    /// Deletes a session from the database.
    async fn delete_session(&self, session_id: &session::Id) -> Result<()>;

    /// Retrieves a session by its ID.
    async fn get_session(&self, session_id: &session::Id) -> Result<Option<session::Record>>;

    /// Retrieves a user by their email address.
    async fn get_user_by_email(&self, email: &str) -> Result<Option<User>>;

    /// Retrieves a user by their unique ID.
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>>;

    /// Retrieves a user by their username.
    async fn get_user_by_username(&self, username: &str) -> Result<Option<User>>;

    /// Retrieves the password hash for a user.
    async fn get_user_password(&self, user_id: &Uuid) -> Result<Option<String>>;

    /// Checks if an image is public.
    async fn is_image_public(&self, image_id: &Uuid) -> Result<bool>;

    /// Registers a new user in the database.
    async fn sign_up_user(
        &self,
        user_summary: &UserSummary,
        email_verified: bool,
    ) -> Result<(User, Option<VerificationCode>)>;

    /// Updates an existing session in the database.
    async fn update_session(&self, record: &session::Record) -> Result<()>;

    /// Updates user details in the database.
    async fn update_user_details(&self, user_id: &Uuid, user_summary: &UserSummary) -> Result<()>;

    /// Updates a user's password in the database.
    async fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<()>;

    /// Checks if a user has access to a specific image.
    async fn user_has_image_access(&self, user_id: &Uuid, image_id: &Uuid) -> Result<bool>;

    /// Checks if a user has access to a specific job seeker profile.
    async fn user_has_profile_access(&self, user_id: &Uuid, job_seeker_profile_id: &Uuid) -> Result<bool>;

    /// Checks if a user owns a specific employer.
    async fn user_owns_employer(&self, user_id: &Uuid, employer_id: &Uuid) -> Result<bool>;

    /// Checks if a user owns a specific job.
    async fn user_owns_job(&self, user_id: &Uuid, job_id: &Uuid) -> Result<bool>;

    /// Verifies a user's email address using a verification code.
    async fn verify_email(&self, code: &Uuid) -> Result<()>;
}

/// Implementation of `DBAuth` for `PgDB`, providing all authentication and authorization
/// related database operations.
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
            .query_opt("select * from auth_get_user_by_email($1::text);", &[&email])
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                has_password: row.get("has_password"),
                has_profile: row.get("has_profile"),
                moderator: row.get("moderator"),
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
                "select * from auth_get_user_by_id_verified($1::uuid);",
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
                moderator: row.get("moderator"),
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
            .query_opt("select * from auth_get_user_by_username($1::text);", &[&username])
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                has_password: row.get("has_password"),
                has_profile: row.get("has_profile"),
                moderator: row.get("moderator"),
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
        let password: Option<String> = db
            .query_one("select auth_get_user_password($1::uuid);", &[&user_id])
            .await?
            .get(0);

        Ok(password)
    }

    #[instrument(skip(self), err)]
    async fn is_image_public(&self, image_id: &Uuid) -> Result<bool> {
        #[cached(
            time = 86400,
            key = "Uuid",
            convert = r#"{ image_id.clone() }"#,
            sync_writes = "by_key",
            result = true
        )]
        async fn inner(db: Object, image_id: &Uuid) -> Result<bool> {
            trace!("db: check if image is public");

            let row = db
                .query_one("select auth_is_image_public($1::uuid);", &[&image_id])
                .await?;

            Ok(row.get(0))
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

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select * from auth_sign_up_user($1::jsonb, $2::boolean);",
                &[&Json(user_summary), &email_verified],
            )
            .await?;
        let user = User {
            user_id: row.get("user_id"),
            auth_hash: row.get("auth_hash"),
            email: row.get("email"),
            email_verified: row.get("email_verified"),
            has_password: row.get("has_password"),
            has_profile: row.get("has_profile"),
            moderator: row.get("moderator"),
            name: row.get("name"),
            password: None,
            username: row.get("username"),
        };
        let email_verification_code = row.get("verification_code");

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
            "select auth_update_user_details($1::uuid, $2::jsonb);",
            &[&user_id, &Json(user_summary)],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self, new_password), err)]
    async fn update_user_password(&self, user_id: &Uuid, new_password: &str) -> Result<()> {
        trace!("db: update user password");

        let db = self.pool.get().await?;
        db.execute(
            "select auth_update_user_password($1::uuid, $2::text);",
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
                "select auth_user_has_image_access($1::uuid, $2::uuid);",
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
                "select auth_user_has_profile_access($1::uuid, $2::uuid);",
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
                "select auth_user_owns_employer($1::uuid, $2::uuid);",
                &[&user_id, &employer_id],
            )
            .await?;

        Ok(row.get(0))
    }

    #[instrument(skip(self), err)]
    async fn user_owns_job(&self, user_id: &Uuid, job_id: &Uuid) -> Result<bool> {
        trace!("db: check if user owns job");

        let db = self.pool.get().await?;
        let row = db
            .query_one(
                "select auth_user_owns_job($1::uuid, $2::uuid);",
                &[&user_id, &job_id],
            )
            .await?;

        Ok(row.get(0))
    }

    #[instrument(skip(self, code), err)]
    async fn verify_email(&self, code: &Uuid) -> Result<()> {
        trace!("db: verify email");

        let db = self.pool.get().await?;
        db.execute("select auth_verify_email($1::uuid);", &[&code]).await?;

        Ok(())
    }
}

/// Type alias for the email verification code (UUID).
pub(crate) type VerificationCode = Uuid;
