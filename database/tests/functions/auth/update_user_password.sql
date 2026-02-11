-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, password, user_id, username) values
    (decode('01', 'hex'), 'user@example.com', true, 'User', 'old-password', :'userID', 'user');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update the password hash
select auth_update_user_password(:'userID'::uuid, 'new-password');

select ok(
    (
        select jsonb_build_object(
            'email', email,
            'email_verified', email_verified,
            'name', name,
            'password', password,
            'username', username
        ) = jsonb_build_object(
            'email', 'user@example.com',
            'email_verified', true,
            'name', 'User',
            'password', 'new-password',
            'username', 'user'
        )
        from "user"
        where user_id = :'userID'::uuid
    ),
    'Should update the password hash'
);

-- Should rotate auth_hash when updating password
select ok(
    (
        select auth_hash <> decode('01', 'hex')
        from "user"
        where user_id = :'userID'::uuid
    ),
    'Should rotate auth_hash when updating password'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
