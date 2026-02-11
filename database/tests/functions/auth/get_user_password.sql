-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set userID '00000000-0000-0000-0000-000000000101'
\set unknownUserID '00000000-0000-0000-0000-999999999999'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, password, user_id, username) values
    (decode('01', 'hex'), 'user@example.com', true, 'User', 'password-hash', :'userID', 'user');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return password hash for existing users
select is(
    auth_get_user_password(:'userID'::uuid),
    'password-hash',
    'Should return password hash for existing users'
);

-- Should return null for unknown users
select is(
    auth_get_user_password(:'unknownUserID'::uuid),
    null::text,
    'Should return null for unknown users'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
