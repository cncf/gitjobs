-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for get_job_salary tests', :'employerID');

insert into job (
    description,
    employer_id,
    job_id,
    kind,
    salary,
    salary_currency,
    salary_max,
    salary_min,
    salary_period,
    status,
    title,
    workplace
) values (
    'Role for salary lookup tests',
    :'employerID',
    :'jobID',
    'full-time',
    120000,
    'USD',
    150000,
    100000,
    'year',
    'draft',
    'Platform Engineer',
    'remote'
);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return all salary fields for existing jobs
select is(
    (
        select jsonb_build_object(
            'salary', salary,
            'salary_currency', salary_currency,
            'salary_max', salary_max,
            'salary_min', salary_min,
            'salary_period', salary_period
        )
        from dashboard_employer_get_job_salary(:'jobID'::uuid)
    ),
    jsonb_build_object(
        'salary', 120000,
        'salary_currency', 'USD',
        'salary_max', 150000,
        'salary_min', 100000,
        'salary_period', 'year'
    ),
    'Should return all salary fields for existing jobs'
);

-- Should return no rows for unknown jobs
select is(
    (
        select count(*)
        from dashboard_employer_get_job_salary('99999999-9999-9999-9999-999999999999'::uuid)
    ),
    0::bigint,
    'Should return no rows for unknown jobs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
