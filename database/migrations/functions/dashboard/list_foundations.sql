-- Returns available foundations.
create or replace function list_foundations()
returns json as $$
    select coalesce(json_agg(json_build_object(
        'name', f.name
    ) order by f.name asc), '[]'::json)
    from foundation f;
$$ language sql;
