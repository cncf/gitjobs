//! This module defines some types and functionality used to paginate.

use anyhow::Result;
use rinja::Template;
use serde::{Deserialize, Serialize};

/// Default pagination limit.
const DEFAULT_PAGINATION_LIMIT: usize = 10;

/// Trait to get and set some pagination values.
pub(crate) trait Pagination {
    /// Get the base url for htmx requests.
    fn get_base_hx_url(&self) -> String;

    /// Get the base url.
    fn get_base_url(&self) -> String;

    /// Get the limit value.
    fn limit(&self) -> Option<usize>;

    /// Get the offset value.
    fn offset(&self) -> Option<usize>;

    /// Set the offset value.
    fn set_offset(&mut self, offset: Option<usize>);
}

/// Pagination navigation links.
#[derive(Debug, Clone, Default, Template, Serialize, Deserialize)]
#[template(path = "navigation_links.html")]
pub(crate) struct NavigationLinks {
    pub first: Option<NavigationLink>,
    pub last: Option<NavigationLink>,
    pub next: Option<NavigationLink>,
    pub prev: Option<NavigationLink>,
}

impl NavigationLinks {
    /// Create a new `NavigationLinks` instance from the filters provided.
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

/// Navigation link.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct NavigationLink {
    pub hx_url: String,
    pub url: String,
}

impl NavigationLink {
    /// Create a new `NavigationLink` instance from the filters provided.
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

/// Offsets used to build the navigation links.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
struct NavigationLinksOffsets {
    first: Option<usize>,
    last: Option<usize>,
    next: Option<usize>,
    prev: Option<usize>,
}

impl NavigationLinksOffsets {
    /// Create a new `NavigationLinksOffsets` instance.
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
            offsets.last = if total % limit == 0 {
                Some(total - limit)
            } else {
                Some(total - (total % limit))
            };
        }

        offsets
    }
}

/// Build URL that includes the filters as query parameters.
pub(crate) fn build_url<T>(base_url: &str, filters: &T) -> Result<String>
where
    T: Serialize + Pagination,
{
    let mut url = base_url.to_string();
    let sep = get_url_filters_separator(&url);
    let filters_params = serde_qs::to_string(filters)?;
    if !filters_params.is_empty() {
        url.push_str(&format!("{sep}{filters_params}"));
    }
    Ok(url)
}

/// Get the separator to use when joining the filters to the url.
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
