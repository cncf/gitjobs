#![warn(clippy::all, clippy::pedantic)]
#![allow(clippy::cmp_owned)]

use std::{path::PathBuf, sync::Arc};

use anyhow::{Context, Result};
use clap::Parser;
use deadpool_postgres::Runtime;
use img::db::DbImageStore;
use notifications::PgNotificationsManager;
use openssl::ssl::{SslConnector, SslMethod, SslVerifyMode};
use postgres_openssl::MakeTlsConnector;
use tokio::{net::TcpListener, signal};
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crate::{
    config::{Config, LogFormat},
    db::PgDB,
};

mod auth;
mod config;
mod db;
mod handlers;
mod img;
mod notifications;
mod router;
mod templates;

#[derive(Debug, Parser)]
#[clap(author, version, about)]
struct Args {
    /// Config file path
    #[clap(short, long)]
    config_file: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup configuration
    let args = Args::parse();
    let cfg = Config::new(args.config_file.as_ref()).context("error setting up configuration")?;

    // Setup logging
    let ts = tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| {
            format!(
                "{}=trace,axum_login=debug,tower_sessions=debug",
                env!("CARGO_CRATE_NAME")
            )
            .into()
        }))
        .with_file(true)
        .with_line_number(true);
    match cfg.log.format {
        LogFormat::Json => ts.json().init(),
        LogFormat::Pretty => ts.init(),
    };

    // Setup task tracker and cancellation token
    let tracker = TaskTracker::new();
    let cancellation_token = CancellationToken::new();

    // Setup database
    let mut builder = SslConnector::builder(SslMethod::tls())?;
    builder.set_verify(SslVerifyMode::NONE);
    let connector = MakeTlsConnector::new(builder.build());
    let pool = cfg.db.create_pool(Some(Runtime::Tokio1), connector)?;
    let db = Arc::new(PgDB::new(pool));
    {
        let db = db.clone();
        let cancellation_token = cancellation_token.clone();
        tracker.spawn(async move {
            db.tx_cleaner(cancellation_token).await;
        });
    }

    // Setup image store
    let image_store = Arc::new(DbImageStore::new(db.clone()));

    // Setup and launch notifications manager
    let notifications_manager = Arc::new(PgNotificationsManager::new(
        db.clone(),
        cfg.email,
        tracker.clone(),
        cancellation_token.clone(),
    )?);

    // Setup and launch HTTP server
    let router = router::setup(cfg.server.clone(), db, image_store, notifications_manager).await?;
    let listener = TcpListener::bind(&cfg.server.addr).await?;
    info!("server started");
    info!(%cfg.server.addr, "listening");
    if let Err(err) = axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
    {
        error!(?err, "server error");
        return Err(err.into());
    }
    info!("server stopped");

    // Ask all tasks to stop and wait for them to finish
    tracker.close();
    cancellation_token.cancel();
    tracker.wait().await;

    Ok(())
}

/// Return a future that will complete when the program is asked to stop via a
/// ctrl+c or terminate signal.
async fn shutdown_signal() {
    // Setup signal handlers
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install ctrl+c signal handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install terminate signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // Wait for any of the signals
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
