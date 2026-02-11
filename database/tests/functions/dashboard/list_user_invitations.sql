-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set approvedEmployerID '00000000-0000-0000-0000-000000000103'
\set newestEmployerID '00000000-0000-0000-0000-000000000102'
\set olderEmployerID '00000000-0000-0000-0000-000000000101'
\set userID '00000000-0000-0000-0000-000000000201'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'member@example.com', 'Member', :'userID', 'member');

insert into employer (company, created_at, description, employer_id) values
    (
        'Older Corp',
        '2026-01-01 10:00:00+00',
        'Older employer',
        :'olderEmployerID'
    ),
    (
        'Newest Corp',
        '2026-01-02 10:00:00+00',
        'Newest employer',
        :'newestEmployerID'
    ),
    (
        'Approved Corp',
        '2026-01-03 10:00:00+00',
        'Approved employer',
        :'approvedEmployerID'
    );

insert into employer_team (approved, employer_id, user_id) values
    (false, :'olderEmployerID', :'userID'),
    (false, :'newestEmployerID', :'userID'),
    (true, :'approvedEmployerID', :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full pending invitations payload sorted by created_at
select is(
    list_user_invitations(:'userID'::uuid)::jsonb,
    jsonb_build_array(
        jsonb_build_object(
            'company',
            'Newest Corp',
            'created_at',
            '2026-01-02 10:00:00+00'::timestamptz,
            'employer_id',
            :'newestEmployerID'::uuid
        ),
        jsonb_build_object(
            'company',
            'Older Corp',
            'created_at',
            '2026-01-01 10:00:00+00'::timestamptz,
            'employer_id',
            :'olderEmployerID'::uuid
        )
    ),
    'Should return full pending invitations payload sorted by created_at'
);

-- Should return empty arrays when the user has no pending invitations
select is(
    list_user_invitations(
        '99999999-9999-9999-9999-999999999999'::uuid
    )::jsonb,
    '[]'::jsonb,
    'Should return empty arrays when the user has no pending invitations'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
