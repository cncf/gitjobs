-- Returns available filters options for job board searches.
create or replace function jobboard_get_jobs_filters_options()
returns json as $$
    select json_build_object(
        'foundations',
        coalesce(json_agg(json_build_object(
            'name', f.name
        ) order by f.name asc), '[]'::json)
    )
    from foundation f;
$$ language sql;
