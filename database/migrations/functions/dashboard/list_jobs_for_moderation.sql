-- Returns jobs for moderation filtered by status.
create or replace function list_jobs_for_moderation(p_status text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'created_at', j.created_at,
        'employer', jsonb_strip_nulls(jsonb_build_object(
            'company', e.company,
            'employer_id', e.employer_id,
            'logo_id', e.logo_id,
            'members', members.members,
            'website_url', e.website_url
        )),
        'job_id', j.job_id,
        'title', j.title
    ) order by j.created_at desc), '[]'::json)
    from job j
    join employer e on j.employer_id = e.employer_id
    left join lateral (
        select jsonb_agg(jsonb_build_object(
            'foundation', m.foundation,
            'level', m.level,
            'logo_url', m.logo_url,
            'member_id', m.member_id,
            'name', m.name
        ) order by m.foundation asc, m.name asc) as members
        from employer_member em
        join member m on em.member_id = m.member_id
        where em.employer_id = e.employer_id
    ) members on true
    where j.status = p_status;
$$ language sql;
