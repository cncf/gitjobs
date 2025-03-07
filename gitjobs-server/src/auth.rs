//! This module contains the authentication and authorization functionality.

use std::{collections::HashMap, sync::Arc};

use anyhow::{Result, anyhow, bail};
use async_trait::async_trait;
use axum::http::header::{AUTHORIZATION, USER_AGENT};
use axum_login::{
    AuthManagerLayer, AuthManagerLayerBuilder,
    tower_sessions::{self, session, session_store},
};
use oauth2::{TokenResponse, reqwest};
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
};

/// Type alias for the auth layer.
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
    pub oidc_providers: OidcProviders,
}

impl AuthnBackend {
    /// Create a new `AuthnBackend` instance.
    pub async fn new(db: DynDB, oauth2_cfg: &OAuth2Config, oidc_cfg: &OidcConfig) -> Result<Self> {
        let http_client = reqwest::ClientBuilder::new()
            .redirect(reqwest::redirect::Policy::none())
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

    /// Authenticate user using `OAuth2` credentials.
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
            .to_string();

        // Get the user if they exist, otherwise sign them up
        let user_summary = match creds.provider {
            OAuth2Provider::GitHub => UserSummary::from_github_profile(&access_token).await?,
        };
        let user = if let Some(user) = self
            .db
            .get_user_by_email(&creds.job_board_id, &user_summary.email)
            .await?
        {
            user
        } else {
            let (user, _) = self.db.sign_up_user(&creds.job_board_id, &user_summary, true).await?;
            user
        };

        Ok(Some(user))
    }

    /// Authenticate user using `Oidc` credentials.
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

        // Extract and verify id token claims
        let id_token_verifier = oidc_provider.client.id_token_verifier();
        let Some(id_token) = token_response.extra_fields().id_token() else {
            bail!("id token missing")
        };
        let claims = id_token.claims(&id_token_verifier, &creds.nonce)?;

        // Get the user if they exist, otherwise sign them up
        let user_summary = match creds.provider {
            OidcProvider::LinuxFoundation => UserSummary::from_oidc_id_token_claims(claims)?,
        };
        let user = if let Some(user) = self
            .db
            .get_user_by_email(&creds.job_board_id, &user_summary.email)
            .await?
        {
            user
        } else {
            let (user, _) = self.db.sign_up_user(&creds.job_board_id, &user_summary, true).await?;
            user
        };

        Ok(Some(user))
    }

    /// Authenticate user using password credentials.
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

    /// Setup `Oidc` providers.
    async fn setup_oidc_providers(
        oidc_cfg: &OidcConfig,
        http_client: reqwest::Client,
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

#[async_trait]
impl axum_login::AuthnBackend for AuthnBackend {
    type User = User;
    type Credentials = Credentials;
    type Error = AuthError;

    async fn authenticate(&self, creds: Self::Credentials) -> Result<Option<Self::User>, Self::Error> {
        match creds {
            Credentials::OAuth2(creds) => self.authenticate_oauth2(creds).await.map_err(AuthError),
            Credentials::Oidc(creds) => self.authenticate_oidc(creds).await.map_err(AuthError),
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

/// Type alias for the structure that holds the `Oidc` providers.
pub(crate) type OidcProviders = HashMap<OidcProvider, Arc<OidcProviderDetails>>;

/// `Oidc` provider client and scopes.
#[derive(Clone)]
pub(crate) struct OidcProviderDetails {
    pub client: oidc::core::CoreClient<
        oidc::EndpointSet,
        oidc::EndpointNotSet,
        oidc::EndpointNotSet,
        oidc::EndpointNotSet,
        oidc::EndpointMaybeSet,
        oidc::EndpointMaybeSet,
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
    Oidc(OidcCredentials),
    Password(PasswordCredentials),
}

/// `OAuth2` credentials.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct OAuth2Credentials {
    pub code: String,
    pub job_board_id: Uuid,
    pub provider: OAuth2Provider,
}

/// `Oidc` credentials.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct OidcCredentials {
    pub code: String,
    pub job_board_id: Uuid,
    pub nonce: oidc::Nonce,
    pub provider: OidcProvider,
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
#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names, dead_code)]
pub(crate) struct User {
    pub user_id: Uuid,
    pub auth_hash: Vec<u8>,
    pub email: String,
    pub email_verified: bool,
    pub name: String,
    pub username: String,

    pub has_password: Option<bool>,
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

/// User information summary.
#[skip_serializing_none]
#[derive(Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct UserSummary {
    pub email: String,
    pub name: String,
    pub username: String,

    pub has_password: Option<bool>,
    pub password: Option<String>,
}

impl UserSummary {
    /// Create a `UserSummary` instance from a GitHub profile.
    async fn from_github_profile(access_token: &str) -> Result<Self> {
        // Setup headers for GitHub API requests
        let mut headers = HeaderMap::new();
        headers.insert(USER_AGENT, "open-job-board".parse()?);
        headers.insert(AUTHORIZATION, format!("Bearer {access_token}").as_str().parse()?);

        // Get user profile from GitHub
        let profile = reqwest::Client::new()
            .get("https://api.github.com/user")
            .headers(headers.clone())
            .send()
            .await?
            .json::<GitHubProfile>()
            .await?;

        // Get user emails from GitHub
        let emails = reqwest::Client::new()
            .get("https://api.github.com/user/emails")
            .headers(headers.clone())
            .send()
            .await?
            .json::<Vec<GitHubUserEmail>>()
            .await?;

        // Get primary email and check it is verified
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

    /// Create a `UserSummary` instance from an oidc id token claims.
    fn from_oidc_id_token_claims(
        claims: &oidc::IdTokenClaims<oidc::EmptyAdditionalClaims, oidc::core::CoreGenderClaim>,
    ) -> Result<Self> {
        // Check if the email is verified
        if !claims.email_verified().unwrap_or(false) {
            bail!("email not verified");
        }

        // Collect some information from the claims
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

/// Get the first localized claim value.
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

/// GitHub profile information.
#[derive(Debug, Deserialize)]
struct GitHubProfile {
    login: String,
    name: String,
}

/// GitHub user email.
#[derive(Debug, Deserialize)]
struct GitHubUserEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// User password update input.
#[derive(Clone, Serialize, Deserialize)]
pub(crate) struct PasswordUpdateInput {
    pub old_password: String,
    pub new_password: String,
}
