//! This module contains authentication and authorization logic for the server.

use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_login::{
    AuthManagerLayer, AuthManagerLayerBuilder,
    tower_sessions::{self, session, session_store},
};
use garde::Validate;
use oauth2::{TokenResponse, reqwest as oauth2_reqwest};
use openidconnect::{self as oidc, LocalizedClaim};
use password_auth::verify_password;
use reqwest::header::HeaderMap;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use time::Duration;
use tower_sessions::{Expiry, SessionManagerLayer, cookie::SameSite};
use uuid::Uuid;

use crate::{
    config::{HttpServerConfig, OAuth2Config, OAuth2Provider, OidcConfig, OidcProvider},
    db::DynDB,
    validation::{
        MAX_LEN_DISPLAY_NAME, MAX_LEN_M, MAX_LEN_S, MIN_PASSWORD_LEN, trimmed_non_empty,
        trimmed_non_empty_opt,
    },
};

/// Type alias for the authentication layer used in the router.
pub(crate) type AuthLayer = AuthManagerLayer<AuthnBackend, SessionStore>;

/// Setup router authentication/authorization layer.
pub(crate) async fn setup_layer(cfg: &HttpServerConfig, db: DynDB) -> Result<AuthLayer> {
    // Setup session layer
    let session_store = SessionStore::new(db.clone());
    let secure = if let Some(cookie) = &cfg.cookie {
        cookie.secure.unwrap_or(true)
    } else {
        true
    };
    let session_layer = SessionManagerLayer::new(session_store)
        .with_expiry(Expiry::OnInactivity(Duration::days(7)))
        .with_http_only(true)
        .with_same_site(SameSite::Lax)
        .with_secure(secure);

    // Setup auth layer
    let authn_backend = AuthnBackend::new(db, &cfg.oauth2, &cfg.oidc).await?;
    let auth_layer = AuthManagerLayerBuilder::new(authn_backend, session_layer).build();

    Ok(auth_layer)
}

// Session store.

/// Store for managing user sessions in the database.
#[derive(Clone)]
pub(crate) struct SessionStore {
    db: DynDB,
}

impl SessionStore {
    /// Create a new `SessionStore` with the given database handle.
    pub fn new(db: DynDB) -> Self {
        Self { db }
    }

    /// Convert an `anyhow::Error` to a session store error.
    #[allow(clippy::needless_pass_by_value)]
    fn to_session_store_error(err: anyhow::Error) -> session_store::Error {
        session_store::Error::Backend(err.to_string())
    }
}

#[async_trait]
impl tower_sessions::SessionStore for SessionStore {
    /// Create a new session record in the database.
    async fn create(&self, record: &mut session::Record) -> session_store::Result<()> {
        self.db
            .create_session(record)
            .await
            .map_err(Self::to_session_store_error)
    }

    /// Save (update) a session record in the database.
    async fn save(&self, record: &session::Record) -> session_store::Result<()> {
        self.db
            .update_session(record)
            .await
            .map_err(Self::to_session_store_error)
    }

    /// Load a session record by session ID from the database.
    async fn load(&self, session_id: &session::Id) -> session_store::Result<Option<session::Record>> {
        self.db
            .get_session(session_id)
            .await
            .map_err(Self::to_session_store_error)
    }

    /// Delete a session record by session ID from the database.
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

/// Backend for authenticating users via `OAuth2`, `Oidc`, or password.
#[derive(Clone)]
pub(crate) struct AuthnBackend {
    db: DynDB,
    http_client: oauth2_reqwest::Client,
    /// Registered `OAuth2` providers.
    pub oauth2_providers: OAuth2Providers,
    /// Registered `Oidc` providers.
    pub oidc_providers: OidcProviders,
}

impl AuthnBackend {
    /// Create a new `AuthnBackend` instance.
    pub async fn new(db: DynDB, oauth2_cfg: &OAuth2Config, oidc_cfg: &OidcConfig) -> Result<Self> {
        let http_client = oauth2_reqwest::ClientBuilder::new()
            .redirect(oauth2_reqwest::redirect::Policy::none())
            .build()?;
        let oauth2_providers = Self::setup_oauth2_providers(oauth2_cfg)?;
        let oidc_providers = Self::setup_oidc_providers(oidc_cfg, http_client.clone()).await?;

        Ok(Self {
            db,
            http_client,
            oauth2_providers,
            oidc_providers,
        })
    }

    /// Authenticate a user using `OAuth2` credentials.
    async fn authenticate_oauth2(&self, creds: OAuth2Credentials) -> Result<Option<User>> {
        // Exchange the authorization code for an access token
        let Some(oauth2_provider) = self.oauth2_providers.get(&creds.provider) else {
            bail!("oauth2 provider not found")
        };
        let access_token = oauth2_provider
            .client
            .exchange_code(oauth2::AuthorizationCode::new(creds.code))
            .request_async(&self.http_client)
            .await?
            .access_token()
            .secret()
            .clone();

        // Get the user if they exist, otherwise sign them up
        let user_summary = match creds.provider {
            OAuth2Provider::GitHub => UserSummary::from_github_profile(&access_token).await?,
        };
        let user = if let Some(user) = self.db.get_user_by_email(&user_summary.email).await? {
            user
        } else {
            let (user, _) = self.db.sign_up_user(&user_summary, true).await?;
            user
        };

        Ok(Some(user))
    }

    /// Authenticate a user using `Oidc` credentials.
    async fn authenticate_oidc(&self, creds: OidcCredentials) -> Result<Option<User>> {
        // Exchange the authorization code for an access and id token
        let Some(oidc_provider) = self.oidc_providers.get(&creds.provider) else {
            bail!("oidc provider not found")
        };
        let token_response = oidc_provider
            .client
            .exchange_code(oidc::AuthorizationCode::new(creds.code))?
            .request_async(&self.http_client)
            .await?;

        // Extract and verify ID token claims.
        let id_token_verifier = oidc_provider.client.id_token_verifier();
        let Some(id_token) = token_response.extra_fields().id_token() else {
            bail!("id token missing")
        };
        let claims = id_token.claims(&id_token_verifier, &creds.nonce)?;

        // Get the user if they exist, otherwise sign them up
        let user_summary = match creds.provider {
            OidcProvider::LinuxFoundation => UserSummary::from_oidc_id_token_claims(claims)?,
        };
        let user = if let Some(user) = self.db.get_user_by_email(&user_summary.email).await? {
            user
        } else {
            let (user, _) = self.db.sign_up_user(&user_summary, true).await?;
            user
        };

        Ok(Some(user))
    }

    /// Authenticate user using password credentials.
    async fn authenticate_password(&self, creds: PasswordCredentials) -> Result<Option<User>> {
        // Get user from database
        let user = self.db.get_user_by_username(&creds.username).await?;

        // Check if the credentials are valid, returning the user if they are
        if let Some(mut user) = user {
            // Check if the user's password is set
            let Some(password_hash) = user.password.clone() else {
                return Ok(None);
            };

            // Verify the password
            if tokio::task::spawn_blocking(move || verify_password(creds.password, &password_hash))
                .await?
                .is_ok()
            {
                user.password = None;
                return Ok(Some(user));
            }
        }

        Ok(None)
    }

    /// Set up `OAuth2` providers from configuration.
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

    /// Set up `Oidc` providers from configuration.
    async fn setup_oidc_providers(
        oidc_cfg: &OidcConfig,
        http_client: oauth2_reqwest::Client,
    ) -> Result<OidcProviders> {
        let mut providers: OidcProviders = HashMap::new();

        for (provider, cfg) in oidc_cfg {
            let issuer_url = oidc::IssuerUrl::new(cfg.issuer_url.clone())?;
            let client = oidc::core::CoreClient::from_provider_metadata(
                oidc::core::CoreProviderMetadata::discover_async(issuer_url, &http_client).await?,
                oidc::ClientId::new(cfg.client_id.clone()),
                Some(oidc::ClientSecret::new(cfg.client_secret.clone())),
            )
            .set_redirect_uri(oidc::RedirectUrl::new(cfg.redirect_uri.clone())?);

            providers.insert(
                provider.clone(),
                Arc::new(OidcProviderDetails {
                    client,
                    scopes: cfg.scopes.clone(),
                }),
            );
        }

        Ok(providers)
    }
}

impl axum_login::AuthnBackend for AuthnBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    /// Authenticate a user using the provided credentials.
    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::OAuth2(creds) => self.authenticate_oauth2(creds).await.map_err(AuthError),
            Credentials::Oidc(creds) => self.authenticate_oidc(creds).await.map_err(AuthError),
            Credentials::Password(creds) => self.authenticate_password(creds).await.map_err(AuthError),
        }
    }

    /// Retrieve a user by user ID from the database.
    async fn get_user(&self, user_id: &axum_login::UserId<Self>) -> Result<Option<Self::User>, Self::Error> {
        self.db.get_user_by_id(user_id).await.map_err(AuthError)
    }
}

/// Type alias for an authentication session using our backend.
pub(crate) type AuthSession = axum_login::AuthSession<AuthnBackend>;

/// Type alias for a map of `OAuth2` providers.
pub(crate) type OAuth2Providers = HashMap<OAuth2Provider, Arc<OAuth2ProviderDetails>>;

/// Details for an `OAuth2` provider, including client and scopes.
#[derive(Clone)]
pub(crate) struct OAuth2ProviderDetails {
    /// `OAuth2` client for this provider.
    pub client: oauth2::basic::BasicClient<
        oauth2::EndpointSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointNotSet,
        oauth2::EndpointSet,
    >,
    /// Scopes requested from the provider.
    pub scopes: Vec<String>,
}

/// Type alias for a map of `Oidc` providers.
pub(crate) type OidcProviders = HashMap<OidcProvider, Arc<OidcProviderDetails>>;

/// Details for an `Oidc` provider, including client and scopes.
#[derive(Clone)]
pub(crate) struct OidcProviderDetails {
    /// `Oidc` client for this provider.
    pub client: oidc::core::CoreClient<
        oidc::EndpointSet,
        oidc::EndpointNotSet,
        oidc::EndpointNotSet,
        oidc::EndpointNotSet,
        oidc::EndpointMaybeSet,
        oidc::EndpointMaybeSet,
    >,
    /// Scopes requested from the provider.
    pub scopes: Vec<String>,
}

/// Wrapper for authentication errors, based on `anyhow::Error`.
#[derive(thiserror::Error, Debug)]
#[error(transparent)]
pub(crate) struct AuthError(#[from] anyhow::Error);

/// Credentials for authenticating a user.
#[derive(Clone, Serialize, Deserialize)]
pub enum Credentials {
    /// `OAuth2` credentials.
    OAuth2(OAuth2Credentials),
    /// `Oidc` credentials.
    Oidc(OidcCredentials),
    /// Username and password credentials.
    Password(PasswordCredentials),
}

/// Credentials for `OAuth2` authentication.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct OAuth2Credentials {
    /// Authorization code from the `OAuth2` provider.
    pub code: String,
    /// The `OAuth2` provider to use.
    pub provider: OAuth2Provider,
}

/// Credentials for `Oidc` authentication.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct OidcCredentials {
    /// Authorization code from the `Oidc` provider.
    pub code: String,
    /// Nonce used for ID token verification.
    pub nonce: oidc::Nonce,
    /// The `Oidc` provider to use.
    pub provider: OidcProvider,
}

/// Credentials for password authentication.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PasswordCredentials {
    /// Username for authentication.
    pub username: String,
    /// Password for authentication.
    pub password: String,
}

// User types and implementations.

/// Represents a user in the system.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct User {
    /// Unique user ID.
    pub user_id: Uuid,
    /// Authentication hash for session validation.
    pub auth_hash: Vec<u8>,
    /// User's email address.
    pub email: String,
    /// Whether the user's email is verified.
    pub email_verified: bool,
    /// Whether the user has a profile.
    pub has_profile: bool,
    /// Whether the user is a moderator.
    pub moderator: bool,
    /// User's display name.
    pub name: String,
    /// User's username.
    pub username: String,

    /// Whether the user has a password set.
    pub has_password: Option<bool>,
    /// User's password hash (if present).
    pub password: Option<String>,
}

impl axum_login::AuthUser for User {
    type Id = Uuid;

    /// Get the user's unique ID.
    fn id(&self) -> Self::Id {
        self.user_id
    }

    /// Get the session authentication hash.
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

/// Summary of user information.
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize, Validate)]
pub(crate) struct UserSummary {
    /// User's email address.
    #[garde(email, length(max = MAX_LEN_M))]
    pub email: String,
    /// User's display name.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_DISPLAY_NAME))]
    pub name: String,
    /// User's username.
    #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_S))]
    pub username: String,

    /// Whether the user has a password set.
    #[garde(skip)]
    pub has_password: Option<bool>,
    /// User's password (if present).
    #[garde(custom(trimmed_non_empty_opt), length(min = MIN_PASSWORD_LEN, max = MAX_LEN_S))]
    pub password: Option<String>,
}

impl UserSummary {
    /// Create a `UserSummary` instance from a GitHub profile.
    async fn from_github_profile(access_token: &str) -> Result<Self> {
        // Setup headers for GitHub API requests.
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "gitjobs".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {access_token}").as_str().parse()?);

        // Get user profile from GitHub.
        let profile = reqwest::Client::new()
            .get("https://api.github.com/user")
            .headers(headers.clone())
            .send()
            .await?
            .json::<GitHubProfile>()
            .await?;

        // Get user emails from GitHub.
        let emails = reqwest::Client::new()
            .get("https://api.github.com/user/emails")
            .headers(headers)
            .send()
            .await?
            .json::<Vec<GitHubUserEmail>>()
            .await?;

        // Get primary, verified email.
        let email = emails
            .into_iter()
            .find(|email| email.primary && email.verified)
            .ok_or_else(|| anyhow!("no valid email found (primary email must be verified)"))?;

        Ok(Self {
            email: email.email,
            name: profile.name,
            username: profile.login,
            has_password: Some(false),
            password: None,
        })
    }

    /// Create a `UserSummary` from `Oidc` Id token claims.
    fn from_oidc_id_token_claims(
        claims: &oidc::IdTokenClaims<oidc::EmptyAdditionalClaims, oidc::core::CoreGenderClaim>,
    ) -> Result<Self> {
        // Ensure email is verified and extract user info.
        if !claims.email_verified().unwrap_or(false) {
            bail!("email not verified");
        }

        let email = claims.email().ok_or_else(|| anyhow!("email missing"))?.to_string();
        let name = get_localized_claim(claims.name()).ok_or_else(|| anyhow!("name missing"))?;
        let username = get_localized_claim(claims.nickname()).ok_or_else(|| anyhow!("nickname missing"))?;

        Ok(Self {
            email,
            name: name.to_string(),
            username: username.to_string(),
            has_password: Some(false),
            password: None,
        })
    }
}

impl From<User> for UserSummary {
    /// Convert a `User` into a `UserSummary`.
    fn from(user: User) -> Self {
        Self {
            email: user.email,
            name: user.name,
            username: user.username,
            has_password: user.has_password,
            password: None,
        }
    }
}

impl std::fmt::Debug for UserSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UserSummary")
            .field("email", &self.email)
            .field("name", &self.name)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

/// Get the first value from a localized claim, if present.
fn get_localized_claim<T>(claim: Option<&LocalizedClaim<T>>) -> Option<T>
where
    T: Clone,
{
    claim.and_then(|v| {
        if let Some((_, v)) = v.iter().next() {
            Some((*v).clone())
        } else {
            None
        }
    })
}

/// GitHub user profile information.
#[derive(Debug, Deserialize)]
struct GitHubProfile {
    /// GitHub username.
    login: String,
    /// GitHub display name.
    name: String,
}

/// GitHub user email information.
#[derive(Debug, Deserialize)]
struct GitHubUserEmail {
    /// Email address.
    email: String,
    /// Whether this is the primary email.
    primary: bool,
    /// Whether this email is verified.
    verified: bool,
}
