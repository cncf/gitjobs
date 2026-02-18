-- Returns available certifications.
create or replace function list_certifications()
returns json as $$
    select coalesce(json_agg(json_build_object(
        'certification_id', c.certification_id,
        'name', c.name,
        'provider', c.provider,
        'short_name', c.short_name,
        'description', c.description,
        'logo_url', c.logo_url,
        'url', c.url
    ) order by c.name asc), '[]'::json)
    from certification c;
$$ language sql;
