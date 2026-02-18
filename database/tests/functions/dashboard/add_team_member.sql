-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set memberUserID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for add_team_member tests', :'employerID');

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'member@example.com', 'Member User', :'memberUserID', 'member-user');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the user id for an existing user email
select is(
    (
        select add_team_member(:'employerID'::uuid, 'member@example.com')
    ),
    :'memberUserID'::uuid,
    'Should return the user id for an existing user email'
);

-- Should insert a pending invitation for the matched user
select ok(
    exists (
        select 1
        from employer_team
        where employer_id = :'employerID'::uuid
        and user_id = :'memberUserID'::uuid
        and approved = false
    ),
    'Should insert a pending invitation for the matched user'
);

-- Should return null when the invitation already exists
select ok(
    (
        select add_team_member(:'employerID'::uuid, 'member@example.com') is null
    ),
    'Should return null when the invitation already exists'
);

-- Should return null for unknown emails
select ok(
    (
        select add_team_member(:'employerID'::uuid, 'unknown@example.com') is null
    ),
    'Should return null for unknown emails'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
