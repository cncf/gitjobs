-- Removes a project for the provided foundation.
create or replace function remove_project(p_foundation text, p_name text)
returns void as $$
    delete from project
    where foundation = p_foundation
    and name = p_name;
$$ language sql;
