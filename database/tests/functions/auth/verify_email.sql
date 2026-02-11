-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(6);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set expiredCodeID '00000000-0000-0000-0000-000000000302'
\set expiredUserID '00000000-0000-0000-0000-000000000102'
\set validCodeID '00000000-0000-0000-0000-000000000301'
\set validUserID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, email_verified, name, user_id, username) values
    (decode('01', 'hex'), 'expired-user@example.com', false, 'Expired User', :'expiredUserID', 'expired-user'),
    (decode('02', 'hex'), 'valid-user@example.com', false, 'Valid User', :'validUserID', 'valid-user');

insert into email_verification_code (created_at, email_verification_code_id, user_id) values
    (current_timestamp - interval '2 days', :'expiredCodeID', :'expiredUserID'),
    (current_timestamp, :'validCodeID', :'validUserID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark user email as verified with a valid code
select verify_email(:'validCodeID'::uuid);

select ok(
    (
        select email_verified
        from "user"
        where user_id = :'validUserID'::uuid
    ),
    'Should mark user email as verified with a valid code'
);

-- Should delete the verification code after successful verification
select is(
    (
        select count(*)
        from email_verification_code
        where email_verification_code_id = :'validCodeID'::uuid
    ),
    0::bigint,
    'Should delete the verification code after successful verification'
);

-- Should fail for invalid codes
select throws_ok(
    $$ select verify_email('99999999-9999-9999-9999-999999999999'::uuid) $$,
    'invalid email verification code',
    'Should fail for invalid codes'
);

-- Should fail for expired codes
select throws_ok(
    $$ select verify_email('00000000-0000-0000-0000-000000000302'::uuid) $$,
    'invalid email verification code',
    'Should fail for expired codes'
);

-- Should keep expired-code user unverified
select ok(
    (
        select not email_verified
        from "user"
        where user_id = :'expiredUserID'::uuid
    ),
    'Should keep expired-code user unverified'
);

-- Should keep expired code row untouched
select is(
    (
        select count(*)
        from email_verification_code
        where email_verification_code_id = :'expiredCodeID'::uuid
    ),
    1::bigint,
    'Should keep expired code row untouched'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
