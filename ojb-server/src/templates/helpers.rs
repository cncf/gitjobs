//! Some helpers for templates.

/// The date format used in the templates.
pub(crate) const DATE_FORMAT: &str = "%Y-%m-%d";

/// Build location string from the location information provided.
pub(crate) fn build_location(
    city: Option<&String>,
    state: Option<&String>,
    country: Option<&String>,
) -> Option<String> {
    let mut location = String::new();

    let mut push = |part: Option<&String>| {
        if let Some(part) = part {
            if !part.is_empty() {
                if !location.is_empty() {
                    location.push_str(", ");
                }
                location.push_str(part.as_str());
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
