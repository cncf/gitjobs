//! This module defines some templates and types used in the jobs page.

use rinja::Template;
use serde::{Deserialize, Serialize};

use crate::templates::PageId;

/// Home page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/home.html")]
pub(crate) struct Page {
    pub page_id: PageId,
}
