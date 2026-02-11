-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set location1ID '00000000-0000-0000-0000-000000000101'
\set location2ID '00000000-0000-0000-0000-000000000102'
\set photo1ID '00000000-0000-0000-0000-000000000301'
\set photo2ID '00000000-0000-0000-0000-000000000302'
\set userID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into image (created_by, image_id) values
    (:'userID', :'photo1ID'),
    (:'userID', :'photo2ID');

insert into location (city, country, location_id, state) values
    ('Barcelona', 'Spain', :'location1ID', null),
    ('Bilbao', 'Spain', :'location2ID', null);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should insert profile when it does not exist
select dashboard_job_seeker_upsert_profile(
    :'userID'::uuid,
    jsonb_build_object(
        'bluesky_url', 'https://bsky.app/alice',
        'certifications', jsonb_build_array(jsonb_build_object('name', 'CKA')),
        'email', 'alice@example.com',
        'education', jsonb_build_array(jsonb_build_object('school', 'University One')),
        'experience', jsonb_build_array(jsonb_build_object('company', 'Acme', 'title', 'Engineer')),
        'facebook_url', 'https://facebook.com/alice',
        'github_url', 'https://github.com/alice',
        'linkedin_url', 'https://linkedin.com/in/alice',
        'location', jsonb_build_object('location_id', :'location1ID'::text),
        'name', 'Alice',
        'open_to_relocation', true,
        'open_to_remote', true,
        'phone', '+34 600 000 001',
        'photo_id', :'photo1ID'::text,
        'projects', jsonb_build_array(jsonb_build_object('name', 'Project One')),
        'public', true,
        'skills', jsonb_build_array('rust', 'sql'),
        'summary', 'Initial summary',
        'twitter_url', 'https://x.com/alice',
        'website_url', 'https://alice.dev'
    )
);

select is(
    (
        select jsonb_build_object(
            'bluesky_url', bluesky_url,
            'certifications', certifications,
            'education', education,
            'email', email,
            'experience', experience,
            'facebook_url', facebook_url,
            'github_url', github_url,
            'location_id', location_id::text,
            'linkedin_url', linkedin_url,
            'name', name,
            'open_to_relocation', open_to_relocation,
            'open_to_remote', open_to_remote,
            'phone', phone,
            'photo_id', photo_id::text,
            'projects', projects,
            'public', public,
            'skills', to_jsonb(skills),
            'summary', summary,
            'twitter_url', twitter_url,
            'website_url', website_url
        )
        from job_seeker_profile
        where user_id = :'userID'::uuid
    ),
    jsonb_build_object(
        'bluesky_url', 'https://bsky.app/alice',
        'certifications', jsonb_build_array(jsonb_build_object('name', 'CKA')),
        'education', jsonb_build_array(jsonb_build_object('school', 'University One')),
        'email', 'alice@example.com',
        'experience', jsonb_build_array(jsonb_build_object('company', 'Acme', 'title', 'Engineer')),
        'facebook_url', 'https://facebook.com/alice',
        'github_url', 'https://github.com/alice',
        'location_id', :'location1ID'::text,
        'linkedin_url', 'https://linkedin.com/in/alice',
        'name', 'Alice',
        'open_to_relocation', true,
        'open_to_remote', true,
        'phone', '+34 600 000 001',
        'photo_id', :'photo1ID'::text,
        'projects', jsonb_build_array(jsonb_build_object('name', 'Project One')),
        'public', true,
        'skills', jsonb_build_array('rust', 'sql'),
        'summary', 'Initial summary',
        'twitter_url', 'https://x.com/alice',
        'website_url', 'https://alice.dev'
    ),
    'Should insert profile when it does not exist'
);

-- Should update existing profile on conflict
select dashboard_job_seeker_upsert_profile(
    :'userID'::uuid,
    jsonb_build_object(
        'bluesky_url', 'https://bsky.app/alice-updated',
        'certifications', jsonb_build_array(jsonb_build_object('name', 'CKAD')),
        'email', 'alice@example.com',
        'education', jsonb_build_array(jsonb_build_object('school', 'University Two')),
        'experience', jsonb_build_array(jsonb_build_object('company', 'Beta', 'title', 'Senior Engineer')),
        'facebook_url', 'https://facebook.com/alice-updated',
        'github_url', 'https://github.com/alice-updated',
        'linkedin_url', 'https://linkedin.com/in/alice-updated',
        'location', jsonb_build_object('location_id', :'location2ID'::text),
        'name', 'Alice Updated',
        'open_to_relocation', false,
        'open_to_remote', false,
        'phone', '+34 600 000 002',
        'photo_id', :'photo2ID'::text,
        'projects', jsonb_build_array(jsonb_build_object('name', 'Project Two')),
        'public', false,
        'skills', jsonb_build_array('go'),
        'summary', 'Updated summary',
        'twitter_url', 'https://x.com/alice-updated',
        'website_url', 'https://alice-updated.dev'
    )
);

select is(
    (
        select jsonb_build_object(
            'bluesky_url', bluesky_url,
            'certifications', certifications,
            'education', education,
            'email', email,
            'experience', experience,
            'facebook_url', facebook_url,
            'github_url', github_url,
            'location_id', location_id::text,
            'name', name,
            'linkedin_url', linkedin_url,
            'open_to_relocation', open_to_relocation,
            'open_to_remote', open_to_remote,
            'phone', phone,
            'photo_id', photo_id::text,
            'projects', projects,
            'public', public,
            'skills', to_jsonb(skills),
            'summary', summary,
            'twitter_url', twitter_url,
            'website_url', website_url
        )
        from job_seeker_profile
        where user_id = :'userID'::uuid
    ),
    jsonb_build_object(
        'bluesky_url', 'https://bsky.app/alice-updated',
        'certifications', jsonb_build_array(jsonb_build_object('name', 'CKAD')),
        'education', jsonb_build_array(jsonb_build_object('school', 'University Two')),
        'email', 'alice@example.com',
        'experience', jsonb_build_array(jsonb_build_object('company', 'Beta', 'title', 'Senior Engineer')),
        'facebook_url', 'https://facebook.com/alice-updated',
        'github_url', 'https://github.com/alice-updated',
        'location_id', :'location2ID'::text,
        'linkedin_url', 'https://linkedin.com/in/alice-updated',
        'name', 'Alice Updated',
        'open_to_relocation', false,
        'open_to_remote', false,
        'phone', '+34 600 000 002',
        'photo_id', :'photo2ID'::text,
        'projects', jsonb_build_array(jsonb_build_object('name', 'Project Two')),
        'public', false,
        'skills', jsonb_build_array('go'),
        'summary', 'Updated summary',
        'twitter_url', 'https://x.com/alice-updated',
        'website_url', 'https://alice-updated.dev'
    ),
    'Should update existing profile on conflict'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
