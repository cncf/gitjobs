-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set unknownUserID '00000000-0000-0000-0000-000000000999'
\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, user_id, username) values
    (decode('01', 'hex'), 'user@example.com', true, 'Old Name', :'userID', 'user');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should not update any row when the user does not exist
select update_user_details(
    :'unknownUserID'::uuid,
    jsonb_build_object('name', 'Unknown User')
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
        'name', 'Old Name',
        'username', 'user'
    ),
    'Should not update any row when the user does not exist'
);

-- Should update the user name from the JSON payload
select update_user_details(
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
