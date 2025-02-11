//! This module implements a database-backed image store.

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

use crate::{
    db::DynDB,
    img::{generate_versions, is_svg, ImageFormat, ImageStore, ImageVersion},
};

/// Database-backed image store.
pub(crate) struct DbImageStore {
    // TODO: switch to DynDBImage when 1.86 is released
    // https://github.com/rust-lang/rust/issues/65991
    db: DynDB,
}

impl DbImageStore {
    /// Create a new `DbImageStore` instance.
    pub(crate) fn new(db: DynDB) -> Self {
        Self { db }
    }
}

#[async_trait]
impl ImageStore for DbImageStore {
    async fn get(&self, image_id: Uuid, version: &str) -> Result<Option<(Vec<u8>, ImageFormat)>> {
        self.db.get_image_version(image_id, version).await
    }

    async fn save(&self, filename: &str, data: Vec<u8>) -> Result<Uuid> {
        // Prepare image versions
        let versions = if is_svg(filename) {
            // Use the original svg image, no need to generate other versions
            vec![ImageVersion {
                data,
                version: "svg".to_string(),
            }]
        } else {
            // Generate versions for different sizes in png format
            generate_versions(&data)?
        };

        // Save image versions to the database
        self.db.save_image_versions(versions).await
    }
}
