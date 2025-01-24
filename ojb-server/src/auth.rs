//! This module contains the authentication and authorization functionality.

use std::fmt::Debug;

use anyhow::anyhow;
use async_trait::async_trait;
use axum_login::tower_sessions::{self, session, session_store};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{db::DynDB, templates::dashboard::employers::EmployerSummary};

/// Wrapper around `anyhow::Error` to represent auth errors.
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(crate) struct AuthError(#[from] anyhow::Error);

/// User information.
#[derive(Clone)]
#[allow(clippy::struct_field_names, dead_code)]
pub(crate) struct User {
    pub user_id: Uuid,
    pub auth_hash: Vec<u8>,
    pub email: String,
    pub email_verified: bool,
    pub employers: Vec<EmployerSummary>,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub username: String,
}

impl axum_login::AuthUser for User {
    type Id = Uuid;

    fn id(&self) -> Self::Id {
        self.user_id
    }

    fn session_auth_hash(&self) -> &[u8] {
        &self.auth_hash
    }
}

impl std::fmt::Debug for User {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("User")
            .field("user_id", &self.user_id)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// User information required to sign up a new user.
#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct NewUser {
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub password: String,
    pub username: String,
}

impl std::fmt::Debug for NewUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewUser")
            .field("email", &self.email)
            .field("first_name", &self.first_name)
            .field("last_name", &self.last_name)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// Backend used to authenticate users.
#[derive(Clone)]
pub(crate) struct AuthnBackend {
    db: DynDB,
}

impl AuthnBackend {
    /// Create a new `AuthnBackend` instance.
    pub fn new(db: DynDB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        // Ensure job board id is present
        let Some(job_board_id) = creds.job_board_id else {
            return Err(anyhow!("job_board_id missing").into());
        };

        // Get user from database
        let user = self.db.get_user_by_username(&job_board_id, &creds.username).await?;

        // Check if the credentials are valid, returning the user if they are
        if let Some(mut user) = user {
            let password = user.password.clone();
            if tokio::task::spawn_blocking(move || verify_password(creds.password, &password))
                .await
                .map_err(anyhow::Error::from)?
                .is_ok()
            {
                user.password.clear();
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // Get user from database
        self.db.get_user_by_id(user_id).await.map_err(AuthError)
    }
}

/// Credentials used to authenticate a user.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct Credentials {
    pub username: String,
    pub password: String,

    pub job_board_id: Option<Uuid>,
    pub next_url: Option<String>,
}

/// Type alias that includes our authentication backend.
pub(crate) type AuthSession = axum_login::AuthSession<AuthnBackend>;

/// Store used to manage user sessions.
#[derive(Clone)]
pub(crate) struct SessionStore {
    db: DynDB,
}

impl SessionStore {
    /// Create a new `SessionStore` instance.
    pub fn new(db: DynDB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl tower_sessions::SessionStore for SessionStore {
    async fn create(&self, record: &mut session::Record) -> session_store::Result<()> {
        self.db.create_session(record).await.map_err(to_session_store_error)
    }

    async fn save(&self, record: &session::Record) -> session_store::Result<()> {
        self.db.update_session(record).await.map_err(to_session_store_error)
    }

    async fn load(&self, session_id: &session::Id) -> session_store::Result<Option<session::Record>> {
        self.db.get_session(session_id).await.map_err(to_session_store_error)
    }

    async fn delete(&self, session_id: &session::Id) -> session_store::Result<()> {
        self.db
            .delete_session(session_id)
            .await
            .map_err(to_session_store_error)
    }
}

impl Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OJBSessionStore").finish_non_exhaustive()
    }
}

/// Convert an `anyhow::Error` to a `tower_sessions::session_store::Error`.
#[allow(clippy::needless_pass_by_value)]
fn to_session_store_error(err: anyhow::Error) -> session_store::Error {
    session_store::Error::Backend(err.to_string())
}
