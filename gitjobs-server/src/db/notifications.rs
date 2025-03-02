//! This module defines some database functionality used to manage
//! notifications.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    PgDB,
    notifications::{NewNotification, Notification},
};

/// Trait that defines some database operations used to manage notifications.
#[async_trait]
pub(crate) trait DBNotifications {
    /// Enqueue a notification to be sent.
    async fn enqueue_notification(&self, notification: &NewNotification) -> Result<()>;

    /// Get pending notification.
    async fn get_pending_notification(&self, uuid: Uuid) -> Result<Option<Notification>>;

    /// Update notification.
    async fn update_notification(
        &self,
        uuid: Uuid,
        notification: &Notification,
        error: Option<String>,
    ) -> Result<()>;
}

#[async_trait]
impl DBNotifications for PgDB {
    /// [DBNotifications::enqueue_notification]
    #[instrument(skip(self), err)]
    async fn enqueue_notification(&self, notification: &NewNotification) -> Result<()> {
        let db = self.pool.get().await?;
        db.execute(
            "
            insert into notification (kind, user_id, template_data)
            values ($1::text, $2::uuid, $3::jsonb);
            ",
            &[
                &notification.kind.to_string(),
                &notification.user_id,
                &notification.template_data,
            ],
        )
        .await?;

        Ok(())
    }

    /// [DBNotifications::get_pending_notification]
    #[instrument(skip(self), err)]
    async fn get_pending_notification(&self, uuid: Uuid) -> Result<Option<Notification>> {
        let conns = self.txs_conns.read().await;
        let (tx, _) = conns.get(&uuid).unwrap();
        let notification = tx
            .query_opt(
                r#"
                select
                    kind,
                    notification_id,
                    email,
                    template_data
                from notification n join "user" on n.user_id = "user".user_id
                where processed = false
                order by notification_id asc
                limit 1
                for update of n skip locked;
                "#,
                &[],
            )
            .await?
            .map(|row| Notification {
                kind: row
                    .get::<_, String>("kind")
                    .as_str()
                    .try_into()
                    .expect("kind to be valid"),
                id: row.get("notification_id"),
                email: row.get("email"),
                template_data: row.get("template_data"),
            });

        Ok(notification)
    }

    /// [DBNotifications::update_notification]
    #[instrument(skip(self), err)]
    async fn update_notification(
        &self,
        uuid: Uuid,
        notification: &Notification,
        error: Option<String>,
    ) -> Result<()> {
        let conns = self.txs_conns.read().await;
        let (tx, _) = conns.get(&uuid).unwrap();
        tx.execute(
            "
            update notification set
                processed = true,
                error = $1::text
            where notification_id = $2::uuid;
            ",
            &[&error, &notification.id],
        )
        .await?;

        Ok(())
    }
}
