//! This module defines some types and functionality to manage and send
//! notifications.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tracing::instrument;
use uuid::Uuid;

use crate::db::DynDB;

/// Abstraction layer over the notifications manager. Trait that defines some
/// operations a notifications manager implementation must support.
#[async_trait]
pub(crate) trait NotificationsManager {
    /// Enqueue a notification to be sent.
    async fn enqueue(&self, notification: &Notification) -> Result<()>;
}

/// Type alias to represent a notifications manager trait object.
pub(crate) type DynNotificationsManager = Arc<dyn NotificationsManager + Send + Sync>;

/// Notifications manager backed by `PostgreSQL`.
pub(crate) struct PgNotificationsManager {
    db: DynDB,
}

impl PgNotificationsManager {
    /// Create a new notifications `Manager` instance.
    pub fn new(db: DynDB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl NotificationsManager for PgNotificationsManager {
    /// [NotificationsManager::enqueue_notification]
    #[instrument(skip(self), err)]
    async fn enqueue(&self, notification: &Notification) -> Result<()> {
        self.db.enqueue_notification(notification).await
    }
}

/// Notification.
#[derive(Debug, Clone)]
pub struct Notification {
    pub kind: NotificationKind,
    pub user_id: Uuid,

    pub template_data: Option<serde_json::Value>,
}

/// Notification kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum NotificationKind {
    EmailVerification,
}

impl std::fmt::Display for NotificationKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NotificationKind::EmailVerification => write!(f, "email-verification"),
        }
    }
}

impl TryFrom<&str> for NotificationKind {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "email-verification" => Ok(Self::EmailVerification),
            _ => Err(anyhow::Error::msg("invalid notification kind")),
        }
    }
}
