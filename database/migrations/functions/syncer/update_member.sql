-- Updates a member for the provided foundation.
create or replace function syncer_update_member(
    p_foundation text,
    p_name text,
    p_level text,
    p_logo_url text
)
returns void as $$
    update member
    set
        level = p_level,
        logo_url = p_logo_url
    where foundation = p_foundation
    and name = p_name;
$$ language sql;
