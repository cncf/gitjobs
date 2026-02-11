-- Checks whether the user belongs to the employer team that owns the job.
create or replace function auth_user_owns_job(p_user_id uuid, p_job_id uuid)
returns boolean as $$
    select exists (
        select 1
        from job j
        join employer_team et using (employer_id)
        where et.user_id = p_user_id
        and j.job_id = p_job_id
        and et.approved = true
    );
$$ language sql;
