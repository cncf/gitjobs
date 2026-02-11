//! Validation utilities and custom validators for form input.
//!
//! This module provides custom garde validators for common validation patterns
//! used throughout the application.
//!
//! Note: garde custom validators require specific signatures that may trigger
//! clippy warnings. The `&()` context parameter and `&Option<T>` patterns are
//! required by garde's API.

#![allow(clippy::ref_option)]
#![allow(clippy::trivially_copy_pass_by_ref)]

/// Maximum number of elements in a collection (filters, tags, etc.).
pub const MAX_ITEMS: usize = 25;

/// Maximum length for biographies.
pub const MAX_LEN_BIO: usize = 1000;

/// Maximum length for full descriptions.
pub const MAX_LEN_DESCRIPTION: usize = 8000;

/// Maximum length for short descriptions.
pub const MAX_LEN_DESCRIPTION_SHORT: usize = 500;

/// Maximum length for display names shown to users.
pub const MAX_LEN_DISPLAY_NAME: usize = 80;

/// Maximum length for entity names (jobs, employers, and similar entities).
pub const MAX_LEN_ENTITY_NAME: usize = 120;

/// Maximum length for tag values.
pub const MAX_LEN_TAG: usize = 50;

/// Minimum length for passwords.
pub const MIN_PASSWORD_LEN: usize = 8;

/// Maximum length for long text fields.
pub const MAX_LEN_L: usize = 2000;

/// Maximum length for medium text fields.
pub const MAX_LEN_M: usize = 250;

/// Maximum length for short text fields.
pub const MAX_LEN_S: usize = 100;

/// Validates that a string is non-empty after trimming whitespace.
///
/// Returns an error if the string is empty or contains only whitespace.
pub fn trimmed_non_empty(value: &impl AsRef<str>, _ctx: &()) -> garde::Result {
    if value.as_ref().trim().is_empty() {
        return Err(garde::Error::new("value cannot be empty or whitespace-only"));
    }
    Ok(())
}

/// Validates that an optional string is non-empty after trimming if present.
///
/// Returns Ok if the value is None, or if it's Some with non-whitespace
/// content. Returns an error if the value is Some but empty or whitespace-only.
pub fn trimmed_non_empty_opt(value: &Option<String>, _ctx: &()) -> garde::Result {
    if let Some(s) = value
        && s.trim().is_empty()
    {
        return Err(garde::Error::new("value cannot be empty or whitespace-only"));
    }
    Ok(())
}

/// Validates that each tag in a vector is non-empty and within max length.
pub fn trimmed_non_empty_tag_vec(value: &Option<Vec<String>>, _ctx: &()) -> garde::Result {
    validate_trimmed_non_empty_vec(value, MAX_LEN_TAG)
}

/// Validates that each string in a vector is non-empty and within max length.
pub fn trimmed_non_empty_vec(value: &Option<Vec<String>>, _ctx: &()) -> garde::Result {
    validate_trimmed_non_empty_vec(value, MAX_LEN_M)
}

// Validates a vector of trimmed non-empty strings with size and item limits.
fn validate_trimmed_non_empty_vec(value: &Option<Vec<String>>, max_len: usize) -> garde::Result {
    if let Some(vec) = value {
        if vec.len() > MAX_ITEMS {
            return Err(garde::Error::new(format!(
                "value exceeds max items of {MAX_ITEMS}"
            )));
        }
        for s in vec {
            if s.trim().is_empty() {
                return Err(garde::Error::new("value cannot be empty or whitespace-only"));
            }
            if s.len() > max_len {
                return Err(garde::Error::new(format!(
                    "value exceeds max length of {max_len}"
                )));
            }
        }
    }
    Ok(())
}

// Tests.

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trimmed_non_empty_accepts_non_empty_values() {
        assert!(trimmed_non_empty(&"value", &()).is_ok());
        assert!(trimmed_non_empty(&"  value  ", &()).is_ok());
    }

    #[test]
    fn test_trimmed_non_empty_opt_accepts_none() {
        assert!(trimmed_non_empty_opt(&None, &()).is_ok());
    }

    #[test]
    fn test_trimmed_non_empty_opt_rejects_whitespace_only_values() {
        assert!(trimmed_non_empty_opt(&Some("   ".to_string()), &()).is_err());
    }

    #[test]
    fn test_trimmed_non_empty_rejects_whitespace_only_values() {
        assert!(trimmed_non_empty(&"", &()).is_err());
        assert!(trimmed_non_empty(&"   ", &()).is_err());
    }

    #[test]
    fn test_trimmed_non_empty_tag_vec_rejects_item_too_long() {
        let item = "a".repeat(MAX_LEN_TAG + 1);
        assert!(trimmed_non_empty_tag_vec(&Some(vec![item]), &()).is_err());
    }

    #[test]
    fn test_trimmed_non_empty_vec_rejects_over_limit_items() {
        let values = vec!["value".to_string(); MAX_ITEMS + 1];
        assert!(trimmed_non_empty_vec(&Some(values), &()).is_err());
    }
}
