-- Returns pending team invitations for a user.
create or replace function dashboard_employer_list_user_invitations(p_user_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'company', e.company,
        'created_at', e.created_at,
        'employer_id', e.employer_id
    ) order by e.created_at desc), '[]'::json)
    from employer_team et
    join employer e using (employer_id)
    where et.user_id = p_user_id
    and et.approved = false;
$$ language sql;
