//! This module defines some database functionality used to manage images.

use std::sync::Arc;

use anyhow::Result;
use async_trait::async_trait;
use tracing::{instrument, trace};
use uuid::Uuid;

use crate::{
    PgDB,
    img::{ImageFormat, ImageVersion},
};

/// Trait that defines some database operations used to manage images.
#[async_trait]
pub(crate) trait DBImage {
    /// Get image version.
    async fn get_image_version(
        &self,
        image_id: Uuid,
        version: &str,
    ) -> Result<Option<(Vec<u8>, ImageFormat)>>;

    /// Save image versions.
    async fn save_image_versions(&self, user_id: &Uuid, versions: Vec<ImageVersion>) -> Result<Uuid>;
}

/// Type alias to represent a `DBImage` trait object.
pub(crate) type DynDBImage = Arc<dyn DBImage + Send + Sync>;

#[async_trait]
impl DBImage for PgDB {
    #[instrument(skip(self), err)]
    async fn get_image_version(
        &self,
        image_id: Uuid,
        version: &str,
    ) -> Result<Option<(Vec<u8>, ImageFormat)>> {
        trace!("db: get image version");

        let db = self.pool.get().await?;
        let Some(row) = db
            .query_opt(
                "select data, format from get_image_version($1::uuid, $2::text)",
                &[&image_id, &version],
            )
            .await?
        else {
            return Ok(None);
        };
        let data = row.get::<_, Vec<u8>>("data");
        let format = ImageFormat::try_from(row.get::<_, &str>("format"))?;

        Ok(Some((data, format)))
    }

    #[instrument(skip(self, versions), err)]
    async fn save_image_versions(&self, user_id: &Uuid, versions: Vec<ImageVersion>) -> Result<Uuid> {
        trace!("db: save image versions");

        // Begin transaction
        let mut db = self.pool.get().await?;
        let tx = db.transaction().await?;

        // Insert image identifier
        let image_id = Uuid::new_v4();
        tx.execute(
            "
            insert into image (
                image_id,
                created_by
            ) values (
                $1::uuid,
                $2::uuid
            )",
            &[&image_id, &user_id],
        )
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
