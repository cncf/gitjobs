-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set locationID '00000000-0000-0000-0000-000000000101'
\set userID '00000000-0000-0000-0000-000000000201'
\set unknownUserID '00000000-0000-0000-0000-999999999999'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into location (city, country, location_id, state) values
    ('Madrid', 'Spain', :'locationID', null);

insert into job_seeker_profile (
    email,
    location_id,
    name,
    public,
    summary,
    user_id
) values (
    'alice@example.com',
    :'locationID',
    'Alice',
    true,
    'Profile summary',
    :'userID'
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return profile data with nested location
select is(
    dashboard_job_seeker_get_profile(:'userID'::uuid)::jsonb,
    '{
        "email": "alice@example.com",
        "location": {
            "city": "Madrid",
            "country": "Spain",
            "location_id": "00000000-0000-0000-0000-000000000101"
        },
        "name": "Alice",
        "public": true,
        "summary": "Profile summary"
    }'::jsonb,
    'Should return profile data with nested location'
);

-- Should return null when profile does not exist
select is(
    dashboard_job_seeker_get_profile(:'unknownUserID'::uuid)::jsonb,
    null::jsonb,
    'Should return null when profile does not exist'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
