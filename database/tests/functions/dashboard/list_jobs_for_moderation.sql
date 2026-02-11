-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set job1ID '00000000-0000-0000-0000-000000000301'
\set job2ID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for list_jobs_for_moderation tests', :'employerID');

insert into job (
    created_at,
    description,
    employer_id,
    job_id,
    kind,
    status,
    title,
    workplace
) values
    (
        '2026-01-01 10:00:00+00',
        'Pending role',
        :'employerID',
        :'job1ID',
        'full-time',
        'pending-approval',
        'Pending Engineer',
        'remote'
    ),
    (
        '2026-01-02 10:00:00+00',
        'Published role',
        :'employerID',
        :'job2ID',
        'full-time',
        'published',
        'Published Engineer',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full moderation payload for the requested status
select is(
    list_jobs_for_moderation('pending-approval')::jsonb,
    jsonb_build_array(
        jsonb_build_object(
            'created_at',
            '2026-01-01 10:00:00+00'::timestamptz,
            'employer',
            jsonb_build_object(
                'company',
                'Acme',
                'employer_id',
                :'employerID'::uuid
            ),
            'job_id',
            :'job1ID'::uuid,
            'title',
            'Pending Engineer'
        )
    ),
    'Should return full moderation payload for the requested status'
);

-- Should return empty arrays when no jobs match the status
select is(
    list_jobs_for_moderation('deleted')::jsonb,
    '[]'::jsonb,
    'Should return empty arrays when no jobs match the status'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
