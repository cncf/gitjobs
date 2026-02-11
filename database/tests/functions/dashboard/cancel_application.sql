-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set applicationID '00000000-0000-0000-0000-000000000601'
\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000301'
\set profileID '00000000-0000-0000-0000-000000000401'
\set userOwnerID '00000000-0000-0000-0000-000000000201'
\set userOtherID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'owner@example.com', 'Owner', :'userOwnerID', 'owner'),
    (decode('02', 'hex'), 'other@example.com', 'Other', :'userOtherID', 'other');

insert into job_seeker_profile (email, job_seeker_profile_id, name, summary, user_id) values
    ('owner@example.com', :'profileID', 'Owner', 'Profile summary', :'userOwnerID');

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for cancel_application tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    ('Role', :'employerID', :'jobID', 'full-time', 'published', 'Platform Engineer', 'remote');

insert into application (application_id, job_id, job_seeker_profile_id) values
    (:'applicationID', :'jobID', :'profileID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should not delete application when user is not the profile owner
select cancel_application(:'applicationID'::uuid, :'userOtherID'::uuid);

select is(
    (select count(*) from application where application_id = :'applicationID'::uuid),
    1::bigint,
    'Should not delete application when user is not the profile owner'
);

-- Should delete application when user owns the profile
select cancel_application(:'applicationID'::uuid, :'userOwnerID'::uuid);

select is(
    (select count(*) from application where application_id = :'applicationID'::uuid),
    0::bigint,
    'Should delete application when user owns the profile'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
