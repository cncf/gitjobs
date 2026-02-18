-- Verifies email using a non-expired verification code.
create or replace function verify_email(p_code uuid)
returns void as $$
declare
    v_user_id uuid;
begin
    -- Delete the verification code if it's valid and return the associated user_id
    delete from email_verification_code
    where email_verification_code_id = p_code
    and created_at > current_timestamp - interval '1 day'
    returning user_id into v_user_id;

    -- If no user_id is returned, the code is invalid or expired
    if v_user_id is null then
        raise exception 'email verification failed: invalid or expired code';
    end if;

    -- Mark the user's email as verified
    update "user"
    set email_verified = true
    where user_id = v_user_id;
end
$$ language plpgsql;
