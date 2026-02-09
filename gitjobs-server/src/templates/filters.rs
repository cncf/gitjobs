//! Custom filters for Askama templates, including formatting and display helpers.
#![allow(
    clippy::cast_precision_loss,
    clippy::inline_always,
    clippy::ref_option,
    clippy::trivially_copy_pass_by_ref,
    clippy::unnecessary_wraps,
    clippy::unused_self
)]

use std::sync::LazyLock;

use chrono::{DateTime, NaiveDate, Utc};
use human_format::{Formatter, Scales};
use tracing::error;

/// Formatter for displaying salary values in human-readable format (e.g., 10K, 1M).
static SALARY_FORMATTER: LazyLock<Formatter> = LazyLock::new(|| {
    let mut scales = Scales::new();
    scales
        .with_base(1000)
        .with_suffixes(vec!["", "K", "M", "B", "T", "P", "E", "Z", "Y"]);

    let mut formatter = Formatter::new();
    formatter.with_scales(scales).with_decimals(0).with_separator("");

    formatter
});

/// Display the value if present, otherwise return an empty string.
#[askama::filter_fn]
pub(crate) fn display_some<T>(value: &Option<T>, _: &dyn askama::Values) -> askama::Result<String>
where
    T: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.to_string()),
        None => Ok(String::new()),
    }
}

/// Display the value if present, otherwise return the provided alternative value.
#[askama::filter_fn]
pub(crate) fn display_some_or<T, U>(
    value: &Option<T>,
    _: &dyn askama::Values,
    alternative: U,
) -> askama::Result<String>
where
    T: std::fmt::Display,
    U: std::fmt::Display,
{
    match value {
        Some(value) => Ok(value.to_string()),
        None => Ok(alternative.to_string()),
    }
}

/// Display the formatted date if present, otherwise return the alternative value.
#[askama::filter_fn]
#[allow(clippy::trivially_copy_pass_by_ref)]
pub(crate) fn display_some_date_or<T>(
    value: &Option<NaiveDate>,
    _: &dyn askama::Values,
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

/// Display the formatted datetime if present, otherwise return an empty string.
#[askama::filter_fn]
#[allow(dead_code)]
pub(crate) fn display_some_datetime(
    value: &Option<DateTime<Utc>>,
    _: &dyn askama::Values,
    format: &str,
) -> askama::Result<String> {
    match value {
        Some(value) => Ok(value.format(format).to_string()),
        None => Ok(String::new()),
    }
}

/// Display the formatted datetime if present, otherwise return the alternative value.
#[askama::filter_fn]
pub(crate) fn display_some_datetime_or<T>(
    value: &Option<DateTime<Utc>>,
    _: &dyn askama::Values,
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

/// Format a salary amount as a human-readable string (e.g., 10K, 1M).
#[askama::filter_fn]
#[allow(clippy::cast_precision_loss, clippy::trivially_copy_pass_by_ref)]
pub(crate) fn humanize_salary(amount: &i64, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(SALARY_FORMATTER.format(*amount as f64))
}

/// Convert a markdown string to HTML using GitHub Flavored Markdown options.
#[askama::filter_fn]
pub(crate) fn md_to_html(s: &str, _: &dyn askama::Values) -> askama::Result<String> {
    let options = markdown::Options::gfm();
    match markdown::to_html_with_options(s, &options) {
        Ok(html) => Ok(html),
        Err(e) => {
            error!("error converting markdown to html: {}", e);
            Ok("error converting markdown to html".to_string())
        }
    }
}

/// Replace hyphens in a string with spaces, returning the unnormalized version.
#[askama::filter_fn]
pub(crate) fn unnormalize(s: &str, _: &dyn askama::Values) -> askama::Result<String> {
    Ok(s.replace('-', " "))
}
