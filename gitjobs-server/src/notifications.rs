//! This module defines some types and functionality to manage and send
//! notifications.

use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use async_trait::async_trait;
use mail_builder::MessageBuilder;
use mail_send::SmtpClientBuilder;
use rinja::Template;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::config::EmailConfig;
use crate::db::DynDB;
use crate::templates::notifications::EmailVerification;

/// Number of workers that will be processing notifications
const PROCESS_NOTIFICATIONS_WORKERS: usize = 5;
/// Amount of time the votes closer will sleep when there is an error
/// processing the notification.
const PROCESS_NOTIFICATION_PAUSE_ON_ERROR: Duration = Duration::from_secs(5);
/// Amount of time the votes closer will sleep when there are no pending votes
/// to close.
const PROCESS_NOTIFICATIONS_PAUSE_ON_NONE: Duration = Duration::from_secs(5);

/// Abstraction layer over the notifications manager. Trait that defines some
/// operations a notifications manager implementation must support.
#[async_trait]
pub(crate) trait NotificationsManager {
    /// Enqueue a notification to be sent.
    async fn enqueue(&self, notification: &NewNotification) -> Result<()>;

    /// Run notifications manager
    async fn run(self: Arc<Self>);
}

/// Type alias to represent a notifications manager trait object.
pub(crate) type DynNotificationsManager = Arc<dyn NotificationsManager + Send + Sync>;

/// Notifications manager backed by `PostgreSQL`.
pub(crate) struct PgNotificationsManager {
    db: DynDB,
    config: EmailConfig,
    tracker: TaskTracker,
    token: CancellationToken,
}

impl PgNotificationsManager {
    /// Create a new notifications `Manager` instance.
    pub fn new(db: DynDB, config: EmailConfig, tracker: TaskTracker, token: CancellationToken) -> Self {
        Self {
            db,
            config,
            tracker,
            token,
        }
    }
}

#[async_trait]
impl NotificationsManager for PgNotificationsManager {
    /// [NotificationsManager::enqueue_notification]
    #[instrument(skip(self), err)]
    async fn enqueue(&self, notification: &NewNotification) -> Result<()> {
        self.db.enqueue_notification(notification).await
    }

    /// [NotificationsManager::run]
    async fn run(self: Arc<Self>) {
        // Call transactions cleaner
        self.db.txs_cleaner(self.token.clone()).await;

        // Spawn workers to process notifications
        for _ in 1..=PROCESS_NOTIFICATIONS_WORKERS {
            let worker = Worker {
                db: self.db.clone(),
                config: self.config.clone(),
                token: self.token.clone(),
            };

            self.tracker.spawn(async move {
                let _ = worker.run().await;
            });
        }
    }
}

pub struct Worker {
    db: DynDB,
    config: EmailConfig,
    token: CancellationToken,
}

impl Worker {
    pub async fn run(&self) -> Result<()> {
        loop {
            // Process notifications
            match self.process_notification().await {
                Ok(Some(_)) => {
                    // One notification was processed, try to process another
                    // one immediately
                }
                Ok(None) => tokio::select! {
                    // No notifications to process, pause unless we've been
                    // asked to stop
                    () = sleep(PROCESS_NOTIFICATIONS_PAUSE_ON_NONE) => {},
                    () = self.token.cancelled() => break,
                },
                Err(_) => {
                    // Something went wrong processing the notification, pause
                    // unless we've been asked to stop
                    tokio::select! {
                        () = sleep(PROCESS_NOTIFICATION_PAUSE_ON_ERROR) => {},
                        () = self.token.cancelled() => break,
                    }
                }
            }

            // Exit if the worker has been asked to stop
            if self.token.is_cancelled() {
                break;
            }
        }

        Ok(())
    }

    pub async fn process_notification(&self) -> Result<Option<Notification>> {
        // Start transaction
        let tx_id = self.db.tx_begin().await?;

        // Get a notification
        let notification = match self.db.get_pending_notification(tx_id).await {
            Ok(notification) => notification,
            Err(err) => {
                self.db.tx_rollback(tx_id).await?;
                return Err(err);
            }
        };

        // Send the notification
        if let Some(notification) = &notification {
            let (subject, body) = match notification.kind {
                NotificationKind::EmailVerification => {
                    let template: EmailVerification =
                        serde_json::from_value(notification.template_data.clone().expect("to be some"))?;

                    let subject = "Email verification";
                    let body = template.render()?;

                    (subject, body)
                }
            };

            let error = match send_email(&self.config, &notification.email, &body, subject).await {
                Err(err) => Some(err.to_string()),
                Ok(()) => None,
            };

            if let Err(err) = self.db.update_notification(tx_id, notification, error).await {
                error!("error updating notification: {err}");
            }
        }

        // End transaction
        self.db.tx_commit(tx_id).await?;

        Ok(notification)
    }
}

/// Notification.
#[derive(Debug, Clone)]
pub struct Notification {
    pub email: String,
    pub kind: NotificationKind,

    pub id: Option<Uuid>,
    pub template_data: Option<serde_json::Value>,
}

/// New notification.
#[derive(Debug, Clone)]
pub struct NewNotification {
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

async fn send_email(config: &EmailConfig, email: &str, body: &str, subject: &str) -> Result<()> {
    let message = MessageBuilder::new()
        .from(("", "tests@gitjobs.dev"))
        .to(("", email))
        .subject(subject)
        .html_body(body);

    SmtpClientBuilder::new(config.host.as_str(), config.port)
        .implicit_tls(false)
        .credentials((config.smtp_user_name.as_str(), config.smtp_password.as_str()))
        .connect()
        .await?
        .send(message)
        .await?;

    Ok(())
}
