//! This module contains the authentication and authorization functionality.

use std::{collections::HashMap, sync::Arc};

use anyhow::{bail, Result};
use async_trait::async_trait;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_login::{
    tower_sessions::{self, session, session_store},
    AuthManagerLayer, AuthManagerLayerBuilder,
};
use oauth2::{reqwest, TokenResponse};
use password_auth::verify_password;
use serde::{Deserialize, Serialize};
use time::Duration;
use tower_sessions::{cookie::SameSite, CachingSessionStore, Expiry, SessionManagerLayer};
use tower_sessions_moka_store::MokaStore;
use uuid::Uuid;

use crate::{
    config::{HttpServerConfig, OAuth2Config},
    db::DynDB,
};

/// Type alias for the auth layer.
pub(crate) type AuthLayer = AuthManagerLayer<AuthnBackend, CachingSessionStore<MokaStore, SessionStore>>;

/// Setup router authentication/authorization layer.
pub(crate) fn setup_layer(cfg: &HttpServerConfig, db: DynDB) -> Result<AuthLayer> {
    // Setup session store
    let session_store = SessionStore::new(db.clone());
    let moka_store = MokaStore::new(Some(1000));
    let caching_session_store = CachingSessionStore::new(moka_store, session_store);

    // Setup session layer
    let secure = if let Some(cookie) = &cfg.cookie {
        cookie.secure.unwrap_or(true)
    } else {
        true
    };
    let session_layer = SessionManagerLayer::new(caching_session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(secure);

    // Setup auth layer
    let authn_backend = AuthnBackend::new(&cfg.oauth2, db)?;
    let auth_layer = AuthManagerLayerBuilder::new(authn_backend, session_layer).build();

    Ok(auth_layer)
}

// Session store.

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

    /// Convert an `anyhow::Error` to a `tower_sessions::session_store::Error`.
    #[allow(clippy::needless_pass_by_value)]
    fn to_session_store_error(err: anyhow::Error) -> session_store::Error {
        session_store::Error::Backend(err.to_string())
    }
}

#[async_trait]
impl tower_sessions::SessionStore for SessionStore {
    async fn create(&self, record: &mut session::Record) -> session_store::Result<()> {
        self.db
            .create_session(record)
            .await
            .map_err(Self::to_session_store_error)
    }

    async fn save(&self, record: &session::Record) -> session_store::Result<()> {
        self.db
            .update_session(record)
            .await
            .map_err(Self::to_session_store_error)
    }

    async fn load(&self, session_id: &session::Id) -> session_store::Result<Option<session::Record>> {
        self.db
            .get_session(session_id)
            .await
            .map_err(Self::to_session_store_error)
    }

    async fn delete(&self, session_id: &session::Id) -> session_store::Result<()> {
        self.db
            .delete_session(session_id)
            .await
            .map_err(Self::to_session_store_error)
    }
}

impl std::fmt::Debug for SessionStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SessionStore").finish_non_exhaustive()
    }
}

// Authentication backend.

/// Backend used to authenticate users.
#[derive(Clone)]
pub(crate) struct AuthnBackend {
    db: DynDB,
    http_client: reqwest::Client,
    pub oauth2_providers: OAuth2Providers,
}

impl AuthnBackend {
    /// Create a new `AuthnBackend` instance.
    pub fn new(oauth2_cfg: &OAuth2Config, db: DynDB) -> Result<Self> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
            .build()?;
        let oauth2_providers = Self::setup_oauth2_providers(oauth2_cfg)?;

        Ok(Self {
            db,
            http_client,
            oauth2_providers,
        })
    }

    /// Authenticate a user using `OAuth2` credentials.
    async fn authenticate_oauth2(&self, creds: OAuth2Credentials) -> Result<Option<User>> {
        // Exchange the authorization code for an access token
        let Some(oauth2_provider) = self.oauth2_providers.get(&creds.provider) else {
            bail!("oauth2 client not found")
        };
        let access_token = oauth2_provider
            .client
            .exchange_code(oauth2::AuthorizationCode::new(creds.code))
            .request_async(&self.http_client)
            .await?
            .access_token()
            .secret()
            .to_string();

        // Get the user if they exist, otherwise sign them up
        let new_user = match creds.provider {
            OAuth2Provider::GitHub => NewUser::from_github_profile(&access_token).await?,
        };
        let user = if let Some(user) = self
            .db
            .get_user_by_email(&creds.job_board_id, &new_user.email)
            .await?
        {
            user
        } else {
            self.db.sign_up_user(&creds.job_board_id, &new_user, true).await?
        };

        Ok(Some(user))
    }

    /// Authenticate a user using password credentials.
    async fn authenticate_password(&self, creds: PasswordCredentials) -> Result<Option<User>> {
        // Ensure job board id is present
        let Some(job_board_id) = creds.job_board_id else {
            bail!("job_board_id missing")
        };

        // Get user from database
        let user = self.db.get_user_by_username(&job_board_id, &creds.username).await?;

        // Check if the credentials are valid, returning the user if they are
        if let Some(mut user) = user {
            // Check if the user's password is set
            let Some(password) = user.password.clone() else {
                return Ok(None);
            };

            // Verify the password
            if tokio::task::spawn_blocking(move || verify_password(creds.password, &password))
                .await?
                .is_ok()
            {
                user.password = None;
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    /// Setup `OAuth2` providers.
    fn setup_oauth2_providers(oauth2_cfg: &OAuth2Config) -> Result<OAuth2Providers> {
        let mut providers: OAuth2Providers = HashMap::new();

        for (provider, cfg) in oauth2_cfg {
            let client = oauth2::basic::BasicClient::new(oauth2::ClientId::new(cfg.client_id.clone()))
                .set_client_secret(oauth2::ClientSecret::new(cfg.client_secret.clone()))
                .set_auth_uri(oauth2::AuthUrl::new(cfg.auth_url.clone())?)
                .set_token_uri(oauth2::TokenUrl::new(cfg.token_url.clone())?)
                .set_redirect_uri(oauth2::RedirectUrl::new(cfg.redirect_uri.clone())?);

            providers.insert(
                provider.clone(),
                Arc::new(OAuth2ProviderDetails {
                    client,
                    scopes: cfg.scopes.clone(),
                }),
            );
        }

        Ok(providers)
    }
}

#[async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::OAuth2(creds) => self.authenticate_oauth2(creds).await.map_err(AuthError),
            Credentials::Password(creds) => self.authenticate_password(creds).await.map_err(AuthError),
        }
    }

    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        // Get user from database
        self.db.get_user_by_id(user_id).await.map_err(AuthError)
    }
}

/// Type alias for `AuthSession` that includes our authentication backend.
pub(crate) type AuthSession = axum_login::AuthSession<AuthnBackend>;

/// Type alias for the structure that holds the `OAuth2` providers.
pub(crate) type OAuth2Providers = HashMap<OAuth2Provider, Arc<OAuth2ProviderDetails>>;

/// Supported `OAuth2` providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OAuth2Provider {
    GitHub,
}

/// `OAuth2` provider client and scopes.
#[derive(Clone)]
pub(crate) struct OAuth2ProviderDetails {
    pub client: oauth2::basic::BasicClient<
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
    pub scopes: Vec<String>,
}

/// Wrapper around `anyhow::Error` to represent auth errors.
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(crate) struct AuthError(#[from] anyhow::Error);

/// Credentials used to authenticate a user.
#[derive(Clone, Serialize, Deserialize)]
pub enum Credentials {
    OAuth2(OAuth2Credentials),
    Password(PasswordCredentials),
}

/// `OAuth2` credentials.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct OAuth2Credentials {
    pub code: String,
    pub job_board_id: Uuid,
    pub provider: OAuth2Provider,
}

/// Password credentials.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PasswordCredentials {
    pub username: String,
    pub password: String,

    pub job_board_id: Option<Uuid>,
}

// User types and implementations.

/// User information.
#[derive(Clone)]
#[allow(clippy::struct_field_names, dead_code)]
pub(crate) struct User {
    pub user_id: Uuid,
    pub auth_hash: Vec<u8>,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub username: String,

    pub password: Option<String>,
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
    pub name: String,
    pub username: String,

    #[serde(skip_serializing)]
    pub password: Option<String>,
}

impl NewUser {
    /// Create a `NewUser` instance from a GitHub profile.
    async fn from_github_profile(access_token: &str) -> Result<Self> {
        let profile = reqwest::Client::new()
            .get("https://api.github.com/user")
            .header(USER_AGENT.as_str(), "open-job-board")
            .header(AUTHORIZATION.as_str(), format!("Bearer {access_token}"))
            .send()
            .await?
            .json::<GitHubProfile>()
            .await?;

        Ok(Self {
            email: profile.email,
            name: profile.name,
            username: profile.login,
            password: None,
        })
    }
}

impl std::fmt::Debug for NewUser {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("NewUser")
            .field("email", &self.email)
            .field("name", &self.name)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// GitHub profile information.
#[derive(Debug, Deserialize)]
struct GitHubProfile {
    login: String,
    name: String,
    email: String,
}
