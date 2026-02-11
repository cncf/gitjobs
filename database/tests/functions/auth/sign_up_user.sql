-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(8);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload for unverified users
with sign_up_output as (
    select *
    from auth_sign_up_user(
        jsonb_build_object(
            'email', 'unverified@example.com',
            'name', 'Unverified User',
            'password', 'password-hash',
            'username', 'unverified-user'
        ),
        false
    )
)
select ok(
    (
        select jsonb_build_object(
            'auth_hash_length', length(encode(auth_hash, 'hex')),
            'email', email,
            'email_verified', email_verified,
            'has_password', has_password,
            'has_profile', has_profile,
            'moderator', moderator,
            'name', name,
            'username', username,
            'verification_code_present', verification_code is not null
        ) = jsonb_build_object(
            'auth_hash_length', 64,
            'email', 'unverified@example.com',
            'email_verified', false,
            'has_password', true,
            'has_profile', false,
            'moderator', false,
            'name', 'Unverified User',
            'username', 'unverified-user',
            'verification_code_present', true
        )
        from sign_up_output
    ),
    'Should return full payload for unverified users'
);

-- Should insert all expected fields for unverified users
select ok(
    exists (
        select 1
        from "user"
        where email = 'unverified@example.com'
        and email_verified = false
        and name = 'Unverified User'
        and password = 'password-hash'
        and username = 'unverified-user'
        and length(encode(auth_hash, 'hex')) = 64
    ),
    'Should insert all expected fields for unverified users'
);

-- Should create an email verification row for unverified users
select ok(
    exists (
        select 1
        from email_verification_code evc
        join "user" u using (user_id)
        where u.email = 'unverified@example.com'
    ),
    'Should create an email verification row for unverified users'
);

-- Should return full payload for verified users
with sign_up_output as (
    select *
    from auth_sign_up_user(
        jsonb_build_object(
            'email', 'verified@example.com',
            'name', 'Verified User',
            'password', 'password-hash',
            'username', 'verified-user'
        ),
        true
    )
)
select ok(
    (
        select jsonb_build_object(
            'auth_hash_length', length(encode(auth_hash, 'hex')),
            'email', email,
            'email_verified', email_verified,
            'has_password', has_password,
            'has_profile', has_profile,
            'moderator', moderator,
            'name', name,
            'username', username,
            'verification_code_present', verification_code is not null
        ) = jsonb_build_object(
            'auth_hash_length', 64,
            'email', 'verified@example.com',
            'email_verified', true,
            'has_password', true,
            'has_profile', false,
            'moderator', false,
            'name', 'Verified User',
            'username', 'verified-user',
            'verification_code_present', false
        )
        from sign_up_output
    ),
    'Should return full payload for verified users'
);

-- Should insert all expected fields for verified users
select ok(
    exists (
        select 1
        from "user"
        where email = 'verified@example.com'
        and email_verified = true
        and name = 'Verified User'
        and password = 'password-hash'
        and username = 'verified-user'
        and length(encode(auth_hash, 'hex')) = 64
    ),
    'Should insert all expected fields for verified users'
);

-- Should not create email verification rows for verified users
select is(
    (
        select count(*)
        from email_verification_code evc
        join "user" u using (user_id)
        where u.email = 'verified@example.com'
    ),
    0::bigint,
    'Should not create email verification rows for verified users'
);

-- Should insert both users
select is(
    (
        select count(*)
        from "user"
        where email in ('unverified@example.com', 'verified@example.com')
    ),
    2::bigint,
    'Should insert both users'
);

-- Should persist distinct auth_hash values per new user
select ok(
    (
        select count(distinct auth_hash)
        from "user"
        where email in ('unverified@example.com', 'verified@example.com')
    ) = 2,
    'Should persist distinct auth_hash values per new user'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
