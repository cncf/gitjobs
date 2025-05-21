//! Templates and types for the job board about page.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{Config, PageId, auth::User, filters};

// Pages templates.

/// Template for the about page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/about/page.html")]
pub(crate) struct Page {
    /// Server configuration.
    pub cfg: Config,
    /// About page content (rendered from markdown source).
    pub content: String,
    /// Identifier for the current page.
    pub page_id: PageId,
    /// Authenticated user information.
    pub user: User,

    /// Name of the authentication provider, if any.
    pub auth_provider: Option<String>,
}
