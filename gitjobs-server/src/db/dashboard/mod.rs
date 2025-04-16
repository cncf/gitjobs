//! This module defines some database functionality for the dashboards.

use async_trait::async_trait;
use employer::DBDashBoardEmployer;
use job_seeker::DBDashBoardJobSeeker;
use moderator::DBDashBoardModerator;

use crate::PgDB;

pub(crate) mod employer;
pub(crate) mod job_seeker;
pub(crate) mod moderator;

/// Trait that defines some database operations used in the dashboards.
#[async_trait]
pub(crate) trait DBDashBoard:
    DBDashBoardEmployer + DBDashBoardJobSeeker + DBDashBoardModerator
{
}

impl DBDashBoard for PgDB {}
