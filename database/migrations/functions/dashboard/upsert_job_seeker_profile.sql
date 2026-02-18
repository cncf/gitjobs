-- Upserts the job seeker profile for the provided user.
create or replace function upsert_job_seeker_profile(p_user_id uuid, p_profile jsonb)
returns void as $$
begin
    insert into job_seeker_profile (
        user_id,
        email,
        name,
        public,
        summary,
        bluesky_url,
        certifications,
        education,
        experience,
        facebook_url,
        github_url,
        linkedin_url,
        location_id,
        open_to_relocation,
        open_to_remote,
        phone,
        photo_id,
        projects,
        skills,
        twitter_url,
        website_url
    ) values (
        p_user_id,
        p_profile->>'email',
        p_profile->>'name',
        (p_profile->>'public')::boolean,
        p_profile->>'summary',
        p_profile->>'bluesky_url',
        nullif(p_profile->'certifications', 'null'::jsonb),
        nullif(p_profile->'education', 'null'::jsonb),
        nullif(p_profile->'experience', 'null'::jsonb),
        p_profile->>'facebook_url',
        p_profile->>'github_url',
        p_profile->>'linkedin_url',
        ((p_profile->'location')->>'location_id')::uuid,
        (p_profile->>'open_to_relocation')::boolean,
        (p_profile->>'open_to_remote')::boolean,
        p_profile->>'phone',
        (p_profile->>'photo_id')::uuid,
        nullif(p_profile->'projects', 'null'::jsonb),
        (
            case
                when p_profile ? 'skills' then
                    array(select jsonb_array_elements_text(p_profile->'skills'))
                else null
            end
        ),
        p_profile->>'twitter_url',
        p_profile->>'website_url'
    )
    on conflict (user_id) do update set
        email = excluded.email,
        name = excluded.name,
        public = excluded.public,
        summary = excluded.summary,
        bluesky_url = excluded.bluesky_url,
        certifications = excluded.certifications,
        education = excluded.education,
        experience = excluded.experience,
        facebook_url = excluded.facebook_url,
        github_url = excluded.github_url,
        linkedin_url = excluded.linkedin_url,
        location_id = excluded.location_id,
        open_to_relocation = excluded.open_to_relocation,
        open_to_remote = excluded.open_to_remote,
        phone = excluded.phone,
        photo_id = excluded.photo_id,
        projects = excluded.projects,
        skills = excluded.skills,
        twitter_url = excluded.twitter_url,
        website_url = excluded.website_url;
end
$$ language plpgsql;
