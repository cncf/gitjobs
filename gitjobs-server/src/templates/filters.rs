//! Some custom filters for templates.

use chrono::{DateTime, Utc};

/// Return the value if it is some, otherwise return an empty string.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn display_some<T>(value: &Option<T>) -> rinja::Result<String>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.to_string()),
        None => Ok(String::new()),
    }
}

/// Return the value if it is some, otherwise return the alternative value.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn display_some_or<T, U>(value: &Option<T>, alternative: U) -> rinja::Result<String>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Return the formatted date if it is some, otherwise return an empty string.
#[allow(clippy::unnecessary_wraps, clippy::ref_option, dead_code)]
pub(crate) fn display_some_date(value: &Option<DateTime<Utc>>, format: &str) -> rinja::Result<String> {
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(String::new()),
    }
}

/// Return the formatted date if it is some, otherwise return the alternative
/// value.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn display_some_date_or<T>(
    value: &Option<DateTime<Utc>>,
    format: &str,
    alternative: T,
) -> rinja::Result<String>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Filter to convert markdown to html.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn md_to_html(s: &str) -> rinja::Result<String> {
    Ok(markdown::to_html(s))
}

/// Return the unnormalized version of the string provided.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn unnormalize(s: &str) -> rinja::Result<String> {
    Ok(s.replace('-', " "))
}
