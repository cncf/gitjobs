//! This module defines some templates and types used in the employer dashboard
//! team page.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::helpers::DATE_FORMAT;

// Pages templates.

/// Team members list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/teams/members_list.html")]
pub(crate) struct MembersListPage {
    pub members: Vec<TeamMember>,
}

/// User invitations list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/teams/invitations_list.html")]
pub(crate) struct UserInvitationsListPage {
    pub invitations: Vec<TeamInvitation>,
}

// Types.

/// Team invitation information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamInvitation {
    pub company: String,
    pub created_at: DateTime<Utc>,
    pub employer_id: Uuid,
}

/// Team member information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamMember {
    pub approved: bool,
    pub email: String,
    pub name: String,
    pub user_id: Uuid,
    pub username: String,
}

/// New team member information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct NewTeamMember {
    pub email: String,
}
