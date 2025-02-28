//! This module defines some database functionality used to manage
//! notifications.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;

use crate::{PgDB, notifications::Notification};

/// Trait that defines some database operations used to manage notifications.
#[async_trait]
pub(crate) trait DBNotifications {
    /// Enqueue a notification to be sent.
    async fn enqueue_notification(&self, notification: &Notification) -> Result<()>;
}

#[async_trait]
impl DBNotifications for PgDB {
    /// [DBNotifications::enqueue_notification]
    #[instrument(skip(self), err)]
    async fn enqueue_notification(&self, notification: &Notification) -> Result<()> {
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
}
