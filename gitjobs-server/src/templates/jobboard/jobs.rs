//! This module defines some templates and types used in the jobs pages.

use anyhow::Result;
use chrono::{DateTime, Utc};
use rinja::Template;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tracing::trace;
use uuid::Uuid;

use crate::templates::{
    PageId,
    dashboard::employer::jobs::{JobKind, Workplace},
    misc::{Location, Member, Project},
};

/// Default pagination limit.
const DEFAULT_PAGINATION_LIMIT: usize = 10;

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/jobs.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct JobsPage {
    pub explore_section: ExploreSection,
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Explore section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/explore_section.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ExploreSection {
    pub filters: Filters,
    pub filters_options: FiltersOptions,
    pub results_section: ResultsSection,
}

/// Filters used in the jobs explore section.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Filters {
    pub benefits: Option<Vec<String>>,
    pub kind: Option<Vec<JobKind>>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
    pub open_source: Option<usize>,
    pub skills: Option<Vec<String>>,
    pub sort_by: Option<String>,
    pub ts_query: Option<String>,
    pub upstream_commitment: Option<usize>,
    pub workplace: Option<Vec<Workplace>>,
}

impl Filters {
    /// Create a new `Filters` instance from the raw query string provided.
    pub(crate) fn new(raw_query: &str) -> Result<Self> {
        let filters: Filters = serde_qs::from_str(raw_query)?;

        trace!("{:?}", filters);
        Ok(filters)
    }

    /// Convert the filters to a raw query string.
    fn to_raw_query(&self) -> Result<String> {
        serde_qs::to_string(self).map_err(Into::into)
    }
}

/// Filters options used in the jobs explore section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FiltersOptions {
    pub kind: Vec<FilterOption>,
    pub workplace: Vec<FilterOption>,
}

/// Filter option details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct FilterOption {
    pub name: String,
    pub value: String,
}

/// Results section template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/results_section.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct ResultsSection {
    pub jobs: Vec<JobSummary>,
    pub navigation_links: NavigationLinks,
    pub total: usize,

    pub offset: Option<usize>,
}

/// Job summary.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobSummary {
    pub employer: Employer,
    pub job_id: uuid::Uuid,
    pub kind: JobKind,
    pub title: String,
    pub workplace: Workplace,

    pub location: Option<Location>,
    pub open_source: Option<i32>,
    pub projects: Option<Vec<Project>>,
    pub published_at: Option<DateTime<Utc>>,
    pub salary: Option<i64>,
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_period: Option<String>,
    pub updated_at: Option<DateTime<Utc>>,
    pub upstream_commitment: Option<i32>,
}

/// Employer details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Employer {
    pub company: String,
    pub employer_id: Uuid,

    pub member: Option<Member>,
    pub website_url: Option<String>,
}

/// Results navigation links.
#[derive(Debug, Clone, Default, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/navigation_links.html")]
pub(crate) struct NavigationLinks {
    pub first: Option<NavigationLink>,
    pub last: Option<NavigationLink>,
    pub next: Option<NavigationLink>,
    pub prev: Option<NavigationLink>,
}

impl NavigationLinks {
    /// Create a new `NavigationLinks` instance from the filters provided.
    pub(crate) fn from_filters(filters: &Filters, total: usize) -> Result<Self> {
        let mut links = NavigationLinks::default();

        let offsets = NavigationLinksOffsets::new(filters.offset, filters.limit, total);
        let mut filters = filters.clone();

        if let Some(first_offset) = offsets.first {
            filters.offset = Some(first_offset);
            links.first = Some(NavigationLink::new(&filters)?);
        }
        if let Some(last_offset) = offsets.last {
            filters.offset = Some(last_offset);
            links.last = Some(NavigationLink::new(&filters)?);
        }
        if let Some(next_offset) = offsets.next {
            filters.offset = Some(next_offset);
            links.next = Some(NavigationLink::new(&filters)?);
        }
        if let Some(prev_offset) = offsets.prev {
            filters.offset = Some(prev_offset);
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
    pub(crate) fn new(filters: &Filters) -> Result<Self> {
        Ok(NavigationLink {
            hx_url: build_url("/jobs/section/results", filters)?,
            url: build_url("/jobs", filters)?,
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

/// Job page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/job.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct JobPage {
    pub job: Job,
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}

/// Build url that includes the filters provided as query parameters.
pub(crate) fn build_url(base_url: &str, filters: &Filters) -> Result<String> {
    let mut url = base_url.to_string();
    let sep = get_url_filters_separator(&url);
    let filters_params = filters.to_raw_query()?;
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

/// Job details.
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::struct_field_names)]
pub(crate) struct Job {
    pub description: String,
    pub employer: Employer,
    pub title: String,
    pub kind: JobKind,
    pub workplace: Workplace,

    pub apply_instructions: Option<String>,
    pub apply_url: Option<String>,
    pub benefits: Option<Vec<String>>,
    pub job_id: Option<Uuid>,
    pub location: Option<Location>,
    pub open_source: Option<i32>,
    pub projects: Option<Vec<Project>>,
    pub published_at: Option<DateTime<Utc>>,
    pub qualifications: Option<String>,
    pub responsibilities: Option<String>,
    pub salary: Option<i64>,
    pub salary_currency: Option<String>,
    pub salary_min: Option<i64>,
    pub salary_max: Option<i64>,
    pub salary_period: Option<String>,
    pub skills: Option<Vec<String>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub upstream_commitment: Option<i32>,
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
