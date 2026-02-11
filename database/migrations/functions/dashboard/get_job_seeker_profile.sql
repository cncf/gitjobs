-- Returns the job seeker profile for the provided user.
create or replace function get_job_seeker_profile(p_user_id uuid)
returns json as $$
    select nullif(jsonb_strip_nulls(jsonb_build_object(
        'bluesky_url', p.bluesky_url,
        'certifications', p.certifications,
        'education', p.education,
        'email', p.email,
        'experience', p.experience,
        'facebook_url', p.facebook_url,
        'github_url', p.github_url,
        'linkedin_url', p.linkedin_url,
        'location', nullif(jsonb_strip_nulls(jsonb_build_object(
            'city', l.city,
            'country', l.country,
            'location_id', l.location_id,
            'state', l.state
        )), '{}'::jsonb),
        'name', p.name,
        'open_to_relocation', p.open_to_relocation,
        'open_to_remote', p.open_to_remote,
        'phone', p.phone,
        'photo_id', p.photo_id,
        'projects', p.projects,
        'public', p.public,
        'skills', p.skills,
        'summary', p.summary,
        'twitter_url', p.twitter_url,
        'website_url', p.website_url
    )), '{}'::jsonb)
    from job_seeker_profile p
    left join location l using (location_id)
    where p.user_id = p_user_id;
$$ language sql;
