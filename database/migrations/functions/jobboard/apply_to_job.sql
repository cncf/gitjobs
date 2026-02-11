-- Applies the user profile to a published job.
create or replace function jobboard_apply_to_job(p_job_id uuid, p_user_id uuid)
returns boolean as $$
    with inserted_application as (
        insert into application (
            job_id,
            job_seeker_profile_id
        )
        select
            j.job_id,
            p.job_seeker_profile_id
        from job j
        join job_seeker_profile p on p.user_id = p_user_id
        where j.job_id = p_job_id
        and j.status = 'published'
        on conflict (job_seeker_profile_id, job_id) do nothing
        returning application_id
    )
    select exists (select 1 from inserted_application);
$$ language sql;
