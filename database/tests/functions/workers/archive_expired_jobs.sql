-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set expiredJobID '00000000-0000-0000-0000-000000000301'
\set freshJobID '00000000-0000-0000-0000-000000000302'
\set draftJobID '00000000-0000-0000-0000-000000000303'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for archive_expired_jobs tests', :'employerID');

insert into job (
    description,
    employer_id,
    job_id,
    kind,
    published_at,
    status,
    title,
    workplace
) values
    (
        'Expired published role',
        :'employerID',
        :'expiredJobID',
        'full-time',
        current_timestamp - interval '40 days',
        'published',
        'Expired Job',
        'remote'
    ),
    (
        'Fresh published role',
        :'employerID',
        :'freshJobID',
        'full-time',
        current_timestamp - interval '10 days',
        'published',
        'Fresh Job',
        'remote'
    ),
    (
        'Old draft role',
        :'employerID',
        :'draftJobID',
        'full-time',
        current_timestamp - interval '40 days',
        'draft',
        'Draft Job',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should archive published jobs older than 30 days
select archive_expired_jobs();

select ok(
    (
        select status = 'archived'
        and archived_at is not null
        and updated_at is not null
        from job
        where job_id = :'expiredJobID'::uuid
    ),
    'Should archive published jobs older than 30 days'
);

-- Should keep fresh published and draft jobs unchanged
select ok(
    (
        select status = 'published'
        and archived_at is null
        and updated_at is null
        from job
        where job_id = :'freshJobID'::uuid
    )
    and (
        select status = 'draft'
        and archived_at is null
        and updated_at is null
        from job
        where job_id = :'draftJobID'::uuid
    ),
    'Should keep fresh published and draft jobs unchanged'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
