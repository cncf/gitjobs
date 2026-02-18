-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set draftJobID '00000000-0000-0000-0000-000000000301'
\set employerID '00000000-0000-0000-0000-000000000101'
\set publishedJobID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for publish_job tests', :'employerID');

insert into job (
    description,
    employer_id,
    job_id,
    kind,
    salary_max_usd_year,
    salary_min_usd_year,
    salary_usd_year,
    status,
    title,
    workplace
) values
    (
        'Draft role',
        :'employerID',
        :'draftJobID',
        'full-time',
        null,
        null,
        null,
        'draft',
        'Draft Job',
        'remote'
    ),
    (
        'Published role',
        :'employerID',
        :'publishedJobID',
        'full-time',
        60000,
        50000,
        55000,
        'published',
        'Published Job',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should publish draft jobs and update normalized salary fields
select publish_job(:'draftJobID'::uuid, 120000, 100000, 150000);

select ok(
    exists (
        select 1
        from job
        where job_id = :'draftJobID'::uuid
        and status = 'pending-approval'
        and archived_at is null
        and salary_usd_year = 120000
        and salary_min_usd_year = 100000
        and salary_max_usd_year = 150000
        and updated_at is not null
    ),
    'Should publish draft jobs and update normalized salary fields'
);

-- Should not modify jobs outside publishable statuses
select publish_job(:'publishedJobID'::uuid, 130000, 110000, 160000);

select is(
    (
        select jsonb_build_object(
            'salary_max_usd_year', salary_max_usd_year,
            'salary_min_usd_year', salary_min_usd_year,
            'salary_usd_year', salary_usd_year,
            'status', status
        )
        from job
        where job_id = :'publishedJobID'::uuid
    ),
    jsonb_build_object(
        'salary_max_usd_year', 60000,
        'salary_min_usd_year', 50000,
        'salary_usd_year', 55000,
        'status', 'published'
    ),
    'Should not modify jobs outside publishable statuses'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
