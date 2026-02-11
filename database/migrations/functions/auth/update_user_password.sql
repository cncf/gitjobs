-- Updates user password and rotates auth hash.
create or replace function auth_update_user_password(
    p_user_id uuid,
    p_new_password text
)
returns void as $$
    update "user" set
        auth_hash = gen_random_bytes(32),
        password = p_new_password
    where user_id = p_user_id;
$$ language sql;
