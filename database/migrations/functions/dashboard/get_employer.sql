-- Returns employer details.
create or replace function dashboard_employer_get_employer(p_employer_id uuid)
returns json as $$
    select nullif(jsonb_strip_nulls(jsonb_build_object(
        'company', e.company,
        'description', e.description,
        'public', e.public,
        'logo_id', e.logo_id,
        'website_url', e.website_url,
        'location', nullif(jsonb_strip_nulls(jsonb_build_object(
            'location_id', l.location_id,
            'city', l.city,
            'country', l.country,
            'state', l.state
        )), '{}'::jsonb),
        'members', (
            select json_agg(json_build_object(
                'member_id', m.member_id,
                'foundation', m.foundation,
                'level', m.level,
                'logo_url', m.logo_url,
                'name', m.name
            ) order by m.foundation asc, m.name asc)
            from employer_member em
            join member m on em.member_id = m.member_id
            where em.employer_id = e.employer_id
        )
    )), '{}'::jsonb)
    from employer e
    left join location l using (location_id)
    where e.employer_id = p_employer_id;
$$ language sql;
