//! Helpers for formatting, normalization, and data conversion in templates.

use std::{collections::HashMap, sync::LazyLock, time::Duration};

use anyhow::Result;
use cached::proc_macro::cached;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use tracing::{debug, warn};
use uuid::Uuid;

use crate::templates::dashboard::employer::employers::EmployerSummary;

/// The date format used in the templates (YYYY-MM-DD).
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

/// The date format used in the jobseeker preview (Month Year).
pub(crate) const DATE_FORMAT_2: &str = "%B %Y";

/// The date format used in the jobboard jobs page (Abbreviated Month Day).
pub(crate) const DATE_FORMAT_3: &str = "%b %e";

/// Build dashboard image URL for a specific image version.
pub(crate) fn build_dashboard_image_url(image_id: &Uuid, version: &str) -> String {
    format!("/dashboard/images/{image_id}/{version}")
}

/// Build job board image URL for a specific image version.
pub(crate) fn build_jobboard_image_url(image_id: &Uuid, version: &str) -> String {
    format!("/jobboard/images/{image_id}/{version}")
}

/// Deserializes an optional string and maps empty values to `None`.
pub(crate) fn empty_string_as_none<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<String>::deserialize(deserializer)?;

    Ok(value.and_then(|value| if value.is_empty() { None } else { Some(value) }))
}

/// Find an employer by id in a list of employers.
pub(crate) fn find_employer<'a>(
    employer_id: Option<&'a Uuid>,
    employers: &'a [EmployerSummary],
) -> Option<&'a EmployerSummary> {
    let employer_id = employer_id?;
    employers.iter().find(|e| e.employer_id == *employer_id)
}

/// Format a location string from city, state, and country information.
pub(crate) fn format_location(
    city: Option<&str>,
    state: Option<&str>,
    country: Option<&str>,
) -> Option<String> {
    let mut location = String::new();

    let mut push = |part: Option<&str>| {
        if let Some(part) = part
            && !part.is_empty()
        {
            if !location.is_empty() {
                location.push_str(", ");
            }
            location.push_str(part);
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

/// Returns true if the option is None or contains the default value.
#[allow(clippy::ref_option)]
pub(crate) fn option_is_none_or_default<T: Default + PartialEq>(v: &Option<T>) -> bool {
    if let Some(value) = v {
        return *value == T::default();
    }
    true
}

/// Regex for matching multiple consecutive hyphens in a string.
static MULTIPLE_HYPHENS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"-{2,}").expect("exprs in MULTIPLE_HYPHENS should be valid"));

/// Normalize a string: lowercase, replace spaces with hyphens, collapse multiple hyphens.
pub(crate) fn normalize(s: &str) -> String {
    let normalized = s.to_lowercase().replace(' ', "-");
    MULTIPLE_HYPHENS.replace(&normalized, "-").to_string()
}

/// Convert a salary to a yearly USD value using exchange rates and period.
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
    let Some(exchange_rate) = exchange_rates.get(&currency.to_lowercase()) else {
        warn!(currency, "exchange rate not found");
        return None;
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

/// Return current exchange rates, defaulting to backup ones if the current
/// ones aren't available. Rates will be cached for 1 day.
#[cached(time = 86400, sync_writes = "by_key")]
async fn get_exchange_rates() -> HashMap<String, f64> {
    // Exchange rates as of 2024-04-28.
    let backup_exchange_rates = HashMap::from([
        ("usd".to_string(), 1.0),
        ("eur".to_string(), 0.878),
        ("gbp".to_string(), 0.751),
        ("cad".to_string(), 1.386),
        ("chf".to_string(), 0.828),
        ("jpy".to_string(), 143.658),
    ]);

    // Download current exchange rates.
    let Ok(exchange_rates) = download_exchange_rates().await else {
        warn!("error downloading current exchange rates, using backup ones");
        return backup_exchange_rates;
    };

    exchange_rates
}

/// Download current exchange rates from a public API.
#[tracing::instrument(skip_all, err)]
async fn download_exchange_rates() -> Result<HashMap<String, f64>> {
    debug!("downloading current exchange rates");

    let url = "https://cdn.jsdelivr.net/npm/@fawazahmed0/currency-api@latest/v1/currencies/usd.json";
    let client = reqwest::Client::builder().timeout(Duration::from_secs(5)).build()?;
    let exchange_rates: ExchangeRatesApiResponse = client.get(url).send().await?.json().await?;

    Ok(exchange_rates.usd)
}

/// Structure for the exchange rates API response.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
struct ExchangeRatesApiResponse {
    pub usd: HashMap<String, f64>,
}
