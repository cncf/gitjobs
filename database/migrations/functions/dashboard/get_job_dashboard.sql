-- Returns full job details for employer dashboard.
create or replace function get_job_dashboard(p_job_id uuid)
returns json as $$
    select nullif(jsonb_strip_nulls(jsonb_build_object(
        'description', j.description,
        'status', j.status,
        'title', j.title,
        'kind', j.kind,
        'workplace', j.workplace,
        'apply_instructions', j.apply_instructions,
        'apply_url', j.apply_url,
        'benefits', j.benefits,
        'job_id', j.job_id,
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
            left join job_project jp using (project_id)
            left join job j2 using (job_id)
            where j2.job_id = p_job_id
        ),
        'published_at', j.published_at,
        'qualifications', j.qualifications,
        'responsibilities', j.responsibilities,
        'review_notes', j.review_notes,
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
                'url', c.url,
                'logo_url', c.logo_url
            ))
            from certification c
            left join job_certification jc using (certification_id)
            left join job j2 using (job_id)
            where j2.job_id = p_job_id
        ),
        'tz_end', j.tz_end,
        'tz_start', j.tz_start,
        'updated_at', j.updated_at,
        'upstream_commitment', j.upstream_commitment
    )), '{}'::jsonb)
    from job j
    left join location l using (location_id)
    where j.job_id = p_job_id
    and j.status <> 'deleted';
$$ language sql;
