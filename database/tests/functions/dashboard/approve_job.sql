-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

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
    ('Acme', 'Employer for approve_job tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    ('Pending approval role', :'employerID', :'jobID', 'full-time', 'pending-approval', 'Platform Engineer', 'remote');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return null old first_published_at on the first approval
select is(
    dashboard_moderator_approve_job(:'jobID'::uuid, :'reviewerID'::uuid),
    null::timestamptz,
    'Should return null old first_published_at on the first approval'
);

-- Should mark job as published and store reviewer metadata
select ok(
    exists (
        select 1
        from job
        where job_id = :'jobID'::uuid
        and status = 'published'
        and reviewed_by = :'reviewerID'::uuid
        and reviewed_at is not null
        and first_published_at is not null
        and published_at is not null
    ),
    'Should mark job as published and store reviewer metadata'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
