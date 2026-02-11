-- update_jobs_views updates the views of the jobs provided.
create or replace function jobboard_update_jobs_views(p_lock_key bigint, p_data jsonb)
returns void as $$
    -- Make sure only one batch of updates is processed at a time
    select pg_advisory_xact_lock(p_lock_key);

    -- Insert or update the corresponding views counters as needed
    insert into job_views (job_id, day, total)
    select views_batch.*
    from (
        select
            (value->>0)::uuid as job_id,
            (value->>1)::date as day,
            (value->>2)::integer as total
        from jsonb_array_elements(p_data)
    ) as views_batch
    join job on job.job_id = views_batch.job_id
    where job.status = 'published'
    on conflict (job_id, day) do
    update set total = job_views.total + excluded.total;
$$ language sql;
