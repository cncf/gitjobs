//! This module defines some templates and types used in the moderator
//! dashboard jobs pages.

use askama::Template;
use serde::{Deserialize, Serialize};

// Pages templates.

/// Pending jobs page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/moderator/pending_jobs.html")]
pub(crate) struct PendingPage {}
