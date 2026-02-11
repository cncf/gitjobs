-- Lists all foundations with a configured landscape URL.
create or replace function syncer_list_foundations()
returns json as $$
    select coalesce(json_agg(json_build_object(
        'landscape_url', f.landscape_url,
        'name', f.name
    ) order by f.name asc), '[]'::json)
    from foundation f
    where f.landscape_url is not null;
$$ language sql;
