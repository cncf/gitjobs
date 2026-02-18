-- Returns projects that match the provided foundation and project query.
create or replace function search_projects(p_foundation text, p_project text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'foundation', p.foundation,
        'logo_url', p.logo_url,
        'maturity', p.maturity,
        'name', p.name,
        'project_id', p.project_id
    ) order by p.name asc), '[]'::json)
    from (
        select
            foundation,
            logo_url,
            maturity,
            name,
            project_id
        from project
        where foundation = p_foundation
        and name ilike '%' || p_project || '%'
        order by name asc
        limit 20
    ) p;
$$ language sql;
