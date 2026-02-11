-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set approvedUser1ID '00000000-0000-0000-0000-000000000201'
\set approvedUser2ID '00000000-0000-0000-0000-000000000202'
\set employerID '00000000-0000-0000-0000-000000000101'
\set pendingUserID '00000000-0000-0000-0000-000000000203'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'approved1@example.com', 'Approved 1', :'approvedUser1ID', 'approved1'),
    (decode('02', 'hex'), 'approved2@example.com', 'Approved 2', :'approvedUser2ID', 'approved2'),
    (decode('03', 'hex'), 'pending@example.com', 'Pending', :'pendingUserID', 'pending');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for delete_team_member tests', :'employerID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'approvedUser1ID'),
    (true, :'employerID', :'approvedUser2ID'),
    (false, :'employerID', :'pendingUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should delete pending invitations directly
select lives_ok(
    $$ select dashboard_employer_delete_team_member(
        '00000000-0000-0000-0000-000000000101'::uuid,
        '00000000-0000-0000-0000-000000000203'::uuid
    ) $$,
    'Should delete pending invitations directly'
);

-- Should delete approved members when another approved member remains
select lives_ok(
    $$ select dashboard_employer_delete_team_member(
        '00000000-0000-0000-0000-000000000101'::uuid,
        '00000000-0000-0000-0000-000000000201'::uuid
    ) $$,
    'Should delete approved members when another approved member remains'
);

-- Should prevent deleting the last approved team member
select throws_ok(
    $$ select dashboard_employer_delete_team_member(
        '00000000-0000-0000-0000-000000000101'::uuid,
        '00000000-0000-0000-0000-000000000202'::uuid
    ) $$,
    'cannot delete last approved team member',
    'Should prevent deleting the last approved team member'
);

-- Should fail for unknown team memberships
select throws_ok(
    $$ select dashboard_employer_delete_team_member(
        '00000000-0000-0000-0000-000000000101'::uuid,
        '99999999-9999-9999-9999-999999999999'::uuid
    ) $$,
    'team member not found',
    'Should fail for unknown team memberships'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
