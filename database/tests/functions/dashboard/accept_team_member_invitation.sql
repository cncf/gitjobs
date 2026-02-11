-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set userID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for accept_team_member_invitation tests', :'employerID');

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'member@example.com', 'Member User', :'userID', 'member-user');

insert into employer_team (approved, employer_id, user_id) values
    (false, :'employerID', :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark the invitation as approved
select accept_team_member_invitation(:'employerID'::uuid, :'userID'::uuid);

select is(
    (
        select approved
        from employer_team
        where employer_id = :'employerID'::uuid
        and user_id = :'userID'::uuid
    ),
    true,
    'Should mark the invitation as approved'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
