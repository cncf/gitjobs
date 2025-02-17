//! This module defines some types to represent the server configuration.

use std::{collections::HashMap, path::PathBuf};

use anyhow::Result;
use deadpool_postgres::Config as DbConfig;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use tracing::instrument;

use crate::auth::OAuth2Provider;

/// Server configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub(crate) struct Config {
    pub db: DbConfig,
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

/// Http server configuration.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub(crate) struct HttpServerConfig {
    pub addr: String,
    pub basic_auth: Option<BasicAuth>,
    pub cookie: Option<CookieConfig>,
    pub oauth2: OAuth2Config,
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

/// Type alias for the `OAuth2` configuration section.
pub(crate) type OAuth2Config = HashMap<OAuth2Provider, OAuth2ProviderConfig>;

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
