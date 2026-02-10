-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(5);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set application1ID '00000000-0000-0000-0000-000000000601'
\set application2ID '00000000-0000-0000-0000-000000000602'
\set application3ID '00000000-0000-0000-0000-000000000603'
\set employerID '00000000-0000-0000-0000-000000000101'
\set job1ID '00000000-0000-0000-0000-000000000301'
\set job2ID '00000000-0000-0000-0000-000000000302'
\set job3DeletedID '00000000-0000-0000-0000-000000000303'
\set jobSeeker1ID '00000000-0000-0000-0000-000000000501'
\set jobSeeker2ID '00000000-0000-0000-0000-000000000502'
\set jobSeeker3ID '00000000-0000-0000-0000-000000000503'
\set locationID '00000000-0000-0000-0000-000000000201'
\set profile1ID '00000000-0000-0000-0000-000000000401'
\set profile2ID '00000000-0000-0000-0000-000000000402'
\set profile3ID '00000000-0000-0000-0000-000000000403'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Location
insert into location (city, country, location_id, state)
values ('San Francisco', 'United States', :'locationID', 'CA');

-- Users
insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'jobSeeker1ID', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'jobSeeker2ID', 'bob'),
    (decode('03', 'hex'), 'carol@example.com', 'Carol', :'jobSeeker3ID', 'carol');

-- Profiles
insert into job_seeker_profile (
    job_seeker_profile_id,
    user_id,
    email,
    name,
    summary,

    experience
) values
    (
        :'profile1ID',
        :'jobSeeker1ID',
        'alice@example.com',
        'Alice',
        'Profile for Alice',

        '[
            {"title": "Staff Engineer", "company": "Acme", "end_date": null},
            {"title": "Engineer", "company": "OldCo", "end_date": "2022-01-01"}
        ]'::jsonb
    ),
    (
        :'profile2ID',
        :'jobSeeker2ID',
        'bob@example.com',
        'Bob',
        'Profile for Bob',

        '[{"title": "Backend Engineer", "company": "Beta", "end_date": "2023-10-01"}]'::jsonb
    ),
    (
        :'profile3ID',
        :'jobSeeker3ID',
        'carol@example.com',
        'Carol',
        'Profile for Carol',

        '[{"title": "DevRel", "company": "Gamma", "end_date": "2023-01-01"}]'::jsonb
    );

-- Employer and jobs
insert into employer (company, description, employer_id)
values ('Applications Employer', 'Employer for search_applications tests', :'employerID');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    location_id,
    title,
    workplace,
    description
) values
    (
        :'job1ID',
        :'employerID',
        'full-time',
        'published',
        :'locationID',
        'Platform Engineer',
        'remote',
        'Role one'
    ),
    (
        :'job2ID',
        :'employerID',
        'full-time',
        'published',
        null,
        'Data Engineer',
        'hybrid',
        'Role two'
    ),
    (
        :'job3DeletedID',
        :'employerID',
        'full-time',
        'deleted',
        null,
        'Deleted Role',
        'remote',
        'Deleted role'
    );

-- Applications
insert into application (
    application_id,
    created_at,
    job_id,
    job_seeker_profile_id
) values
    (:'application1ID', current_timestamp - interval '1 day', :'job1ID', :'profile1ID'),
    (:'application2ID', current_timestamp - interval '2 days', :'job2ID', :'profile2ID'),
    (:'application3ID', current_timestamp - interval '3 days', :'job3DeletedID', :'profile3ID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return only applications for non-deleted employer jobs
select is(
    (select total from search_applications(:'employerID'::uuid, '{}'::jsonb)),
    2::bigint,
    'Should return only applications for non-deleted employer jobs'
);

-- Should sort applications by applied_at descending
select is(
    (
        select (applications::jsonb->0->>'application_id')::uuid
        from search_applications(:'employerID'::uuid, '{}'::jsonb)
    ),
    :'application1ID'::uuid,
    'Should sort applications by applied_at descending'
);

-- Should extract the latest role as last_position
select is(
    (
        select applications::jsonb->0->>'last_position'
        from search_applications(:'employerID'::uuid, '{}'::jsonb)
    ),
    'Staff Engineer at Acme',
    'Should extract the latest role as last_position'
);

-- Should filter applications by job_id
select is(
    (
        select total
        from search_applications(
            :'employerID'::uuid,
            jsonb_build_object('job_id', :'job2ID'::text)
        )
    ),
    1::bigint,
    'Should filter applications by job_id'
);

-- Should respect limit and offset pagination
select is(
    (
        select (applications::jsonb->0->>'application_id')::uuid
        from search_applications(
            :'employerID'::uuid,
            jsonb_build_object('limit', 1, 'offset', 1)
        )
    ),
    :'application2ID'::uuid,
    'Should respect limit and offset pagination'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
