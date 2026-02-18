-- Returns employers where the user is an approved team member.
create or replace function list_employers(p_user_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'employer_id', e.employer_id,
        'company', e.company,
        'logo_id', e.logo_id
    ) order by e.company asc), '[]'::json)
    from employer e
    join employer_team et using (employer_id)
    where et.user_id = p_user_id
    and et.approved = true;
$$ language sql;
