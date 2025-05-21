-- Returns some stats about the job board in json format.
create or replace function get_stats()
returns json as $$
    select json_strip_nulls(json_build_object(
        'generated_at', floor(extract(epoch from current_timestamp) * 1000),
        'jobs', json_build_object(
            'published_running_total', (
                select json_agg(json_build_array(
                    floor(extract(epoch from jobs_day) * 1000),
                    running_total
                ))
                from (
                    select
                        jobs_day,
                        sum(total) over (order by jobs_day asc) as running_total
                    from (
                        select
                            date_trunc('day', first_published_at) as jobs_day,
                            count(*) as total
                        from job
                        where first_published_at is not null
                        group by date_trunc('day', first_published_at)
                    ) mt
                ) rt
            ),
            'published_per_foundation', (
                select json_agg(json_build_array(foundation, jobs))
                from (
                    select foundation, count(*) as jobs
                    from (
                        select distinct f.name as foundation, j.job_id
                        from job j join job_project jp on j.job_id = jp.job_id
                        join project p on jp.project_id = p.project_id
                        join foundation f on p.foundation = f.name
                    )
                    group by foundation
                    order by jobs desc
                ) foundation_jobs
            ),
            'published_per_month', (
                select json_agg(row_to_json(year_month_count))
                from (
                    select
                        extract('year' from first_published_at) as year,
                        extract('month' from first_published_at) as month,
                        count(*) as total
                    from job
                    where first_published_at is not null
                    group by
                        extract('year' from first_published_at),
                        extract('month' from first_published_at)
                    order by year desc, month desc
                ) year_month_count
            ),
            'views_daily', (
                select json_agg(json_build_array(
                    floor(extract(epoch from day) * 1000),
                    total
                ))
                from (
                    select day, sum(total) as total
                    from job_views
                    where day >= current_date - '1 month'::interval
                    group by day
                    order by day asc
                ) dt
            ),
            'views_monthly', (
                select json_agg(json_build_array(
                    floor(extract(epoch from month) * 1000),
                    total
                ))
                from (
                    select date_trunc('month', day) as month, sum(total) as total
                    from job_views
                    where day >= current_date - '2 year'::interval
                    group by month
                    order by month asc
                ) mt
            )
        )
    ));
$$ language sql;
