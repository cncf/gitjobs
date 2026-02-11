-- Returns all job applications for the provided user.
create or replace function dashboard_job_seeker_list_applications(p_user_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'application_id', a.application_id,
        'applied_at', a.applied_at,
        'job_id', a.job_id,
        'job_location', a.job_location,
        'job_status', a.job_status,
        'job_title', a.job_title,
        'job_workplace', a.job_workplace
    ) order by a.applied_at desc), '[]'::json)
    from (
        select
            ap.application_id,
            ap.created_at as applied_at,
            ap.job_id,
            nullif(jsonb_strip_nulls(jsonb_build_object(
                'city', l.city,
                'country', l.country,
                'location_id', l.location_id,
                'state', l.state
            )), '{}'::jsonb) as job_location,
            j.status as job_status,
            j.title as job_title,
            j.workplace as job_workplace
        from application ap
        join job j on ap.job_id = j.job_id
        join job_seeker_profile p on ap.job_seeker_profile_id = p.job_seeker_profile_id
        left join location l on j.location_id = l.location_id
        where p.user_id = p_user_id
    ) a;
$$ language sql;
