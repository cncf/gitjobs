//! This module defines some templates and types used in the about page.

use rinja::Template;
use serde::{Deserialize, Serialize};

/// About page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/about/page.html")]
pub(crate) struct Page {}
