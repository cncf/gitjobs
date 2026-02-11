-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set profileID '00000000-0000-0000-0000-000000000201'
\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into job_seeker_profile (email, job_seeker_profile_id, name, summary, user_id) values
    ('alice@example.com', :'profileID', 'Alice', 'Summary', :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the profile owner user id
select is(
    dashboard_employer_get_job_seeker_user_id(:'profileID'::uuid),
    :'userID'::uuid,
    'Should return the profile owner user id'
);

-- Should return null for unknown profiles
select is(
    dashboard_employer_get_job_seeker_user_id('99999999-9999-9999-9999-999999999999'::uuid),
    null::uuid,
    'Should return null for unknown profiles'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
