-- Returns the locations that match the query provided.
create or replace function search_locations(p_ts_query text)
returns table(
    location_id uuid,
    city text,
    country text,
    state text
) as $$
declare
    v_ts_query_with_prefix_matching tsquery;
begin
    -- Prepare ts query with prefix matching
    select ts_rewrite(
        websearch_to_tsquery(p_ts_query),
        format(
            '
            select
                to_tsquery(lexeme),
                to_tsquery(lexeme || '':*'')
            from unnest(tsvector_to_array(to_tsvector(%L))) as lexeme
            ', p_ts_query
        )
    ) into v_ts_query_with_prefix_matching;

    return query
    select
        lws.location_id,
        lws.city,
        lws.country,
        lws.state
    from (
        select
            l.location_id,
            l.city,
            l.country,
            l.state,
            ts_rank(tsdoc, v_ts_query_with_prefix_matching, 1) as score
        from location l
        where v_ts_query_with_prefix_matching @@ tsdoc
        order by score desc
        limit 20
    ) as lws;
end
$$ language plpgsql;
