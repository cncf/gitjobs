-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set verifiedUserID '00000000-0000-0000-0000-000000000101'
\set unverifiedUserID '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, password, user_id, username) values
    (decode('01', 'hex'), 'verified@example.com', true, 'Verified User', 'hash1', :'verifiedUserID', 'verified-user'),
    (decode('02', 'hex'), 'unverified@example.com', false, 'Unverified User', 'hash2', :'unverifiedUserID', 'unverified-user');

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
        from auth_get_user_by_email('verified@example.com')
    ),
    jsonb_build_object(
        'auth_hash', '01',
        'email', 'verified@example.com',
        'email_verified', true,
        'has_password', true,
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
        from auth_get_user_by_email('unverified@example.com')
    ),
    0::bigint,
    'Should return no row for unverified users'
);

-- Should return no row for unknown emails
select is(
    (
        select count(*)
        from auth_get_user_by_email('missing@example.com')
    ),
    0::bigint,
    'Should return no row for unknown emails'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
