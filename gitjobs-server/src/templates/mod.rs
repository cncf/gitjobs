//! This module defines the templates used to render the different parts of the
//! job boards sites.

use serde::{Deserialize, Serialize};

pub(crate) mod auth;
pub(crate) mod dashboard;
mod filters;
mod helpers;
pub(crate) mod jobboard;
pub(crate) mod misc;
pub(crate) mod notifications;
pub(crate) mod pagination;

/// Identifier for a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PageId {
    EmployerDashboard,
    JobBoard,
    JobSeekerDashboard,
    LogIn,
    SignUp,
}
