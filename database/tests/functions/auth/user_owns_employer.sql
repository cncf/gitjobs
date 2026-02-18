-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set approvedUserID '00000000-0000-0000-0000-000000000201'
\set pendingUserID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'approved@example.com', 'Approved', :'approvedUserID', 'approved'),
    (decode('02', 'hex'), 'pending@example.com', 'Pending', :'pendingUserID', 'pending');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for user_owns_employer tests', :'employerID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'approvedUserID'),
    (false, :'employerID', :'pendingUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return true for approved team members
select is(
    user_owns_employer(:'approvedUserID'::uuid, :'employerID'::uuid),
    true,
    'Should return true for approved team members'
);

-- Should return false for unapproved team members
select is(
    user_owns_employer(:'pendingUserID'::uuid, :'employerID'::uuid),
    false,
    'Should return false for unapproved team members'
);

-- Should return false for unknown employers
select is(
    user_owns_employer(
        :'approvedUserID'::uuid,
        '99999999-9999-9999-9999-999999999999'::uuid
    ),
    false,
    'Should return false for unknown employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
