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
\set reviewerID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, moderator, name, user_id, username) values
    (decode('01', 'hex'), 'reviewer@example.com', true, 'Reviewer', :'reviewerID', 'reviewer');

insert into employer (company, description, employer_id) values
    ('Acme', 'Employer for reject_job tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    ('Pending approval role', :'employerID', :'jobID', 'full-time', 'pending-approval', 'Platform Engineer', 'remote');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark job as rejected and store notes and reviewer metadata
select reject_job(
    :'jobID'::uuid,
    :'reviewerID'::uuid,
    'Needs more detail'
);

select ok(
    exists (
        select 1
        from job
        where job_id = :'jobID'::uuid
        and status = 'rejected'
        and review_notes = 'Needs more detail'
        and reviewed_at is not null
        and reviewed_by = :'reviewerID'::uuid
    ),
    'Should mark job as rejected and store notes and reviewer metadata'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
