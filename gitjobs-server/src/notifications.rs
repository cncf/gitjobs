//! This module defines some types and functionality to manage and send
//! notifications.

use std::{sync::Arc, time::Duration};

use anyhow::Result;
use async_trait::async_trait;
use lettre::{
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
    message::{Mailbox, MessageBuilder, header::ContentType},
    transport::smtp::authentication::Credentials,
};
use rinja::Template;
use serde::{Deserialize, Serialize};
use tokio::time::sleep;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{config::EmailConfig, db::DynDB, templates::notifications::EmailVerification};

/// Number of workers to deliver notifications.
const NUM_WORKERS: usize = 2;

/// Amount of time to sleep when there is an error delivering a notification.
const PAUSE_ON_ERROR: Duration = Duration::from_secs(30);

/// Amount of time to sleep when there are no notifications to deliver.
const PAUSE_ON_NONE: Duration = Duration::from_secs(15);

/// Abstraction layer over the notifications manager. This trait that defines
/// some operations a notifications manager implementation must support.
///
/// A notifications manager is in charge of delivering notifications to users.
#[async_trait]
pub(crate) trait NotificationsManager {
    /// Enqueue a notification to be sent.
    async fn enqueue(&self, notification: &NewNotification) -> Result<()>;
}

/// Type alias to represent a notifications manager trait object.
pub(crate) type DynNotificationsManager = Arc<dyn NotificationsManager + Send + Sync>;

/// Notifications manager backed by `PostgreSQL`.
pub(crate) struct PgNotificationsManager {
    db: DynDB,
    cfg: EmailConfig,
    tracker: TaskTracker,
    cancellation_token: CancellationToken,
}

impl PgNotificationsManager {
    /// Create a new `PgNotificationsManager` instance.
    pub(crate) fn new(
        db: DynDB,
        cfg: EmailConfig,
        tracker: TaskTracker,
        cancellation_token: CancellationToken,
    ) -> Result<Self> {
        let notifications_manager = Self {
            db,
            cfg,
            tracker,
            cancellation_token,
        };
        notifications_manager.run()?;

        Ok(notifications_manager)
    }

    /// Run notifications manager.
    fn run(&self) -> Result<()> {
        // Setup smtp client
        let smtp_client = AsyncSmtpTransport::<Tokio1Executor>::relay(&self.cfg.smtp.host)?
            .credentials(Credentials::new(
                self.cfg.smtp.username.clone(),
                self.cfg.smtp.password.clone(),
            ))
            .build();

        // Setup and run some workers to deliver notifications
        for _ in 1..=NUM_WORKERS {
            let mut worker = Worker {
                db: self.db.clone(),
                cfg: self.cfg.clone(),
                smtp_client: smtp_client.clone(),
                cancellation_token: self.cancellation_token.clone(),
            };
            self.tracker.spawn(async move {
                worker.run().await;
            });
        }

        Ok(())
    }
}

#[async_trait]
impl NotificationsManager for PgNotificationsManager {
    /// [NotificationsManager::enqueue_notification]
    #[instrument(skip(self), err)]
    async fn enqueue(&self, notification: &NewNotification) -> Result<()> {
        self.db.enqueue_notification(notification).await
    }
}

/// Worker in charge of delivering notifications.
struct Worker {
    db: DynDB,
    cfg: EmailConfig,
    smtp_client: AsyncSmtpTransport<Tokio1Executor>,
    cancellation_token: CancellationToken,
}

impl Worker {
    /// Run the worker.
    async fn run(&mut self) {
        loop {
            // Try to deliver a pending notification
            match self.deliver_notification().await {
                Ok(Some(())) => {
                    // One notification was delivered, try to deliver another
                    // one immediately
                }
                Ok(None) => tokio::select! {
                    // No pending notifications, pause unless we've been asked
                    // to stop
                    () = sleep(PAUSE_ON_NONE) => {},
                    () = self.cancellation_token.cancelled() => break,
                },
                Err(err) => {
                    // Something went wrong delivering the notification, pause
                    // unless we've been asked to stop
                    error!("error delivering notification: {err}");
                    tokio::select! {
                        () = sleep(PAUSE_ON_ERROR) => {},
                        () = self.cancellation_token.cancelled() => break,
                    }
                }
            }

            // Exit if the worker has been asked to stop
            if self.cancellation_token.is_cancelled() {
                break;
            }
        }
    }

    /// Deliver pending notification (if any).
    #[instrument(skip(self), err)]
    async fn deliver_notification(&mut self) -> Result<Option<()>> {
        // Begin transaction
        let client_id = self.db.tx_begin().await?;

        // Get pending notification
        let notification = match self.db.get_pending_notification(client_id).await {
            Ok(notification) => notification,
            Err(err) => {
                self.db.tx_rollback(client_id).await?;
                return Err(err);
            }
        };

        // Deliver notification (if any)
        if let Some(notification) = &notification {
            // Prepare subject and body based on notification kind
            let (subject, body) = match notification.kind {
                NotificationKind::EmailVerification => {
                    let template_data = notification.template_data.clone().expect("to be some");
                    let template: EmailVerification = serde_json::from_value(template_data)?;
                    let subject = "Verify your email address";
                    let body = template.render()?;
                    (subject, body)
                }
            };

            // Prepare message and send email
            let err = match self.send_email(&notification.email, subject, body).await {
                Ok(()) => None,
                Err(err) => Some(err.to_string()),
            };

            // Update notification with result
            if let Err(err) = self.db.update_notification(client_id, notification, err).await {
                error!("error updating notification: {err}");
            }
        }

        // Commit transaction
        self.db.tx_commit(client_id).await?;

        Ok(Some(()))
    }

    /// Send email to the given address.
    async fn send_email(&self, to_address: &str, subject: &str, body: String) -> Result<()> {
        // Prepare message
        let message = MessageBuilder::new()
            .from(Mailbox::new(
                Some(self.cfg.from_name.clone()),
                self.cfg.from_address.parse()?,
            ))
            .to(to_address.parse()?)
            .header(ContentType::TEXT_HTML)
            .subject(subject)
            .body(body)?;

        // Send email
        self.smtp_client.send(message).await?;

        Ok(())
    }
}

/// Information required to create a new notification.
#[derive(Debug, Clone)]
pub(crate) struct NewNotification {
    pub kind: NotificationKind,
    pub user_id: Uuid,

    pub template_data: Option<serde_json::Value>,
}

/// Information required to deliver a notification.
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Notification {
    pub notification_id: Uuid,
    pub email: String,
    pub kind: NotificationKind,

    pub template_data: Option<serde_json::Value>,
}

/// Notification kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum NotificationKind {
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
