-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000301'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for delete_job tests', :'employerID');

insert into job (job_id, employer_id, kind, status, title, workplace, description) values
    (
        :'jobID',
        :'employerID',
        'full-time',
        'published',
        'Role to Delete',
        'remote',
        'Description'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should soft-delete a job and set deleted_at
select delete_job(:'jobID'::uuid);

select ok(
    exists (
        select 1
        from job
        where job_id = :'jobID'::uuid
        and status = 'deleted'
        and deleted_at is not null
    ),
    'Should soft-delete a job and set deleted_at'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
