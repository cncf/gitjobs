//! This module defines some types to represent the syncer configuration.

use std::path::PathBuf;

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
    pub log: LogConfig,
}

impl Config {
    /// Create a new Config instance.
    #[instrument(err)]
    pub(crate) fn new(config_file: Option<&PathBuf>) -> Result<Self> {
        let mut figment = Figment::new().merge(Serialized::default("log.format", "json"));

        if let Some(config_file) = config_file {
            figment = figment.merge(Yaml::file(config_file));
        }

        figment
            .merge(Env::prefixed("GITJOBS_").split("__"))
            .extract()
            .map_err(Into::into)
    }
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
