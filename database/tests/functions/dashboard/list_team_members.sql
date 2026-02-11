-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set userAliceID '00000000-0000-0000-0000-000000000201'
\set userBobID '00000000-0000-0000-0000-000000000202'
\set userCarolID '00000000-0000-0000-0000-000000000203'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for list_team_members tests', :'employerID');

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userAliceID', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'userBobID', 'bob'),
    (decode('03', 'hex'), 'carol@example.com', 'Carol', :'userCarolID', 'carol');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employerID', :'userBobID'),
    (true, :'employerID', :'userAliceID'),
    (false, :'employerID', :'userCarolID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full team members payload sorted by name
select is(
    dashboard_employer_list_team_members(:'employerID'::uuid)::jsonb,
    '[
        {
            "approved": true,
            "email": "alice@example.com",
            "name": "Alice",
            "user_id": "00000000-0000-0000-0000-000000000201",
            "username": "alice"
        },
        {
            "approved": true,
            "email": "bob@example.com",
            "name": "Bob",
            "user_id": "00000000-0000-0000-0000-000000000202",
            "username": "bob"
        },
        {
            "approved": false,
            "email": "carol@example.com",
            "name": "Carol",
            "user_id": "00000000-0000-0000-0000-000000000203",
            "username": "carol"
        }
    ]'::jsonb,
    'Should return full team members payload sorted by name'
);

-- Should return empty arrays for unknown employers
select is(
    dashboard_employer_list_team_members('99999999-9999-9999-9999-999999999999'::uuid)::jsonb,
    '[]'::jsonb,
    'Should return empty arrays for unknown employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
