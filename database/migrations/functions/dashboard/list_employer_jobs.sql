-- Returns all non-deleted jobs for an employer.
create or replace function list_employer_jobs(p_employer_id uuid)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'job_id', j.job_id,
        'created_at', j.created_at,
        'title', j.title,
        'status', j.status,
        'workplace', j.workplace,
        'archived_at', j.archived_at,
        'published_at', j.published_at,
        'review_notes', j.review_notes,
        'city', l.city,
        'country', l.country
    ) order by j.published_at desc, j.created_at desc), '[]'::json)
    from job j
    left join location l using (location_id)
    where j.employer_id = p_employer_id
    and j.status <> 'deleted';
$$ language sql;
