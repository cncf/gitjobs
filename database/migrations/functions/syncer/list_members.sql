-- Lists all members for the provided foundation.
create or replace function list_members(p_foundation text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'foundation', p_foundation,
        'level', m.level,
        'logo_url', m.logo_url,
        'name', m.name
    ) order by m.name asc), '[]'::json)
    from member m
    where m.foundation = p_foundation;
$$ language sql;
