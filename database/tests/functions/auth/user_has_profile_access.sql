-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set applicantProfileID '00000000-0000-0000-0000-000000000401'
\set applicantUserID '00000000-0000-0000-0000-000000000301'
\set applicationID '00000000-0000-0000-0000-000000000501'
\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000201'
\set teamApprovedUserID '00000000-0000-0000-0000-000000000302'
\set teamPendingUserID '00000000-0000-0000-0000-000000000303'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'applicant@example.com', 'Applicant', :'applicantUserID', 'applicant'),
    (decode('02', 'hex'), 'approved@example.com', 'Approved', :'teamApprovedUserID', 'approved'),
    (decode('03', 'hex'), 'pending@example.com', 'Pending', :'teamPendingUserID', 'pending');

insert into job_seeker_profile (email, job_seeker_profile_id, name, summary, user_id) values
    ('applicant@example.com', :'applicantProfileID', 'Applicant', 'Summary', :'applicantUserID');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for profile access tests', :'employerID');

insert into job (description, employer_id, job_id, kind, status, title, workplace) values
    (
        'Role for profile access tests',
        :'employerID',
        :'jobID',
        'full-time',
        'published',
        'Platform Engineer',
        'remote'
    );

insert into application (application_id, job_id, job_seeker_profile_id) values
    (:'applicationID', :'jobID', :'applicantProfileID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'teamApprovedUserID'),
    (false, :'employerID', :'teamPendingUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should allow approved team members to access applied profiles
select is(
    user_has_profile_access(:'teamApprovedUserID'::uuid, :'applicantProfileID'::uuid),
    true,
    'Should allow approved team members to access applied profiles'
);

-- Should deny unapproved team members
select is(
    user_has_profile_access(:'teamPendingUserID'::uuid, :'applicantProfileID'::uuid),
    false,
    'Should deny unapproved team members'
);

-- Should deny unknown profiles
select is(
    user_has_profile_access(
        :'teamApprovedUserID'::uuid,
        '99999999-9999-9999-9999-999999999999'::uuid
    ),
    false,
    'Should deny unknown profiles'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
