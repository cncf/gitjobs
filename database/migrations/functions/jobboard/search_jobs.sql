-- Returns the jobs that match the filters provided.
create or replace function search_jobs(p_board_id uuid, p_filters jsonb)
returns table(jobs json, total bigint) as $$
declare
    v_kind text[];
    v_workplace text[];
    v_limit int := coalesce((p_filters->>'limit')::int, 10);
    v_offset int := coalesce((p_filters->>'offset')::int, 0);
    v_sort_by text := coalesce(p_filters->>'sort_by', 'date');
    v_tsquery_with_prefix_matching tsquery;
begin
    -- Prepare filters
    if p_filters ? 'kind' then
        select array_agg(e::text) into v_kind
        from jsonb_array_elements_text(p_filters->'kind') e;
    end if;
    if p_filters ? 'workplace' then
        select array_agg(e::text) into v_workplace
        from jsonb_array_elements_text(p_filters->'workplace') e;
    end if;
    if p_filters ? 'ts_query' then
        select ts_rewrite(
            websearch_to_tsquery(p_filters->>'ts_query'),
            format('
                select
                    to_tsquery(lexeme),
                    to_tsquery(lexeme || '':*'')
                from unnest(tsvector_to_array(to_tsvector(%L))) as lexeme
                ', p_filters->>'ts_query'
            )
        ) into v_tsquery_with_prefix_matching;
    end if;

    return query
    with filtered_jobs as (
        select
            j.description,
            j.status,
            j.title,
            j.kind,
            j.workplace,
            j.apply_instructions,
            j.apply_url,
            j.benefits,
            j.job_id,
            j.open_source,
            j.published_at,
            j.qualifications,
            j.responsibilities,
            j.salary,
            j.salary_currency,
            j.salary_min,
            j.salary_max,
            j.salary_period,
            j.skills,
            j.updated_at,
            j.upstream_commitment,
            (
                select nullif(jsonb_strip_nulls(jsonb_build_object(
                    'location_id', l.location_id,
                    'city', l.city,
                    'country', l.country,
                    'state', l.state
                )), '{}'::jsonb)
            ) as location,
            (
                select json_agg(json_build_object(
                    'project_id', p.project_id,
                    'name', p.name,
                    'maturity', p.maturity,
                    'logo_url', p.logo_url
                ))
                from project p
                left join job_project using (project_id)
                left join job using (job_id)
                where job_id = j.job_id
            ) as projects
        from job j
        join employer e on j.employer_id = e.employer_id
        left join location l on j.location_id = l.location_id
        where e.job_board_id = p_board_id
        and j.status = 'published'
        and
            case when cardinality(v_kind) > 0 then
            j.kind = any(v_kind) else true end
        and
            case when cardinality(v_workplace) > 0 then
            j.workplace = any(v_workplace) else true end
        and
            case when v_tsquery_with_prefix_matching is not null then
                v_tsquery_with_prefix_matching @@ j.tsdoc
            else true end
    )
    select
        (
            select coalesce(json_agg(json_build_object(
                'description', description,
                'status', status,
                'title', title,
                'kind', kind,
                'workplace', workplace,
                'apply_instructions', apply_instructions,
                'apply_url', apply_url,
                'benefits', benefits,
                'job_id', job_id,
                'open_source', open_source,
                'published_at', published_at,
                'qualifications', qualifications,
                'responsibilities', responsibilities,
                'salary', salary,
                'salary_currency', salary_currency,
                'salary_min', salary_min,
                'salary_max', salary_max,
                'salary_period', salary_period,
                'skills', skills,
                'updated_at', updated_at,
                'upstream_commitment', upstream_commitment,
                'location', location,
                'projects', projects
            )), '[]')
            from (
                select *
                from filtered_jobs
                order by
                    (case when v_sort_by = 'date' then published_at end) desc
                limit v_limit
                offset v_offset
            ) filtered_jobs_page
        ),
        (
            select count(*) from filtered_jobs
        );
end
$$ language plpgsql;
