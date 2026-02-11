-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set approvedUserID '00000000-0000-0000-0000-000000000201'
\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000301'
\set pendingUserID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'approved@example.com', 'Approved', :'approvedUserID', 'approved'),
    (decode('02', 'hex'), 'pending@example.com', 'Pending', :'pendingUserID', 'pending');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for user_owns_job tests', :'employerID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'approvedUserID'),
    (false, :'employerID', :'pendingUserID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    (
        'Role for ownership tests',
        :'employerID',
        :'jobID',
        'full-time',
        'published',
        'Platform Engineer',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return true for approved owners of the job employer
select is(
    auth_user_owns_job(:'approvedUserID'::uuid, :'jobID'::uuid),
    true,
    'Should return true for approved owners of the job employer'
);

-- Should return false for unapproved team members
select is(
    auth_user_owns_job(:'pendingUserID'::uuid, :'jobID'::uuid),
    false,
    'Should return false for unapproved team members'
);

-- Should return false for unknown jobs
select is(
    auth_user_owns_job(:'approvedUserID'::uuid, '99999999-9999-9999-9999-999999999999'::uuid),
    false,
    'Should return false for unknown jobs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
