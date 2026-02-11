-- Returns a verified user by username including password.
create or replace function auth_get_user_by_username(p_username text)
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
    password text
) as $$
    select
        u.user_id,
        u.auth_hash,
        u.email,
        u.email_verified,
        u.password is not null as has_password,
        p.job_seeker_profile_id is not null as has_profile,
        u.moderator,
        u.name,
        u.username,
        u.password
    from "user" u
    left join job_seeker_profile p on u.user_id = p.user_id
    where u.username = p_username
    and u.password is not null
    and u.email_verified = true;
$$ language sql;
