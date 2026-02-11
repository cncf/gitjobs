-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set unknownUserID '00000000-0000-0000-0000-999999999999'
\set unverifiedUserID '00000000-0000-0000-0000-000000000102'
\set verifiedUserID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, user_id, username) values
    (decode('01', 'hex'), 'verified@example.com', true, 'Verified User', :'verifiedUserID', 'verified-user'),
    (decode('02', 'hex'), 'unverified@example.com', false, 'Unverified User', :'unverifiedUserID', 'unverified-user');

insert into job_seeker_profile (email, name, public, summary, user_id) values
    ('verified@example.com', 'Verified User', true, 'Profile summary', :'verifiedUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the full payload for verified users
select is(
    (
        select jsonb_build_object(
            'auth_hash', encode(auth_hash, 'hex'),
            'email', email,
            'email_verified', email_verified,
            'has_password', has_password,
            'has_profile', has_profile,
            'moderator', moderator,
            'name', name,
            'password', password,
            'user_id', user_id::text,
            'username', username
        )
        from auth_get_user_by_id_verified(:'verifiedUserID'::uuid)
    ),
    jsonb_build_object(
        'auth_hash', '01',
        'email', 'verified@example.com',
        'email_verified', true,
        'has_password', false,
        'has_profile', true,
        'moderator', false,
        'name', 'Verified User',
        'password', null,
        'user_id', :'verifiedUserID',
        'username', 'verified-user'
    ),
    'Should return the full payload for verified users'
);

-- Should return no row for unverified users
select is(
    (
        select count(*)
        from auth_get_user_by_id_verified(:'unverifiedUserID'::uuid)
    ),
    0::bigint,
    'Should return no row for unverified users'
);

-- Should return no row for unknown users
select is(
    (
        select count(*)
        from auth_get_user_by_id_verified(:'unknownUserID'::uuid)
    ),
    0::bigint,
    'Should return no row for unknown users'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
