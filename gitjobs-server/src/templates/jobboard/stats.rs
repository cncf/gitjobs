//! Templates and types for the job board about page.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{Config, PageId, auth::User};

// Pages templates.

/// Template for the stats page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/stats/page.html")]
pub(crate) struct Page {
    /// Server configuration.
    pub cfg: Config,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Stats information in JSON format.
    pub stats: Stats,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}

// Types.

/// Stats information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct Stats {
    /// Jobs statistics.
    pub jobs: JobsStats,
    /// Timestamp representing the current time.
    pub ts_now: Timestamp,
    /// Timestamp representing one month ago.
    pub ts_one_month_ago: Timestamp,
    /// Timestamp representing two years ago.
    pub ts_two_years_ago: Timestamp,
}

/// Jobs statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub(crate) struct JobsStats {
    /// Number of jobs published per foundation.
    /// Each entry is a tuple of (foundation, count).
    pub published_per_foundation: Vec<(String, Total)>,

    /// Number of jobs published per month.
    /// Each entry is a tuple of (month, year, count).
    pub published_per_month: Vec<(String, String, Total)>,

    /// Running total of published jobs.
    /// Each entry is a tuple of (timestamp, count).
    pub published_running_total: Vec<(Timestamp, Total)>,

    /// Number of job views per day.
    /// Each entry is a tuple of (timestamp, count).
    pub views_daily: Vec<(Timestamp, Total)>,

    /// Number of job views per month.
    /// Each entry is a tuple of (timestamp, count).
    pub views_monthly: Vec<(Timestamp, Total)>,
}

/// Type alias for a timestamp.
type Timestamp = u64;

/// Type alias for a total count.
type Total = u64;
