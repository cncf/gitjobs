//! This module defines some templates and types used in the employer dashboard
//! team page.

use askama::Template;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Add member page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/team/add_member.html")]
pub(crate) struct AddMemberPage {}

/// Team members list page template.
#[derive(Debug, Clone, Template, Serialize, Deserialize)]
#[template(path = "dashboard/employer/team/members_list.html")]
pub(crate) struct MembersListPage {
    pub team_members: Vec<TeamMember>,
}

/// Team member information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct TeamMember {
    pub email: String,
    pub invitation_accepted: bool,
    pub name: String,
    pub user_id: Uuid,
    pub username: String,
}

/// New team member information.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub(crate) struct NewTeamMember {
    pub email: String,
}
