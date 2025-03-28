//! This module defines some templates and types used in the about page.

use askama::Template;
use serde::{Deserialize, Serialize};

// Pages templates.

/// About page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/about/page.html")]
pub(crate) struct Page {
    pub content: String,
}
