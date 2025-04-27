//! Some helpers for templates.

use std::{collections::HashMap, sync::LazyLock};

use anyhow::Result;
use cached::proc_macro::cached;
use regex::Regex;
use tracing::warn;
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
pub(crate) async fn normalize_salary(
    salary: Option<i64>,
    currency: Option<&String>,
    period: Option<&String>,
) -> Option<i64> {
    // Currency and period must be provided to convert the salary.
    let (Some(salary), Some(currency), Some(period)) = (salary, currency, period) else {
        return None;
    };

    // Convert to USD.
    let exchange_rates = get_exchange_rates().await;
    let Some(exchange_rate) = exchange_rates.get(currency) else {
        warn!("invalid exchange rate");
        return None; // Unsupported exchange rate.
    };

    #[allow(clippy::cast_precision_loss)]
    let salary_usd = salary as f64 / exchange_rate;

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

/// Return current exchange rates defaulting to backup ones if the current ones aren't available. Values will be cached for 1 day.
#[cached(time = 86400, sync_writes = "by_key")]
async fn get_exchange_rates() -> HashMap<String, f64> {
    let mut backup_exchange_rates = HashMap::from([
        ("usd".to_string(), 1.0),
        ("eur".to_string(), 0.87),
        ("gbp".to_string(), 0.7476),
        ("cad".to_string(), 1.3832),
        ("chf".to_string(), 0.8117),
        ("jpy".to_string(), 143.6587),
    ]);

    let Ok(exchange_rates) = download_exchange_rates().await else {
        return backup_exchange_rates; // If current exchange rates aren't available, return backup ones.
    };
    backup_exchange_rates.extend(exchange_rates); // Update current rates with backup ones in case an exchange rate is missing.

    backup_exchange_rates
}

/// Download current exchange rates.
async fn download_exchange_rates() -> Result<HashMap<String, f64>> {
    let url = "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json";
    let data = reqwest::get(url).await?.text().await?;
    let exchange_rates: ExchangeRatesApiResponse = serde_json::from_str(&data)?;

    Ok(exchange_rates.usd)
}

#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct ExchangeRatesApiResponse {
    pub usd: HashMap<String, f64>,
}
