-- Adds a member for the provided foundation.
create or replace function syncer_add_member(
    p_foundation text,
    p_name text,
    p_level text,
    p_logo_url text
)
returns void as $$
    insert into member (foundation, name, level, logo_url)
    values (p_foundation, p_name, p_level, p_logo_url);
$$ language sql;
