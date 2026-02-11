-- Returns filter options for employer applications.
create or replace function dashboard_employer_get_applications_filters_options(p_employer_id uuid)
returns json as $$
    select json_build_object(
        'jobs',
        coalesce(json_agg(json_build_object(
            'job_id', j.job_id,
            'created_at', j.created_at,
            'title', j.title,
            'status', j.status,
            'workplace', j.workplace,
            'city', l.city,
            'country', l.country
        ) order by j.created_at desc), '[]'::json)
    )
    from job j
    left join location l using (location_id)
    where j.employer_id = p_employer_id
    and j.status <> 'deleted';
$$ language sql;
