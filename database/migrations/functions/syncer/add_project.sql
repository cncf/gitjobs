-- Adds a project for the provided foundation.
create or replace function syncer_add_project(
    p_foundation text,
    p_name text,
    p_maturity text,
    p_logo_url text
)
returns void as $$
    insert into project (foundation, name, maturity, logo_url)
    values (p_foundation, p_name, p_maturity, p_logo_url);
$$ language sql;
