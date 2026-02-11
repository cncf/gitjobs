-- Returns the owner user id for the provided job seeker profile.
create or replace function get_job_seeker_user_id(p_job_seeker_profile_id uuid)
returns uuid as $$
    select p.user_id
    from job_seeker_profile p
    where p.job_seeker_profile_id = p_job_seeker_profile_id;
$$ language sql;
