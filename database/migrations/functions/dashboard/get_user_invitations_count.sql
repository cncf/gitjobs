-- Returns the number of pending team invitations for the user.
create or replace function dashboard_employer_get_user_invitations_count(p_user_id uuid)
returns bigint as $$
    select count(*)
    from employer_team et
    where et.user_id = p_user_id
    and et.approved = false;
$$ language sql;
