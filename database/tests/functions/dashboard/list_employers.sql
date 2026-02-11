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
\set employerPendingID '00000000-0000-0000-0000-000000000103'
\set userID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'owner@example.com', 'Owner', :'userID', 'owner');

insert into employer (company, description, employer_id) values
    ('Beta Corp', 'Employer B', :'employer2ID'),
    ('Acme Corp', 'Employer A', :'employer1ID'),
    ('Pending Corp', 'Employer Pending', :'employerPendingID');

insert into employer_team (approved, employer_id, user_id) values
    (true, :'employer1ID', :'userID'),
    (true, :'employer2ID', :'userID'),
    (false, :'employerPendingID', :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload with approved employers sorted by company
select is(
    list_employers(:'userID'::uuid)::jsonb,
    '[
        {
            "company": "Acme Corp",
            "employer_id": "00000000-0000-0000-0000-000000000101",
            "logo_id": null
        },
        {
            "company": "Beta Corp",
            "employer_id": "00000000-0000-0000-0000-000000000102",
            "logo_id": null
        }
    ]'::jsonb,
    'Should return full payload with approved employers sorted by company'
);

-- Should return empty arrays when the user has no approved employers
select is(
    list_employers('99999999-9999-9999-9999-999999999999'::uuid)::jsonb,
    '[]'::jsonb,
    'Should return empty arrays when the user has no approved employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
