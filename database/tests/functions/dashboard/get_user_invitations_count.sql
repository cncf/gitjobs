-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employer1ID '00000000-0000-0000-0000-000000000101'
\set employer2ID '00000000-0000-0000-0000-000000000102'
\set employer3ID '00000000-0000-0000-0000-000000000103'
\set user1ID '00000000-0000-0000-0000-000000000201'
\set user2ID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'user1ID', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'user2ID', 'bob');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer 1', :'employer1ID'),
    ('Beta Corp', 'Employer 2', :'employer2ID'),
    ('Gamma Corp', 'Employer 3', :'employer3ID');

insert into employer_team (approved, employer_id, user_id) values
    (false, :'employer1ID', :'user1ID'),
    (false, :'employer2ID', :'user1ID'),
    (true, :'employer3ID', :'user1ID'),
    (false, :'employer1ID', :'user2ID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should count only pending invitations for the selected user
select is(
    dashboard_employer_get_user_invitations_count(:'user1ID'::uuid),
    2::bigint,
    'Should count only pending invitations for the selected user'
);

-- Should return pending invitation count for other users independently
select is(
    dashboard_employer_get_user_invitations_count(:'user2ID'::uuid),
    1::bigint,
    'Should return pending invitation count for other users independently'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
