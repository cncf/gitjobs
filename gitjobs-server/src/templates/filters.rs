//! Some custom filters for templates.

use std::sync::LazyLock;

use chrono::{DateTime, NaiveDate, Utc};
use human_format::{Formatter, Scales};
use tracing::error;

/// Salary formatter.
static SALARY_FORMATTER: LazyLock<Formatter> = LazyLock::new(|| {
    let mut scales = Scales::new();
    scales
        .with_base(1000)
        .with_suffixes(vec!["", "K", "M", "B", "T", "P", "E", "Z", "Y"]);

    let mut formatter = Formatter::new();
    formatter.with_scales(scales).with_decimals(0).with_separator("");

    formatter
});

/// Return the value if it is some, otherwise return an empty string.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn display_some<T>(value: &Option<T>) -> askama::Result<String>
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
pub(crate) fn display_some_or<T, U>(value: &Option<T>, alternative: U) -> askama::Result<String>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Return the formatted date if it is some, otherwise return the alternative
/// value.
#[allow(
    clippy::unnecessary_wraps,
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref
)]
pub(crate) fn display_some_date_or<T>(
    value: &Option<NaiveDate>,
    format: &str,
    alternative: T,
) -> askama::Result<String>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Return the formatted datetime if it is some, otherwise return an empty
/// string.
#[allow(clippy::unnecessary_wraps, clippy::ref_option, dead_code)]
pub(crate) fn display_some_datetime(value: &Option<DateTime<Utc>>, format: &str) -> askama::Result<String> {
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(String::new()),
    }
}

/// Return the formatted datetime if it is some, otherwise return the
/// alternative value.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn display_some_datetime_or<T>(
    value: &Option<DateTime<Utc>>,
    format: &str,
    alternative: T,
) -> askama::Result<String>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Return the salary amount in humanized format.
#[allow(
    clippy::unnecessary_wraps,
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::cast_precision_loss
)]
pub(crate) fn humanize_salary(amount: &i64) -> askama::Result<String> {
    Ok(SALARY_FORMATTER.format(*amount as f64))
}

/// Filter to convert markdown to html.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn md_to_html(s: &str) -> askama::Result<String> {
    let options = markdown::Options::gfm();
    match markdown::to_html_with_options(s, &options) {
        Ok(html) => Ok(html),
        Err(e) => {
            error!("error converting markdown to html: {}", e);
            Ok("error converting markdown to html".to_string())
        }
    }
}

/// Return the unnormalized version of the string provided.
#[allow(clippy::unnecessary_wraps, clippy::ref_option)]
pub(crate) fn unnormalize(s: &str) -> askama::Result<String> {
    Ok(s.replace('-', " "))
}
