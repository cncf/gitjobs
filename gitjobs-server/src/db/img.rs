//! This module defines some database functionality used to manage images.

use anyhow::Result;
use async_trait::async_trait;
use tracing::instrument;
use uuid::Uuid;

use crate::{img::ImageVersion, PgDB};

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
    async fn save_image_versions(&self, versions: Vec<ImageVersion>) -> Result<Uuid>;
}

#[async_trait]
impl DBImage for PgDB {
    /// [DBImage::get_image_version]
    #[instrument(skip(self), err)]
    async fn get_image_version(
        &self,
        image_id: Uuid,
        version: &str,
    ) -> Result<Option<(Vec<u8>, ImageFormat)>> {
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

/// Image format.
#[derive(Debug, Clone)]
pub(crate) enum ImageFormat {
    Png,
    Svg,
}

impl TryFrom<&str> for ImageFormat {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "png" => Ok(Self::Png),
            "svg" => Ok(Self::Svg),
            _ => Err(anyhow::Error::msg("invalid image format")),
        }
    }
}
