-- Approves a job and returns the previous first_published_at value.
create or replace function dashboard_moderator_approve_job(p_job_id uuid, p_reviewer uuid)
returns timestamptz as $$
    with old as (
        select first_published_at
        from job
        where job_id = p_job_id
    )
    update job
    set
        status = 'published',
        first_published_at = coalesce(first_published_at, current_timestamp),
        published_at = current_timestamp,
        reviewed_at = current_timestamp,
        reviewed_by = p_reviewer
    where job_id = p_job_id
    returning (select first_published_at from old);
$$ language sql;
