-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, user_id, username) values
    (decode('01', 'hex'), 'user@example.com', true, 'Old Name', :'userID', 'user');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update the user name from the JSON payload
select auth_update_user_details(
    :'userID'::uuid,
    jsonb_build_object('name', 'New Name')
);

select is(
    (
        select jsonb_build_object(
            'email', email,
            'email_verified', email_verified,
            'name', name,
            'username', username
        )
        from "user"
        where user_id = :'userID'::uuid
    ),
    jsonb_build_object(
        'email', 'user@example.com',
        'email_verified', true,
        'name', 'New Name',
        'username', 'user'
    ),
    'Should update the user name from the JSON payload'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
