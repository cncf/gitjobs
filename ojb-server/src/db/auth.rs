//! This module defines some database functionality used for authentication and
//! authorization.

use anyhow::Result;
use async_trait::async_trait;
use axum_login::tower_sessions::session;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    auth::{NewUser, User},
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

    /// Get user by id.
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>>;

    /// Get user by username.
    async fn get_user_by_username(&self, job_board_id: &Uuid, username: &str) -> Result<Option<User>>;

    /// Sign up a new user.
    async fn sign_up_user(&self, job_board_id: &Uuid, user: &NewUser) -> Result<()>;

    /// Update session.
    async fn update_session(&self, record: &session::Record) -> Result<()>;
}

#[async_trait]
impl DBAuth for PgDB {
    /// [DBAuth::create_session]
    #[instrument(skip(self), err)]
    async fn create_session(&self, record: &session::Record) -> Result<()> {
        trace!("creating session in database");

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

    /// [DBAuth::delete_session]
    #[instrument(skip(self), err)]
    async fn delete_session(&self, session_id: &session::Id) -> Result<()> {
        trace!("deleting session from database");

        let db = self.pool.get().await?;
        db.execute(
            "delete from session where session_id = $1::text;",
            &[&session_id.to_string()],
        )
        .await?;

        Ok(())
    }

    /// [DBAuth::get_session]
    #[instrument(skip(self), err)]
    async fn get_session(&self, session_id: &session::Id) -> Result<Option<session::Record>> {
        trace!("getting session from database");

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

    /// [DBAuth::get_user_by_id]
    #[instrument(skip(self), err)]
    async fn get_user_by_id(&self, user_id: &Uuid) -> Result<Option<User>> {
        trace!("getting user (by id) from database");

        let db = self.pool.get().await?;
        let user = db
            .query_opt(
                r#"
                select
                    user_id,
                    auth_hash,
                    email,
                    email_verified,
                    first_name,
                    last_name,
                    username
                from "user"
                where user_id = $1::uuid
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
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                password: String::new(),
                username: row.get("username"),
            });

        Ok(user)
    }

    /// [DBAuth::get_user_by_username]
    #[instrument(skip(self), err)]
    async fn get_user_by_username(&self, job_board_id: &Uuid, username: &str) -> Result<Option<User>> {
        trace!("getting user (by username) from database");

        let db = self.pool.get().await?;
        let user = db
            .query_opt(
                r#"
                select
                    user_id,
                    auth_hash,
                    email,
                    email_verified,
                    first_name,
                    last_name,
                    password,
                    username
                from "user"
                where username = $1::text
                and job_board_id = $2::uuid
                and email_verified = true;
                "#,
                &[&username, &job_board_id],
            )
            .await?
            .map(|row| User {
                user_id: row.get("user_id"),
                auth_hash: row.get("auth_hash"),
                email: row.get("email"),
                email_verified: row.get("email_verified"),
                first_name: row.get("first_name"),
                last_name: row.get("last_name"),
                password: row.get("password"),
                username: row.get("username"),
            });

        Ok(user)
    }

    /// [DBAuth::sign_up_user]
    #[instrument(skip(self), err)]
    async fn sign_up_user(&self, job_board_id: &Uuid, user: &NewUser) -> Result<()> {
        trace!("signing up user in database");

        let db = self.pool.get().await?;
        db.execute(
            r#"
            insert into "user" (
                auth_hash,
                email,
                first_name,
                last_name,
                password,
                username,
                job_board_id
            )
            values (
                gen_random_bytes(32),
                $1::text,
                $2::text,
                $3::text,
                $4::text,
                $5::text,
                $6::uuid
            );
            "#,
            &[
                &user.email,
                &user.first_name,
                &user.last_name,
                &user.password,
                &user.username,
                &job_board_id,
            ],
        )
        .await?;

        Ok(())
    }

    /// [DBAuth::update_session]
    #[instrument(skip(self), err)]
    async fn update_session(&self, record: &session::Record) -> Result<()> {
        trace!("updating session in database");

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
}
