//! This module defines some templates and types used in the home page.

use askama::Template;
use serde::{Deserialize, Serialize};

use crate::templates::{PageId, filters};

// Pages templates.

/// Home page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/home.html")]
#[allow(clippy::struct_field_names)]
pub(crate) struct Page {
    pub logged_in: bool,
    pub page_id: PageId,

    pub name: Option<String>,
    pub username: Option<String>,
}
