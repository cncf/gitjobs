//! This module defines some database functionality used to manage images.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{img::ImageVersion, PgDB};

/// Trait that defines some database operations used to manage images.
#[async_trait]
pub(crate) trait DBImage {
    /// Save image versions.
    async fn save_image_versions(&self, versions: Vec<ImageVersion>) -> Result<Uuid>;
}

#[async_trait]
impl DBImage for PgDB {
    /// [DBImage::save_image_versions]
    #[instrument(skip(self), err)]
    async fn save_image_versions(&self, versions: Vec<ImageVersion>) -> Result<Uuid> {
        // Begin transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Insert image identifier
        let image_id = Uuid::new_v4();
        tx.execute("insert into image (image_id) values ($1::uuid)", &[&image_id])
            .await?;

        // Insert image versions
        for v in versions {
            tx.execute(
                "
                insert into image_version (image_id, version, data)
                values ($1::uuid, $2::text, $3::bytea)
                ",
                &[&image_id, &v.version, &v.data],
            )
            .await?;
        }

        // Commit transaction
        tx.commit().await?;

        Ok(image_id)
    }
}
