//! Some helpers for templates.

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

/// The date format used in the templates.
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

/// Build url for an image version.
pub(crate) fn build_image_url(image_id: Uuid, version: &str) -> String {
    format!("/images/{image_id}/{version}")
}

/// Build location string from the location information provided.
pub(crate) fn build_location(
    city: Option<&str>,
    state: Option<&str>,
    country: Option<&str>,
) -> Option<String> {
    let mut location = String::new();

    let mut push = |part: Option<&str>| {
        if let Some(part) = part {
            if !part.is_empty() {
                if !location.is_empty() {
                    location.push_str(", ");
                }
                location.push_str(part);
            }
        }
    };

    push(city);
    push(state);
    push(country);

    if !location.is_empty() {
        return Some(location);
    }
    None
}

/// Regular expression to match multiple hyphens.
static MULTIPLE_HYPHENS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-{2,}").expect("exprs in MULTIPLE_HYPHENS to be valid"));

/// Normalize string.
pub(crate) fn normalize(s: &str) -> String {
    let normalized = s.to_lowercase().replace(' ', "-");
    let normalized = MULTIPLE_HYPHENS.replace(&normalized, "-").to_string();
    normalized
}
