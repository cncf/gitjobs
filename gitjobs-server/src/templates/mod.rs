//! This module defines the templates used to render the different parts of the
//! job board.

use serde::{Deserialize, Serialize};

pub(crate) mod auth;
pub(crate) mod dashboard;
pub(crate) mod filters;
pub(crate) mod helpers;
pub(crate) mod jobboard;
pub(crate) mod misc;
pub(crate) mod notifications;
pub(crate) mod pagination;

/// Identifier for a page.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum PageId {
    About,
    EmployerDashboard,
    JobBoard,
    JobSeekerDashboard,
    LogIn,
    NotFound,
    SignUp,
}
