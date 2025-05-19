//! Types and functionality for managing image storage, formats, and processing.

use std::{io::Cursor, sync::Arc};

use anyhow::Result;
use async_trait::async_trait;
use uuid::Uuid;

pub(crate) mod db;

/// Trait for image storage backends supporting get and save operations.
#[async_trait]
pub(crate) trait ImageStore {
    /// Retrieve an image version from the store.
    async fn get(&self, image_id: Uuid, version: &str) -> Result<Option<(Vec<u8>, ImageFormat)>>;

    /// Save an image to the store and return its unique identifier.
    async fn save(&self, user_id: &Uuid, filename: &str, data: Vec<u8>) -> Result<Uuid>;
}

/// Thread-safe trait object alias for image storage implementations.
pub(crate) type DynImageStore = Arc<dyn ImageStore + Send + Sync>;

/// Generate resized versions of an image for multiple predefined sizes.
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

        // Encode resized version of the image to png format
        let mut buf = vec![];
        version.write_to(&mut Cursor::new(&mut buf), image::ImageFormat::Png)?;

        versions.push(ImageVersion {
            data: buf,
            version: (*size_name).to_string(),
        });
    }

    Ok(versions)
}

/// Represents a version of an image of a specific size (or format).
#[derive(Debug, Clone)]
pub(crate) struct ImageVersion {
    /// Raw image data in the specified format.
    pub data: Vec<u8>,
    /// Version label, e.g., "small", "medium", or "large".
    pub version: String,
}

/// Supported image formats for storage and processing.
#[derive(Debug, Clone, strum::Display, strum::EnumString)]
#[strum(serialize_all = "lowercase")]
pub(crate) enum ImageFormat {
    /// PNG image format.
    Png,
    /// SVG image format.
    Svg,
}

/// Returns true if the file name has an SVG extension (case-insensitive).
pub(crate) fn is_svg(file_name: &str) -> bool {
    if let Some(extension) = file_name.split('.').next_back() {
        if extension.to_lowercase() == "svg" {
            return true;
        }
    }
    false
}
