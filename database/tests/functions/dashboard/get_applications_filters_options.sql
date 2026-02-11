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
\set newestJobID '00000000-0000-0000-0000-000000000302'
\set oldestJobID '00000000-0000-0000-0000-000000000301'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'locationID', 'CA');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for get_applications_filters_options tests', :'employerID');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    location_id,
    workplace,
    title,
    description,
    created_at
) values
    (
        :'oldestJobID',
        :'employerID',
        'full-time',
        'published',
        :'locationID',
        'hybrid',
        'Oldest Job',
        'Older job',
        '2026-01-01 10:00:00+00'
    ),
    (
        :'newestJobID',
        :'employerID',
        'full-time',
        'published',
        :'locationID',
        'remote',
        'Newest Job',
        'Newest job',
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
        'Deleted job',
        '2026-01-03 10:00:00+00'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full filters options payload excluding deleted jobs
select is(
    (
        select jsonb_build_object(
            'jobs',
            (
                select jsonb_agg(job - 'created_at')
                from jsonb_array_elements(
                    (get_applications_filters_options(:'employerID'::uuid)->'jobs')::jsonb
                ) as job
            )
        )
    ),
    jsonb_build_object(
        'jobs',
        jsonb_build_array(
            jsonb_build_object(
                'city', 'San Francisco',
                'country', 'United States',
                'job_id', :'newestJobID',
                'status', 'published',
                'title', 'Newest Job',
                'workplace', 'remote'
            ),
            jsonb_build_object(
                'city', 'San Francisco',
                'country', 'United States',
                'job_id', :'oldestJobID',
                'status', 'published',
                'title', 'Oldest Job',
                'workplace', 'hybrid'
            )
        )
    ),
    'Should return full filters options payload excluding deleted jobs'
);

-- Should return empty jobs for unknown employers
select is(
    get_applications_filters_options(
        '99999999-9999-9999-9999-999999999999'::uuid
    )::jsonb,
    '{"jobs":[]}'::jsonb,
    'Should return empty jobs for unknown employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
