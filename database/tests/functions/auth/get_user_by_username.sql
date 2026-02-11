-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set unverifiedWithPasswordID '00000000-0000-0000-0000-000000000103'
\set withPasswordID '00000000-0000-0000-0000-000000000101'
\set withoutPasswordID '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, password, user_id, username) values
    (decode('01', 'hex'), 'with-password@example.com', true, 'With Password', 'hash1', :'withPasswordID', 'with-password'),
    (decode('02', 'hex'), 'without-password@example.com', true, 'Without Password', null, :'withoutPasswordID', 'without-password'),
    (decode('03', 'hex'), 'unverified@example.com', false, 'Unverified Password', 'hash3', :'unverifiedWithPasswordID', 'unverified-password');

insert into job_seeker_profile (email, name, public, summary, user_id) values
    ('with-password@example.com', 'With Password', true, 'Profile summary', :'withPasswordID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return the full payload for verified users with password
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
        from auth_get_user_by_username('with-password')
    ),
    jsonb_build_object(
        'auth_hash', '01',
        'email', 'with-password@example.com',
        'email_verified', true,
        'has_password', true,
        'has_profile', true,
        'moderator', false,
        'name', 'With Password',
        'password', 'hash1',
        'user_id', :'withPasswordID',
        'username', 'with-password'
    ),
    'Should return the full payload for verified users with password'
);

-- Should return no row when the user has no password
select is(
    (
        select count(*)
        from auth_get_user_by_username('without-password')
    ),
    0::bigint,
    'Should return no row when the user has no password'
);

-- Should return no row when the user is not verified
select is(
    (
        select count(*)
        from auth_get_user_by_username('unverified-password')
    ),
    0::bigint,
    'Should return no row when the user is not verified'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
