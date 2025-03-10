-- Check if the user has access to the image provided.
create or replace function user_has_image_access(p_user_id uuid, p_image_id uuid)
returns boolean as $$
begin
    -- Profile photo or employer logo: user created the image
    perform from image
    where image_id = p_image_id
    and created_by = p_user_id;
    if found then return true; end if;

    -- Profile photo: applied to a employer's job
    perform from job_seeker_profile p
    join applicant a on p.job_seeker_profile_id = a.job_seeker_profile_id
    join job j on a.job_id = j.job_id
    join employer_team et on j.employer_id = et.employer_id
    where p.photo_id = p_image_id
    and et.user_id = p_user_id;

    -- Employer logo: user belongs to the employer team
    perform from employer e
    join employer_team et using (employer_id)
    where e.logo_id = p_image_id
    and et.user_id = p_user_id;
    return found;
end
$$ language plpgsql;
