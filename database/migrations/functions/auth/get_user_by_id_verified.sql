-- Returns a verified user by id.
create or replace function auth_get_user_by_id_verified(p_user_id uuid)
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
        null::text as password
    from "user" u
    left join job_seeker_profile p on u.user_id = p.user_id
    where u.user_id = p_user_id
    and u.email_verified = true;
$$ language sql;
