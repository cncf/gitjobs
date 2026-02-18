-- Checks whether the user can access the provided job seeker profile.
create or replace function user_has_profile_access(
    p_user_id uuid,
    p_job_seeker_profile_id uuid
)
returns boolean as $$
    select exists (
        select 1
        from job_seeker_profile p
        join application a on p.job_seeker_profile_id = a.job_seeker_profile_id
        join job j on a.job_id = j.job_id
        join employer_team et on j.employer_id = et.employer_id
        where et.user_id = p_user_id
        and p.job_seeker_profile_id = p_job_seeker_profile_id
        and et.approved = true
    );
$$ language sql;
