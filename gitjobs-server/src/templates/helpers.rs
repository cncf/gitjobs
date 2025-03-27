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

/// Convert salary to USD yearly.
/// TODO(tegioz): refresh exchange rates automatically.
pub(crate) fn normalize_salary(
    salary: Option<i64>,
    currency: Option<&String>,
    period: Option<&String>,
) -> Option<i64> {
    // Currency and period must be provided to convert the salary.
    let (Some(salary), Some(currency), Some(period)) = (salary, currency, period) else {
        return None;
    };

    // Convert to USD.
    let conversion_rate = match currency.to_lowercase().as_str() {
        "usd" => 1.0,
        "eur" => 1.08,
        "gbp" => 1.29,
        "cad" => 0.7,
        "chf" => 1.13,
        "jpy" => 0.0066,
        _ => {
            return None; // Unsupported currency.
        }
    };
    #[allow(clippy::cast_precision_loss)]
    let salary_usd = salary as f64 * conversion_rate;

    // Convert to yearly salary.
    let salary_usd_year = match period.as_str() {
        "year" => salary_usd,
        "month" => salary_usd * 12.0,
        "week" => salary_usd * 52.0,
        "day" => salary_usd * 5.0 * 52.0,
        "hour" => salary_usd * 40.0 * 52.0,
        _ => {
            return None; // Unsupported period.
        }
    };

    #[allow(clippy::cast_possible_truncation)]
    Some(salary_usd_year as i64)
}
