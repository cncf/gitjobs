//! This module defines the templates used to render the different parts of the
//! job boards sites.

use serde::{Deserialize, Serialize};

pub(crate) mod auth;
pub(crate) mod dashboard;
mod filters;
mod helpers;
pub(crate) mod jobboard;
pub(crate) mod misc;

/// The current page being rendered.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CurrentPage {
    EmployerDashboard,
    JobBoard,
    JobSeekerDashboard,
    LogIn,
    SignUp,
}
