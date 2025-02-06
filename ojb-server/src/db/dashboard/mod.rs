//! This module defines some database functionality for the employer dashboard.

use async_trait::async_trait;
use employer::DBDashBoardEmployer;

use crate::PgDB;

mod employer;
mod job_seeker;

/// Trait that defines some database operations used in the employer dashboard.
#[async_trait]
pub(crate) trait DBDashBoard: DBDashBoardEmployer {}

impl DBDashBoard for PgDB {}
