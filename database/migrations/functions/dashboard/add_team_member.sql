-- Adds a team member invitation by user email.
create or replace function dashboard_employer_add_team_member(
    p_employer_id uuid,
    p_email text
)
returns uuid as $$
    insert into employer_team (
        employer_id,
        user_id,
        approved
    )
    select
        p_employer_id,
        user_id,
        false
    from "user"
    where email = p_email
    on conflict do nothing
    returning user_id;
$$ language sql;
