-- Returns matching locations as a json array.
create or replace function search_locations_json(p_ts_query text)
returns json as $$
    select coalesce(json_agg(json_build_object(
        'location_id', l.location_id,
        'city', l.city,
        'country', l.country,
        'state', l.state
    )), '[]'::json)
    from search_locations(p_ts_query) l;
$$ language sql;
