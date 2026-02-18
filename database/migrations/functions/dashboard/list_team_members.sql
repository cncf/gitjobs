-- Returns team members for an employer.
create or replace function list_team_members(p_employer_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'approved', et.approved,
        'email', u.email,
        'name', u.name,
        'user_id', u.user_id,
        'username', u.username
    ) order by u.name asc), '[]'::json)
    from employer_team et
    join "user" u using (user_id)
    where et.employer_id = p_employer_id;
$$ language sql;
