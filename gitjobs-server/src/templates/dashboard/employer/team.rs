//! Templates and types for the employer dashboard team page.

use askama::Template;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::templates::helpers::DATE_FORMAT;

// Pages templates.

/// Template for the team members list page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/teams/members_list.html")]
pub(crate) struct MembersListPage {
    /// Count of approved team members.
    pub approved_members_count: usize,
    /// List of team members.
    pub members: Vec<TeamMember>,
}

/// Template for the user invitations list page.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/teams/invitations_list.html")]
pub(crate) struct UserInvitationsListPage {
    /// List of team invitations.
    pub invitations: Vec<TeamInvitation>,
}

// Types.

/// Information about a team invitation.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamInvitation {
    /// Name of the company.
    pub company: String,
    /// Timestamp when the invitation was created.
    pub created_at: DateTime<Utc>,
    /// ID of the employer who sent the invitation.
    pub employer_id: Uuid,
}

/// Information about a team member.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamMember {
    /// Whether the member is approved.
    pub approved: bool,
    /// Email address of the member.
    pub email: String,
    /// Full name of the member.
    pub name: String,
    /// Unique ID of the user.
    pub user_id: Uuid,
    /// Username of the member.
    pub username: String,
}

/// Information for adding a new team member.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct NewTeamMember {
    /// Email address of the new member.
    pub email: String,
}
