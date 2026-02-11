-- Archives a pending or published job.
create or replace function dashboard_employer_archive_job(p_job_id uuid)
returns void as $$
    update job
    set
        status = 'archived',
        archived_at = current_timestamp,
        updated_at = current_timestamp
    where job_id = p_job_id
    and (status = 'pending-approval' or status = 'published');
$$ language sql;
