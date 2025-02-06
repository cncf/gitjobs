//! This module defines an abstraction layer over the database.

use std::sync::Arc;

use async_trait::async_trait;
use auth::DBAuth;
use common::DBCommon;
use dashboard::DBDashBoard;
use deadpool_postgres::Pool;

mod auth;
mod common;
mod dashboard;

/// Abstraction layer over the database. Trait that defines some operations a
/// DB implementation must support.
#[async_trait]
pub(crate) trait DB: DBAuth + DBCommon + DBDashBoard {}

/// Type alias to represent a DB trait object.
pub(crate) type DynDB = Arc<dyn DB + Send + Sync>;

/// DB implementation backed by `PostgreSQL`.
pub(crate) struct PgDB {
    pool: Pool,
}

impl PgDB {
    /// Create a new `PgDB` instance.
    pub(crate) fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

impl DB for PgDB {}
