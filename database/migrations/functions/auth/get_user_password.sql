-- Returns the password hash for the provided user id.
create or replace function get_user_password(p_user_id uuid)
returns text as $$
    select u.password
    from "user" u
    where u.user_id = p_user_id;
$$ language sql;
