-- Removes a member for the provided foundation.
create or replace function syncer_remove_member(p_foundation text, p_name text)
returns void as $$
    delete from member
    where foundation = p_foundation
    and name = p_name;
$$ language sql;
