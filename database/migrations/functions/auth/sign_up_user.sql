-- Creates a user and generates an email verification code if needed.
create or replace function sign_up_user(
    p_user_summary jsonb,
    p_email_verified boolean
)
returns table(
    user_id uuid,
    auth_hash bytea,
    email text,
    email_verified boolean,
    has_password boolean,
    has_profile boolean,
    moderator boolean,
    name text,
    username text,
    verification_code uuid
) as $$
declare
    v_verification_code uuid;
begin
    -- Insert the new user and fill the return payload
    insert into "user" (
        auth_hash,
        email,
        email_verified,
        name,
        password,
        username
    ) values (
        gen_random_bytes(32),
        p_user_summary->>'email',
        p_email_verified,
        p_user_summary->>'name',
        p_user_summary->>'password',
        p_user_summary->>'username'
    )
    returning
        "user".user_id,
        "user".auth_hash,
        "user".email,
        "user".email_verified,
        "user".password is not null,
        "user".moderator,
        "user".name,
        "user".username
    into
        user_id,
        auth_hash,
        email,
        email_verified,
        has_password,
        moderator,
        name,
        username;

    -- New users do not have profiles yet
    has_profile := false;

    -- If the email is not verified, generate an email verification code
    if not p_email_verified then
        insert into email_verification_code (user_id)
        values (user_id)
        returning email_verification_code_id into v_verification_code;
    end if;

    -- Return the verification code when generated
    verification_code := v_verification_code;

    return next;
end
$$ language plpgsql;
