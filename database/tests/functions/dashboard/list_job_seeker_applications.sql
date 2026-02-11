-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set application1ID '00000000-0000-0000-0000-000000000601'
\set application2ID '00000000-0000-0000-0000-000000000602'
\set applicationOtherID '00000000-0000-0000-0000-000000000603'
\set employerID '00000000-0000-0000-0000-000000000101'
\set job1ID '00000000-0000-0000-0000-000000000301'
\set job2ID '00000000-0000-0000-0000-000000000302'
\set job3ID '00000000-0000-0000-0000-000000000303'
\set locationID '00000000-0000-0000-0000-000000000401'
\set profileID '00000000-0000-0000-0000-000000000501'
\set profileOtherID '00000000-0000-0000-0000-000000000502'
\set userID '00000000-0000-0000-0000-000000000201'
\set userOtherID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'userOtherID', 'bob');

insert into job_seeker_profile (email, job_seeker_profile_id, name, summary, user_id) values
    ('alice@example.com', :'profileID', 'Alice', 'Profile summary', :'userID'),
    ('bob@example.com', :'profileOtherID', 'Bob', 'Profile summary', :'userOtherID');

insert into location (city, country, location_id, state) values
    ('Valencia', 'Spain', :'locationID', null);

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for list_job_seeker_applications tests', :'employerID');

insert into job (description, employer_id, job_id, kind, location_id, status, title, workplace) values
    ('Role one', :'employerID', :'job1ID', 'full-time', :'locationID', 'published', 'Role One', 'remote'),
    ('Role two', :'employerID', :'job2ID', 'full-time', null, 'draft', 'Role Two', 'hybrid'),
    ('Role three', :'employerID', :'job3ID', 'full-time', null, 'published', 'Role Three', 'remote');

insert into application (application_id, created_at, job_id, job_seeker_profile_id) values
    (:'application1ID', '2026-01-02 10:00:00+00', :'job1ID', :'profileID'),
    (:'application2ID', '2026-01-01 10:00:00+00', :'job2ID', :'profileID'),
    (:'applicationOtherID', '2025-12-31 10:00:00+00', :'job3ID', :'profileOtherID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full applications payload sorted by applied_at
select is(
    dashboard_job_seeker_list_applications(:'userID'::uuid)::jsonb,
    jsonb_build_array(
        jsonb_build_object(
            'application_id',
            :'application1ID'::uuid,
            'applied_at',
            '2026-01-02 10:00:00+00'::timestamptz,
            'job_id',
            :'job1ID'::uuid,
            'job_location',
            jsonb_build_object(
                'city',
                'Valencia',
                'country',
                'Spain',
                'location_id',
                :'locationID'::uuid
            ),
            'job_status',
            'published',
            'job_title',
            'Role One',
            'job_workplace',
            'remote'
        ),
        jsonb_build_object(
            'application_id',
            :'application2ID'::uuid,
            'applied_at',
            '2026-01-01 10:00:00+00'::timestamptz,
            'job_id',
            :'job2ID'::uuid,
            'job_location',
            null,
            'job_status',
            'draft',
            'job_title',
            'Role Two',
            'job_workplace',
            'hybrid'
        )
    ),
    'Should return full applications payload sorted by applied_at'
);

-- Should return empty arrays for users without applications
select is(
    dashboard_job_seeker_list_applications(
        '99999999-9999-9999-9999-999999999999'::uuid
    )::jsonb,
    '[]'::jsonb,
    'Should return empty arrays for users without applications'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
