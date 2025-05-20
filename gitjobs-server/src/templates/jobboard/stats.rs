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
    /// Authenticated user information.
    pub user: User,
    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}
