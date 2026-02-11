-- Lists all projects for the provided foundation.
create or replace function syncer_list_projects(p_foundation text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'foundation', p_foundation,
        'logo_url', p.logo_url,
        'maturity', p.maturity,
        'name', p.name
    ) order by p.name asc), '[]'::json)
    from project p
    where p.foundation = p_foundation;
$$ language sql;
