//! Types and helpers for paginating results and generating navigation links.

use std::fmt::Write as _;

use anyhow::Result;
use askama::Template;
use serde::{Deserialize, Serialize};

/// Default number of items per page for pagination.
#[cfg(test)]
const DEFAULT_PAGINATION_LIMIT: usize = 10;
#[cfg(not(test))]
const DEFAULT_PAGINATION_LIMIT: usize = 20;

/// Trait for types that support pagination.
pub(crate) trait Pagination {
    /// Returns the base URL for htmx requests.
    fn get_base_hx_url(&self) -> String;

    /// Returns the base URL for standard requests.
    fn get_base_url(&self) -> String;

    /// Returns the pagination limit, if set.
    fn limit(&self) -> Option<usize>;

    /// Returns the pagination offset, if set.
    fn offset(&self) -> Option<usize>;

    /// Sets the pagination offset.
    fn set_offset(&mut self, offset: Option<usize>);
}

/// Holds navigation links for paginated results.
#[derive(Debug, Clone, Default, Template, PartialEq, Serialize, Deserialize)]
#[template(path = "navigation_links.html")]
pub(crate) struct NavigationLinks {
    /// Link to the first page, if available.
    pub first: Option<NavigationLink>,
    /// Link to the last page, if available.
    pub last: Option<NavigationLink>,
    /// Link to the next page, if available.
    pub next: Option<NavigationLink>,
    /// Link to the previous page, if available.
    pub prev: Option<NavigationLink>,
}

impl NavigationLinks {
    /// Builds navigation links from filters and total item count.
    pub(crate) fn from_filters<T>(filters: &T, total: usize) -> Result<Self>
    where
        T: Serialize + Clone + Pagination,
    {
        let mut links = NavigationLinks::default();

        let offsets = NavigationLinksOffsets::new(filters.offset(), filters.limit(), total);
        let mut filters = filters.clone();

        if let Some(first_offset) = offsets.first {
            filters.set_offset(Some(first_offset));
            links.first = Some(NavigationLink::new(&filters)?);
        }
        if let Some(last_offset) = offsets.last {
            filters.set_offset(Some(last_offset));
            links.last = Some(NavigationLink::new(&filters)?);
        }
        if let Some(next_offset) = offsets.next {
            filters.set_offset(Some(next_offset));
            links.next = Some(NavigationLink::new(&filters)?);
        }
        if let Some(prev_offset) = offsets.prev {
            filters.set_offset(Some(prev_offset));
            links.prev = Some(NavigationLink::new(&filters)?);
        }

        Ok(links)
    }
}

/// Represents a single navigation link for pagination.
#[derive(Debug, Clone, Serialize, PartialEq, Deserialize)]
pub(crate) struct NavigationLink {
    /// URL for htmx requests.
    pub hx_url: String,
    /// Standard URL for navigation.
    pub url: String,
}

impl NavigationLink {
    /// Creates a navigation link from the given filters.
    pub(crate) fn new<T>(filters: &T) -> Result<Self>
    where
        T: Serialize + Pagination,
    {
        let base_hx_url = filters.get_base_hx_url();
        let base_url = filters.get_base_url();

        Ok(NavigationLink {
            hx_url: build_url(&base_hx_url, filters)?,
            url: build_url(&base_url, filters)?,
        })
    }
}

/// Holds offset values for building pagination navigation links.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct NavigationLinksOffsets {
    /// Offset for the first page.
    first: Option<usize>,
    /// Offset for the last page.
    last: Option<usize>,
    /// Offset for the next page.
    next: Option<usize>,
    /// Offset for the previous page.
    prev: Option<usize>,
}

impl NavigationLinksOffsets {
    /// Calculates offsets for navigation links based on current state.
    fn new(offset: Option<usize>, limit: Option<usize>, total: usize) -> Self {
        let mut offsets = NavigationLinksOffsets::default();

        // Use default offset and limit values if not provided
        let offset = offset.unwrap_or(0);
        let limit = limit.unwrap_or(DEFAULT_PAGINATION_LIMIT);

        // There are more results going backwards
        if offset > 0 {
            // First
            offsets.first = Some(0);

            // Previous
            offsets.prev = Some(offset.saturating_sub(limit));
        }

        // There are more results going forward
        if total.saturating_sub(offset + limit) > 0 {
            // Next
            offsets.next = Some(offset + limit);

            // Last
            offsets.last = if total.is_multiple_of(limit) {
                Some(total - limit)
            } else {
                Some(total - (total % limit))
            };
        }

        offsets
    }
}

/// Builds a URL with query parameters for the given filters.
pub(crate) fn build_url<T>(base_url: &str, filters: &T) -> Result<String>
where
    T: Serialize + Pagination,
{
    let sep = get_url_filters_separator(base_url);
    let filters_params = serde_qs::to_string(filters)?;

    let mut url = base_url.to_string();
    if !filters_params.is_empty() {
        write!(url, "{sep}{filters_params}")?;
    }

    Ok(url)
}

/// Determines the separator to use when appending filters to a URL.
fn get_url_filters_separator(url: &str) -> &str {
    if url.contains('?') {
        if url.ends_with('?') || url.ends_with('&') {
            ""
        } else {
            "&"
        }
    } else {
        "?"
    }
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_PAGINATION_LIMIT, NavigationLinksOffsets, get_url_filters_separator};

    // Macro for generating tests for NavigationLinksOffsets logic.
    macro_rules! navigation_links_offsets_tests {
        ($(
            $name:ident: {
                offset: $offset:expr,
                limit: $limit:expr,
                total: $total:expr,
                expected_offsets: $expected_offsets:expr
            }
        ,)*) => {
        $(
            #[test]
            fn $name() {
                let offsets = NavigationLinksOffsets::new($offset, $limit, $total);
                assert_eq!(offsets, $expected_offsets);
            }
        )*
        }
    }

    navigation_links_offsets_tests! {
        test_navigation_links_offsets_1: {
            offset: Some(0),
            limit: Some(10),
            total: 20,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: Some(10),
                next: Some(10),
                prev: None,
            }
        },

        test_navigation_links_offsets_2: {
            offset: Some(10),
            limit: Some(10),
            total: 20,
            expected_offsets: NavigationLinksOffsets {
                first: Some(0),
                last: None,
                next: None,
                prev: Some(0),
            }
        },

        test_navigation_links_offsets_3: {
            offset: Some(0),
            limit: Some(10),
            total: 21,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: Some(20),
                next: Some(10),
                prev: None,
            }
        },

        test_navigation_links_offsets_4: {
            offset: Some(10),
            limit: Some(10),
            total: 15,
            expected_offsets: NavigationLinksOffsets {
                first: Some(0),
                last: None,
                next: None,
                prev: Some(0),
            }
        },

        test_navigation_links_offsets_5: {
            offset: Some(0),
            limit: Some(10),
            total: 10,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: None,
                next: None,
                prev: None,
            }
        },

        test_navigation_links_offsets_6: {
            offset: Some(0),
            limit: Some(10),
            total: 5,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: None,
                next: None,
                prev: None,
            }
        },

        test_navigation_links_offsets_7: {
            offset: Some(0),
            limit: Some(10),
            total: 0,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: None,
                next: None,
                prev: None,
            }
        },

        test_navigation_links_offsets_8: {
            offset: None,
            limit: Some(10),
            total: 15,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: Some(10),
                next: Some(10),
                prev: None,
            }
        },

        test_navigation_links_offsets_9: {
            offset: None,
            limit: None,
            total: 15,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: Some(DEFAULT_PAGINATION_LIMIT),
                next: Some(DEFAULT_PAGINATION_LIMIT),
                prev: None,
            }
        },

        test_navigation_links_offsets_10: {
            offset: Some(20),
            limit: Some(10),
            total: 50,
            expected_offsets: NavigationLinksOffsets {
                first: Some(0),
                last: Some(40),
                next: Some(30),
                prev: Some(10),
            }
        },

        test_navigation_links_offsets_11: {
            offset: Some(2),
            limit: Some(10),
            total: 20,
            expected_offsets: NavigationLinksOffsets {
                first: Some(0),
                last: Some(10),
                next: Some(12),
                prev: Some(0),
            }
        },

        test_navigation_links_offsets_12: {
            offset: Some(0),
            limit: Some(10),
            total: 5,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: None,
                next: None,
                prev: None,
            }
        },

        test_navigation_links_offsets_13: {
            offset: Some(0),
            limit: Some(10),
            total: 11,
            expected_offsets: NavigationLinksOffsets {
                first: None,
                last: Some(10),
                next: Some(10),
                prev: None,
            }
        },
    }

    // Macro for generating tests for get_url_filters_separator logic.
    macro_rules! get_url_filters_separator_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let (url, expected_sep) = $value;
                assert_eq!(get_url_filters_separator(url), expected_sep);
            }
        )*
        }
    }

    get_url_filters_separator_tests! {
        test_get_url_filters_separator_1: ("https://example.com", "?"),
        test_get_url_filters_separator_2: ("https://example.com?", ""),
        test_get_url_filters_separator_3: ("https://example.com?param1=value1", "&"),
        test_get_url_filters_separator_4: ("https://example.com?param1=value1&", ""),
    }
}
