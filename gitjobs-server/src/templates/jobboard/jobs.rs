//! This module defines some templates and types used in the jobs page.

use rinja::Template;
use serde::{Deserialize, Serialize};

/// Jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "jobboard/jobs/page.html")]
pub(crate) struct Page {}
