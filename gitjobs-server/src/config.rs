//! This module defines some types to represent the server configuration.

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use deadpool_postgres::Config as DbConfig;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Yaml},
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

/// Server configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Config {
    pub db: DbConfig,
    pub email: EmailConfig,
    pub log: LogConfig,
    pub server: HttpServerConfig,
}

impl Config {
    /// Create a new Config instance.
    #[instrument(err)]
    pub(crate) fn new(config_file: Option<&PathBuf>) -> Result<Self> {
        let mut figment = Figment::new()
            .merge(Serialized::default("log.format", "json"))
            .merge(Serialized::default("server.addr", "127.0.0.1:9000"));

        if let Some(config_file) = config_file {
            figment = figment.merge(Yaml::file(config_file));
        }

        figment
            .merge(Env::prefixed("GITJOBS_").split("__"))
            .extract()
            .map_err(Into::into)
    }
}

/// Email configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct EmailConfig {
    pub from_address: String,
    pub from_name: String,
    pub smtp: SmtpConfig,
}

/// SMTP configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

/// Logs configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct LogConfig {
    pub format: LogFormat,
}

/// Format to use in logs.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LogFormat {
    Json,
    Pretty,
}

/// Http server configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct HttpServerConfig {
    pub addr: String,
    pub base_url: String,
    pub login: LoginOptions,
    pub oauth2: OAuth2Config,
    pub oidc: OidcConfig,

    pub analytics: Option<AnalyticsConfig>,
    pub basic_auth: Option<BasicAuth>,
    pub cookie: Option<CookieConfig>,
}

/// Analytics configuration.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub(crate) struct AnalyticsConfig {
    pub google_tag_id: Option<String>,
    pub osano_script_url: Option<String>,
}

/// Basic authentication configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct BasicAuth {
    pub enabled: bool,
    pub username: String,
    pub password: String,
}

/// Cookie configuration.
#[derive(Debug, Clone, Default, PartialEq, Deserialize, Serialize)]
pub(crate) struct CookieConfig {
    pub secure: Option<bool>,
}

/// Login options enabled.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) struct LoginOptions {
    pub email: bool,
    pub github: bool,
    pub linuxfoundation: bool,
}

/// Type alias for the `OAuth2` configuration section.
pub(crate) type OAuth2Config = HashMap<OAuth2Provider, OAuth2ProviderConfig>;

/// Supported `OAuth2` providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OAuth2Provider {
    GitHub,
}

/// `OAuth2` provider configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OAuth2ProviderConfig {
    pub auth_url: String,
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub token_url: String,
}

/// Type alias for the `Oidc` configuration section.
pub(crate) type OidcConfig = HashMap<OidcProvider, OidcProviderConfig>;

/// Supported `Oidc` providers.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub(crate) enum OidcProvider {
    LinuxFoundation,
}

/// `Oidc` provider configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct OidcProviderConfig {
    pub client_id: String,
    pub client_secret: String,
    pub issuer_url: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
}
