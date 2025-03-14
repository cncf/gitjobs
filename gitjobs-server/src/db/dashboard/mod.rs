//! This module defines some database functionality for the dashboards.

use async_trait::async_trait;
use employer::DBDashBoardEmployer;
use job_seeker::DBDashBoardJobSeeker;

use crate::PgDB;

pub(crate) mod employer;
pub(crate) mod job_seeker;

/// Trait that defines some database operations used in the dashboards.
#[async_trait]
pub(crate) trait DBDashBoard: DBDashBoardEmployer + DBDashBoardJobSeeker {}

impl DBDashBoard for PgDB {}
