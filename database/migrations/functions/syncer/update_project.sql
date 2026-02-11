-- Updates a project for the provided foundation.
create or replace function syncer_update_project(
    p_foundation text,
    p_name text,
    p_maturity text,
    p_logo_url text
)
returns void as $$
    update project
    set
        maturity = p_maturity,
        logo_url = p_logo_url
    where foundation = p_foundation
    and name = p_name;
$$ language sql;
