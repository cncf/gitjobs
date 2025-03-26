//! This module defines an abstraction layer over the database.

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::{Result, bail};
use async_trait::async_trait;
use auth::DBAuth;
use chrono::{DateTime, TimeDelta, Utc};
use dashboard::DBDashBoard;
use deadpool_postgres::{Client, Pool};
use img::DBImage;
use jobboard::DBJobBoard;
use misc::DBMisc;
use notifications::DBNotifications;
use tokio::sync::RwLock;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use uuid::Uuid;
use workers::DBWorkers;

pub(crate) mod auth;
pub(crate) mod dashboard;
pub(crate) mod img;
pub(crate) mod jobboard;
pub(crate) mod misc;
pub(crate) mod notifications;
pub(crate) mod workers;

/// Error message when a transaction client is not found.
const TX_CLIENT_NOT_FOUND: &str = "transaction client not found, it probably timed out";

/// How often the transaction cleaner should run (in seconds).
const TXS_CLEANER_FREQUENCY: Duration = Duration::from_secs(10);

/// How long a transaction client should be kept alive.
const TXS_CLIENT_TIMEOUT: TimeDelta = TimeDelta::seconds(10);

/// Abstraction layer over the database. Trait that defines some operations a
/// DB implementation must support.
#[async_trait]
pub(crate) trait DB:
    DBJobBoard + DBDashBoard + DBAuth + DBImage + DBNotifications + DBWorkers + DBMisc
{
    /// Begin transaction.
    async fn tx_begin(&self) -> Result<Uuid>;

    /// Commit transaction.
    async fn tx_commit(&self, client_id: Uuid) -> Result<()>;

    /// Rollback transaction.
    async fn tx_rollback(&self, client_id: Uuid) -> Result<()>;
}

/// Type alias to represent a DB trait object.
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

/// DB implementation backed by `PostgreSQL`.
pub(crate) struct PgDB {
    pool: Pool,
    txs_clients: RwLock<HashMap<Uuid, (Client, DateTime<Utc>)>>,
}

impl PgDB {
    /// Create a new `PgDB` instance.
    pub(crate) fn new(pool: Pool) -> Self {
        Self {
            pool,
            txs_clients: RwLock::new(HashMap::new()),
        }
    }

    /// Process that cleans up transactions clients that have timed out.
    pub(crate) async fn tx_cleaner(&self, cancellation_token: CancellationToken) {
        loop {
            // Check if we've been asked to stop or pause until next run
            select! {
                () = cancellation_token.cancelled() => break,
                () = sleep(TXS_CLEANER_FREQUENCY) => {}
            };

            // Collect timed out clients to discard
            let clients_reader = self.txs_clients.read().await;
            let mut clients_to_discard: Vec<Uuid> = vec![];
            for (id, (_, ts)) in clients_reader.iter() {
                if Utc::now() - ts > TXS_CLIENT_TIMEOUT {
                    clients_to_discard.push(*id);
                }
            }
            drop(clients_reader);

            // Discard timed out clients
            if !clients_to_discard.is_empty() {
                let mut clients_writer = self.txs_clients.write().await;
                for id in clients_to_discard {
                    clients_writer.remove(&id);
                }
            }
        }
    }
}

#[async_trait]
impl DB for PgDB {
    #[instrument(skip(self), err)]
    async fn tx_begin(&self) -> Result<Uuid> {
        // Get client from pool and begin transaction
        let db = self.pool.get().await?;
        db.batch_execute("begin;").await?;

        // Track client used for the transaction
        let client_id = Uuid::new_v4();
        let mut txs_clients = self.txs_clients.write().await;
        txs_clients.insert(client_id, (db, Utc::now()));

        Ok(client_id)
    }

    #[instrument(skip(self), err)]
    async fn tx_commit(&self, client_id: Uuid) -> Result<()> {
        // Get client used for the transaction
        let mut txs_clients = self.txs_clients.write().await;
        let Some((tx, _)) = txs_clients.remove(&client_id) else {
            bail!(TX_CLIENT_NOT_FOUND);
        };

        // Commit transaction
        tx.batch_execute("commit;").await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn tx_rollback(&self, client_id: Uuid) -> Result<()> {
        // Get client used for the transaction
        let mut txs_clients = self.txs_clients.write().await;
        let Some((tx, _)) = txs_clients.remove(&client_id) else {
            bail!(TX_CLIENT_NOT_FOUND);
        };

        // Rollback transaction
        tx.batch_execute("rollback;").await?;

        Ok(())
    }
}
