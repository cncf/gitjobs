//! This module defines some types and functionality to manage images.

use std::{io::Cursor, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

pub(crate) mod db;

/// Trait that defines the operations an image store must support.
#[async_trait]
pub(crate) trait ImageStore {
    /// Get an image version of the requested size from the store.
    async fn get(&self, image_id: Uuid, version: &str) -> Result<Option<Vec<u8>>>;

    /// Save an image to the store.
    async fn save(&self, filename: &str, data: Vec<u8>) -> Result<Uuid>;
}

/// Type alias to represent an `ImageStore` trait object.
pub(crate) type DynImageStore = Arc<dyn ImageStore + Send + Sync>;

/// Version of an image of a specific size.
#[derive(Debug, Clone)]
pub(crate) struct ImageVersion {
    pub data: Vec<u8>,
    pub version: String,
}

/// Generate versions of different sizes for an image.
pub(crate) fn generate_versions(data: &[u8]) -> Result<Vec<ImageVersion>> {
    // Read image data
    let img = image::ImageReader::new(Cursor::new(data))
        .with_guessed_format()?
        .decode()?;

    // Generate versions for different sizes
    let mut versions = vec![];
    for (size_name, size) in &[("small", 100), ("medium", 200), ("large", 400)] {
        // Resize image
        let version = img.resize(*size, *size, image::imageops::FilterType::Lanczos3);

        // Encode resized version of the image
        let mut buf = vec![];
        version.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        versions.push(ImageVersion {
            data: buf,
            version: (*size_name).to_string(),
        });
    }

    Ok(versions)
}

/// Check if the image is in SVG format.
pub(crate) fn is_svg(file_name: &str) -> bool {
    if let Some(extension) = file_name.split('.').last() {
        if extension.to_lowercase() == "svg" {
            return true;
        }
    }
    false
}
