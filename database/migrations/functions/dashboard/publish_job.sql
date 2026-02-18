-- Publishes a job and refreshes normalized salary values.
create or replace function publish_job(
    p_job_id uuid,
    p_salary_usd_year bigint,
    p_salary_min_usd_year bigint,
    p_salary_max_usd_year bigint
)
returns void as $$
    update job
    set
        status = 'pending-approval',
        updated_at = current_timestamp,
        archived_at = null,
        salary_usd_year = p_salary_usd_year,
        salary_min_usd_year = p_salary_min_usd_year,
        salary_max_usd_year = p_salary_max_usd_year
    where job_id = p_job_id
    and (status = 'archived' or status = 'draft' or status = 'rejected');
$$ language sql;
