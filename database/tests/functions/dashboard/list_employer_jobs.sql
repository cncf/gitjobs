-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set deletedJobID '00000000-0000-0000-0000-000000000303'
\set employerID '00000000-0000-0000-0000-000000000101'
\set locationID '00000000-0000-0000-0000-000000000201'
\set newestJobID '00000000-0000-0000-0000-000000000301'
\set olderJobID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'locationID', 'CA');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for list_employer_jobs tests', :'employerID');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    location_id,
    workplace,
    title,
    description,
    created_at,
    published_at
) values
    (
        :'newestJobID',
        :'employerID',
        'full-time',
        'published',
        :'locationID',
        'remote',
        'Newest Job',
        'Newest role',
        '2026-01-01 10:00:00+00',
        '2026-01-03 10:00:00+00'
    ),
    (
        :'olderJobID',
        :'employerID',
        'full-time',
        'published',
        :'locationID',
        'hybrid',
        'Older Job',
        'Older role',
        '2025-12-31 10:00:00+00',
        '2026-01-02 10:00:00+00'
    ),
    (
        :'deletedJobID',
        :'employerID',
        'full-time',
        'deleted',
        :'locationID',
        'remote',
        'Deleted Job',
        'Deleted role',
        '2026-01-02 10:00:00+00',
        '2026-01-03 10:00:00+00'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full jobs payload excluding deleted rows
select is(
    dashboard_employer_list_employer_jobs(:'employerID'::uuid)::jsonb,
    jsonb_build_array(
        jsonb_build_object(
            'archived_at',
            null,
            'city',
            'San Francisco',
            'country',
            'United States',
            'created_at',
            '2026-01-01 10:00:00+00'::timestamptz,
            'job_id',
            :'newestJobID'::uuid,
            'published_at',
            '2026-01-03 10:00:00+00'::timestamptz,
            'review_notes',
            null,
            'status',
            'published',
            'title',
            'Newest Job',
            'workplace',
            'remote'
        ),
        jsonb_build_object(
            'archived_at',
            null,
            'city',
            'San Francisco',
            'country',
            'United States',
            'created_at',
            '2025-12-31 10:00:00+00'::timestamptz,
            'job_id',
            :'olderJobID'::uuid,
            'published_at',
            '2026-01-02 10:00:00+00'::timestamptz,
            'review_notes',
            null,
            'status',
            'published',
            'title',
            'Older Job',
            'workplace',
            'hybrid'
        )
    ),
    'Should return full jobs payload excluding deleted rows'
);

-- Should return empty arrays for unknown employers
select is(
    dashboard_employer_list_employer_jobs('99999999-9999-9999-9999-999999999999'::uuid)::jsonb,
    '[]'::jsonb,
    'Should return empty arrays for unknown employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
