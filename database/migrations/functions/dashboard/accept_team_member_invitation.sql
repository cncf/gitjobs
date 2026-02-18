-- Accepts a pending team member invitation.
create or replace function accept_team_member_invitation(
    p_employer_id uuid,
    p_user_id uuid
)
returns void as $$
    update employer_team
    set approved = true
    where employer_id = p_employer_id
    and user_id = p_user_id;
$$ language sql;
