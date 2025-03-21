//! Some helpers for templates.

use std::sync::LazyLock;

use regex::Regex;
use uuid::Uuid;

use crate::templates::dashboard::employer::employers::EmployerSummary;

/// The date format used in the templates.
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

/// The date format used in the jobseeker preview.
pub(crate) const DATE_FORMAT_2: &str = "%B %Y";

/// The date format used in the jobboard jobs page.
pub(crate) const DATE_FORMAT_3: &str = "%b %e";

/// Build dashboard url for an image version.
pub(crate) fn build_dashboard_image_url(image_id: &Uuid, version: &str) -> String {
    format!("/dashboard/images/{image_id}/{version}")
}

/// Build job board url for an image version.
pub(crate) fn build_jobboard_image_url(image_id: &Uuid, version: &str) -> String {
    format!("/jobboard/images/{image_id}/{version}")
}

/// Find the employer with the given id in the list of employers.
pub(crate) fn find_employer<'a>(
    employer_id: Option<&'a Uuid>,
    employers: &'a [EmployerSummary],
) -> Option<&'a EmployerSummary> {
    let employer_id = employer_id?;
    employers.iter().find(|e| e.employer_id == *employer_id)
}

/// Format location string from the location information provided.
pub(crate) fn format_location(
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

/// Check if the value provided is none or some and default.
#[allow(clippy::ref_option)]
#[allow(dead_code)]
pub(crate) fn option_is_none_or_default<T: Default + PartialEq>(v: &Option<T>) -> bool {
    if let Some(value) = v {
        return *value == T::default();
    }
    true
}

/// Regular expression to match multiple hyphens.
static MULTIPLE_HYPHENS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-{2,}").expect("exprs in MULTIPLE_HYPHENS should be valid"));

/// Normalize string.
pub(crate) fn normalize(s: &str) -> String {
    let normalized = s.to_lowercase().replace(' ', "-");
    let normalized = MULTIPLE_HYPHENS.replace(&normalized, "-").to_string();
    normalized
}
