-- Returns the full published job payload for the job board.
create or replace function jobboard_get_job_jobboard(p_job_id uuid)
returns json as $$
    select nullif(jsonb_strip_nulls(jsonb_build_object(
        'description', j.description,
        'job_id', j.job_id,
        'kind', j.kind,
        'title', j.title,
        'workplace', j.workplace,
        'apply_instructions', j.apply_instructions,
        'apply_url', j.apply_url,
        'benefits', j.benefits,
        'location', nullif(jsonb_strip_nulls(jsonb_build_object(
            'location_id', l.location_id,
            'city', l.city,
            'country', l.country,
            'state', l.state
        )), '{}'::jsonb),
        'open_source', j.open_source,
        'projects', (
            select json_agg(json_build_object(
                'project_id', p.project_id,
                'foundation', p.foundation,
                'logo_url', p.logo_url,
                'maturity', p.maturity,
                'name', p.name
            ))
            from project p
            join job_project jp on p.project_id = jp.project_id
            where jp.job_id = j.job_id
        ),
        'published_at', j.published_at,
        'qualifications', j.qualifications,
        'responsibilities', j.responsibilities,
        'salary', j.salary,
        'salary_currency', j.salary_currency,
        'salary_min', j.salary_min,
        'salary_max', j.salary_max,
        'salary_period', j.salary_period,
        'seniority', j.seniority,
        'skills', j.skills,
        'certifications', (
            select json_agg(json_build_object(
                'certification_id', c.certification_id,
                'name', c.name,
                'provider', c.provider,
                'short_name', c.short_name,
                'description', c.description,
                'logo_url', c.logo_url,
                'url', c.url
            ))
            from certification c
            join job_certification jc on c.certification_id = jc.certification_id
            where jc.job_id = j.job_id
        ),
        'tz_end', j.tz_end,
        'tz_start', j.tz_start,
        'updated_at', j.updated_at,
        'upstream_commitment', j.upstream_commitment,
        'employer', nullif(jsonb_strip_nulls(jsonb_build_object(
            'company', e.company,
            'description', e.description,
            'employer_id', e.employer_id,
            'logo_id', e.logo_id,
            'members', members.members,
            'website_url', e.website_url
        )), '{}'::jsonb)
    )), '{}'::jsonb)
    from job j
    join employer e on j.employer_id = e.employer_id
    left join lateral (
        select
            jsonb_agg(jsonb_build_object(
                'member_id', m.member_id,
                'foundation', m.foundation,
                'level', m.level,
                'logo_url', m.logo_url,
                'name', m.name
            ) order by m.foundation asc, m.name asc) as members
        from employer_member em
        join member m on em.member_id = m.member_id
        where em.employer_id = e.employer_id
    ) members on true
    left join location l on j.location_id = l.location_id
    where j.job_id = p_job_id
    and j.status = 'published';
$$ language sql;
