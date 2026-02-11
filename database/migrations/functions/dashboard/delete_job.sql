-- Soft-deletes a job.
create or replace function delete_job(p_job_id uuid)
returns void as $$
    update job
    set
        status = 'deleted',
        deleted_at = current_timestamp
    where job_id = p_job_id;
$$ language sql;
