-- Archives published jobs that expired more than 30 days ago.
create or replace function archive_expired_jobs()
returns void as $$
    update job
    set
        status = 'archived',
        archived_at = current_timestamp,
        updated_at = current_timestamp
    where status = 'published'
    and published_at + '30 days'::interval < current_timestamp;
$$ language sql;
