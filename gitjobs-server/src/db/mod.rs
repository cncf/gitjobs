//! This module defines an abstraction layer over the database.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use auth::DBAuth;
use dashboard::DBDashBoard;
use deadpool_postgres::{Client, Pool};
use img::DBImage;
use notifications::DBNotifications;
use std::time::{Duration, SystemTime};
use tokio::select;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use tracing::instrument;
use uuid::Uuid;

use crate::templates::misc::{Location, Member, Project};

mod auth;
mod dashboard;
pub(crate) mod img;
mod notifications;

/// Abstraction layer over the database. Trait that defines some operations a
/// DB implementation must support.
#[async_trait]
pub(crate) trait DB: DBAuth + DBDashBoard + DBImage + DBNotifications {
    /// Get the job board id from the host provided.
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>>;

    /// Search locations.
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>>;

    /// Search members.
    async fn search_members(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Member>>;

    /// Search projects.
    async fn search_projects(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Project>>;

    /// Begin transaction.
    async fn tx_begin(&self) -> Result<Uuid>;

    /// Commit transaction.
    async fn tx_commit(&self, conn_id: Uuid) -> Result<()>;

    /// Rollback transaction.
    async fn tx_rollback(&self, conn_id: Uuid) -> Result<()>;

    /// Discard transactions that have been active too long.
    async fn txs_cleaner(&self, token: CancellationToken);
}

/// Type alias to represent a DB trait object.
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

/// DB implementation backed by `PostgreSQL`.
pub(crate) struct PgDB {
    pool: Pool,
    txs_conns: Arc<RwLock<HashMap<Uuid, (Client, SystemTime)>>>,
}

impl PgDB {
    /// Create a new `PgDB` instance.
    pub(crate) fn new(pool: Pool) -> Self {
        let txs_conns = Arc::new(RwLock::new(HashMap::new()));
        Self { pool, txs_conns }
    }
}

#[async_trait]
impl DB for PgDB {
    /// [DB::get_job_board_id]
    #[instrument(skip(self), err)]
    async fn get_job_board_id(&self, host: &str) -> Result<Option<Uuid>> {
        let db = self.pool.get().await?;
        let job_board_id = db
            .query_opt(
                "
                select job_board_id
                from job_board
                where host = $1::text
                and active = true
                ",
                &[&host],
            )
            .await?
            .map(|row| row.get("job_board_id"));

        Ok(job_board_id)
    }

    /// [DB::search_locations]
    #[instrument(skip(self), err)]
    async fn search_locations(&self, ts_query: &str) -> Result<Vec<Location>> {
        let db = self.pool.get().await?;
        let locations = db
            .query(
                "
                select
                    location_id,
                    city,
                    country,
                    state
                from search_locations($1::text)
                ",
                &[&ts_query],
            )
            .await?
            .into_iter()
            .map(|row| Location {
                location_id: row.get("location_id"),
                city: row.get("city"),
                country: row.get("country"),
                state: row.get("state"),
            })
            .collect();

        Ok(locations)
    }

    /// [DB::search_members]
    #[instrument(skip(self), err)]
    async fn search_members(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Member>> {
        let db = self.pool.get().await?;
        let members = db
            .query(
                "
                select
                    member_id,
                    name,
                    level,
                    logo_url
                from member
                where job_board_id = $1::uuid
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&job_board_id, &name],
            )
            .await?
            .into_iter()
            .map(|row| Member {
                member_id: row.get("member_id"),
                name: row.get("name"),
                level: row.get("level"),
                logo_url: row.get("logo_url"),
            })
            .collect();

        Ok(members)
    }

    /// [DB::search_projects]
    #[instrument(skip(self), err)]
    async fn search_projects(&self, job_board_id: &Uuid, name: &str) -> Result<Vec<Project>> {
        let db = self.pool.get().await?;
        let projects = db
            .query(
                "
                select
                    project_id,
                    name,
                    maturity,
                    logo_url
                from project
                where job_board_id = $1::uuid
                and name ilike '%' || $2::text || '%'
                limit 20;
                ",
                &[&job_board_id, &name],
            )
            .await?
            .into_iter()
            .map(|row| Project {
                project_id: row.get("project_id"),
                name: row.get("name"),
                maturity: row.get("maturity"),
                logo_url: row.get("logo_url"),
            })
            .collect();

        Ok(projects)
    }

    /// [DB::tx_begin]
    #[instrument(skip(self), err)]
    async fn tx_begin(&self) -> Result<Uuid> {
        let tx = self.pool.get().await?;
        tx.batch_execute("BEGIN").await?;
        let conn_id = Uuid::new_v4();
        let mut txs_conns = self.txs_conns.write().await;
        txs_conns.insert(conn_id, (tx, SystemTime::now()));

        Ok(conn_id)
    }

    /// [DB::tx_commit]
    #[instrument(skip(self), err)]
    async fn tx_commit(&self, conn_id: Uuid) -> Result<()> {
        let mut txs_conns = self.txs_conns.write().await;
        let (tx, _) = txs_conns.remove(&conn_id).unwrap();

        tx.batch_execute("COMMIT").await?;

        Ok(())
    }

    /// [DB::tx_rollback]
    #[instrument(skip(self), err)]
    async fn tx_rollback(&self, conn_id: Uuid) -> Result<()> {
        let mut txs_conns = self.txs_conns.write().await;
        let (tx, _) = txs_conns.remove(&conn_id).unwrap();

        tx.batch_execute("ROLLBACK").await?;

        Ok(())
    }

    /// [DB::txs_cleaner]
    async fn txs_cleaner(&self, token: CancellationToken) {
        let clients = self.txs_conns.clone();
        tokio::spawn(async move {
            loop {
                if token.is_cancelled() {
                    break;
                }
                select! {
                    () = token.cancelled() => break,
                    () = tokio::time::sleep(std::time::Duration::from_secs(5)) => {}
                };

                let clients_reader = clients.read().await;
                let mut clients_to_discard: Vec<Uuid> = vec![];
                let max_time = Duration::from_secs(5);

                for (id, (_, ts)) in clients_reader.iter() {
                    if ts.elapsed().unwrap() >= max_time {
                        clients_to_discard.push(*id);
                    }
                }

                if !clients_to_discard.is_empty() {
                    let mut clients_writer = clients.write().await;

                    for id in clients_to_discard {
                        let _ = clients_writer.remove(&id).unwrap();
                    }
                }
            }
        });
    }
}
