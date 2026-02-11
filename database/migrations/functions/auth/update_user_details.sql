-- Updates user details from the provided payload.
create or replace function update_user_details(
    p_user_id uuid,
    p_user_summary jsonb
)
returns void as $$
    update "user" set
        name = p_user_summary->>'name'
    where user_id = p_user_id;
$$ language sql;
