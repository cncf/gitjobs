-- Returns stats for a specific job in json format.
create or replace function get_job_stats(p_job_id uuid)
returns json as $$
    select json_strip_nulls(json_build_object(
        'search_appearances_daily', (
            select coalesce(json_agg(json_build_array(
                floor(extract(epoch from day) * 1000),
                total
            )), '[]'::json)
            from (
                select day, total
                from search_appearances
                where job_id = p_job_id
                and day >= current_date - '1 month'::interval
                order by day asc
            ) daily_search_appearances
        ),
        'search_appearances_total_last_month', (
            select coalesce(sum(total), 0)
            from search_appearances
            where job_id = p_job_id
            and day >= current_date - '1 month'::interval
        ),
        'views_daily', (
            select coalesce(json_agg(json_build_array(
                floor(extract(epoch from day) * 1000),
                total
            )), '[]'::json)
            from (
                select day, total
                from job_views
                where job_id = p_job_id
                and day >= current_date - '1 month'::interval
                order by day asc
            ) daily_views
        ),
        'views_total_last_month', (
            select coalesce(sum(total), 0)
            from job_views
            where job_id = p_job_id
            and day >= current_date - '1 month'::interval
        )
    ));
$$ language sql;
