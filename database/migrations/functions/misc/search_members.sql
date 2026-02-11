-- Returns members that match the provided foundation and member query.
create or replace function misc_search_members(p_foundation text, p_member text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'foundation', m.foundation,
        'level', m.level,
        'logo_url', m.logo_url,
        'member_id', m.member_id,
        'name', m.name
    ) order by m.name asc), '[]'::json)
    from (
        select
            foundation,
            level,
            logo_url,
            member_id,
            name
        from member
        where foundation = p_foundation
        and name ilike '%' || p_member || '%'
        order by name asc
        limit 20
    ) m;
$$ language sql;
