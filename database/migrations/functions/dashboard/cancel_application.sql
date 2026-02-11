-- Cancels a job application for the given user.
create or replace function dashboard_job_seeker_cancel_application(
    p_application_id uuid,
    p_user_id uuid
)
returns void as $$
    delete from application
    where application_id in (
        select a.application_id
        from application a
        join job_seeker_profile p using (job_seeker_profile_id)
        where a.application_id = p_application_id
        and p.user_id = p_user_id
    );
$$ language sql;
