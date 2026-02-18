-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set deletedJobID '00000000-0000-0000-0000-000000000302'
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
    ),
    (
        :'deletedJobID',
        :'employerID',
        'full-time',
        'deleted',
        'Already Deleted Role',
        'remote',
        'Already deleted description'
    );

update job
set deleted_at = '2024-01-01 00:00:00+00'
where job_id = :'deletedJobID'::uuid;

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

-- Should not update already-deleted jobs
select delete_job(:'deletedJobID'::uuid);

select is(
    (
        select jsonb_build_object(
            'deleted_at', deleted_at,
            'status', status
        )
        from job
        where job_id = :'deletedJobID'::uuid
    ),
    jsonb_build_object(
        'deleted_at', '2024-01-01 00:00:00+00'::timestamptz,
        'status', 'deleted'
    ),
    'Should not update already-deleted jobs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
