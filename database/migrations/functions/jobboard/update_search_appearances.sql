-- update_search_appearances updates the search appearances of the jobs provided.
create or replace function jobboard_update_search_appearances(p_lock_key bigint, p_data jsonb)
returns void as $$
    -- Make sure only one batch of updates is processed at a time
    select pg_advisory_xact_lock(p_lock_key);

    -- Insert or update the corresponding search appearances counters as needed
    insert into search_appearances (job_id, day, total)
    select appearances_batch.*
    from (
        select
            (value->>0)::uuid as job_id,
            (value->>1)::date as day,
            (value->>2)::integer as total
        from jsonb_array_elements(p_data)
    ) as appearances_batch
    join job on job.job_id = appearances_batch.job_id
    where job.status = 'published'
    on conflict (job_id, day) do
    update set total = search_appearances.total + excluded.total;
$$ language sql;
