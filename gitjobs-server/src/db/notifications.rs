//! This module defines database functionality used to manage notifications, including
//! enqueueing, retrieving, and updating notification records.

use std::sync::Arc;

use anyhow::{Result, bail};
use async_trait::async_trait;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    db::TX_CLIENT_NOT_FOUND,
    notifications::{NewNotification, Notification},
};

/// Trait that defines database operations used to manage notifications.
#[async_trait]
pub(crate) trait DBNotifications {
    /// Enqueues a notification to be delivered.
    async fn enqueue_notification(&self, notification: &NewNotification) -> Result<()>;

    /// Retrieves a pending notification for delivery.
    async fn get_pending_notification(&self, client_id: Uuid) -> Result<Option<Notification>>;

    /// Updates a notification after a delivery attempt.
    async fn update_notification(
        &self,
        client_id: Uuid,
        notification: &Notification,
        error: Option<String>,
    ) -> Result<()>;
}

#[async_trait]
impl DBNotifications for PgDB {
    #[instrument(skip(self, notification), err)]
    async fn enqueue_notification(&self, notification: &NewNotification) -> Result<()> {
        trace!("db: enqueue notification");

        // Nothing to enqueue
        if notification.recipients.is_empty() {
            trace!("db: skip enqueue notification with empty recipients");
            return Ok(());
        }

        // Enqueue notification in database
        let db = self.pool.get().await?;
        db.execute(
            "select enqueue_notification($1::text, $2::jsonb, $3::uuid[]);",
            &[
                &notification.kind.to_string(),
                &notification.template_data,
                &notification.recipients,
            ],
        )
        .await?;

        Ok(())
    }

    #[instrument(skip(self), err)]
    async fn get_pending_notification(&self, client_id: Uuid) -> Result<Option<Notification>> {
        // Get transaction client
        let tx = {
            let clients = self.txs_clients.read().await;
            let Some((tx, _)) = clients.get(&client_id) else {
                bail!(TX_CLIENT_NOT_FOUND);
            };
            Arc::clone(tx)
        };

        // Get pending notification (if any)
        let notification = tx
            .query_opt("select * from get_pending_notification();", &[])
            .await?
            .map(|row| Notification {
                email: row.get("email"),
                kind: row
                    .get::<_, String>("kind")
                    .as_str()
                    .try_into()
                    .expect("kind to be valid"),
                notification_id: row.get("notification_id"),
                template_data: row.get("template_data"),
            });

        Ok(notification)
    }

    /// Updates the notification record after processing, marking it as processed and
    /// recording any error.
    #[instrument(skip(self, notification), err)]
    async fn update_notification(
        &self,
        client_id: Uuid,
        notification: &Notification,
        error: Option<String>,
    ) -> Result<()> {
        trace!("db: update notification");

        // Get transaction client
        let tx = {
            let clients = self.txs_clients.read().await;
            let Some((tx, _)) = clients.get(&client_id) else {
                bail!(TX_CLIENT_NOT_FOUND);
            };
            Arc::clone(tx)
        };

        // Update notification
        tx.execute(
            "select update_notification($1::uuid, $2::text);",
            &[&notification.notification_id, &error],
        )
        .await?;

        Ok(())
    }
}
