//! This module defines some templates and types used in the about page.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{PageId, auth::User, filters};

// Pages templates.

/// About page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/about/page.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub content: String,
    pub page_id: PageId,
    pub user: User,
}
