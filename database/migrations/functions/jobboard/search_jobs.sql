-- Returns the jobs that match the filters provided.
create or replace function search_jobs(p_board_id uuid, p_filters jsonb)
returns table(jobs json, total bigint) as $$
declare
    v_benefits text[];
    v_date_from date := (p_filters->>'date_from');
    v_date_to date := (p_filters->>'date_to');
    v_max_distance real := (p_filters->>'max_distance')::real;
    v_kind text[];
    v_limit int := coalesce((p_filters->>'limit')::int, 10);
    v_location_id uuid := (p_filters->>'location_id')::uuid;
    v_offset int := coalesce((p_filters->>'offset')::int, 0);
    v_open_source int := (p_filters->>'open_source')::int;
    v_projects text[];
    v_salary_min bigint := (p_filters->>'salary_min')::bigint;
    v_seniority text := (p_filters->>'seniority');
    v_skills text[];
    v_tsquery_with_prefix_matching tsquery;
    v_upstream_commitment int := (p_filters->>'upstream_commitment')::int;
    v_workplace text[];
begin
    -- Prepare filters
    if p_filters ? 'benefits' then
        select array_agg(e::text) into v_benefits
        from jsonb_array_elements_text(p_filters->'benefits') e;
    end if;
    if p_filters ? 'kind' then
        select array_agg(e::text) into v_kind
        from jsonb_array_elements_text(p_filters->'kind') e;
    end if;
    if p_filters ? 'projects' then
        select array_agg(e::text) into v_projects
        from jsonb_array_elements_text(p_filters->'projects') e;
    end if;
    if p_filters ? 'skills' then
        select array_agg(e::text) into v_skills
        from jsonb_array_elements_text(p_filters->'skills') e;
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
            j.job_id,
            j.kind,
            j.published_at,
            j.title,
            j.workplace,
            j.open_source,
            j.salary,
            j.salary_currency,
            j.salary_min,
            j.salary_max,
            j.salary_period,
            j.seniority,
            j.updated_at,
            j.upstream_commitment,
            (
                select nullif(jsonb_strip_nulls(jsonb_build_object(
                    'company', e.company,
                    'employer_id', e.employer_id,
                    'logo_id', e.logo_id,
                    'website_url', e.website_url,
                    'member', (
                        select nullif(jsonb_strip_nulls(jsonb_build_object(
                            'member_id', m.member_id,
                            'name', m.name,
                            'level', m.level,
                            'logo_url', m.logo_url
                        )), '{}'::jsonb)
                    )
                )), '{}'::jsonb)
            ) as employer,
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
        left join member m on e.member_id = m.member_id
        where e.job_board_id = p_board_id
        and j.status = 'published'
        and
            case when cardinality(v_benefits) > 0 then
                j.benefits @> v_benefits
            else true end
        and
            case when v_date_from is not null then
                j.published_at >= v_date_from
            else true end
        and
            case when v_date_to is not null then
                j.published_at <= v_date_to
            else true end
        and
            case when cardinality(v_kind) > 0 then
                j.kind = any(v_kind)
            else true end
        and
            case when v_location_id is not null and v_max_distance is not null then
                st_dwithin(
                    (select coordinates from location where location_id = v_location_id),
                    (select coordinates from location where location_id = j.location_id),
                    v_max_distance
                )
            else true end
        and
            case when v_open_source is not null then
                j.open_source >= v_open_source
            else true end
        and
            case when cardinality(v_projects) > 0 then
                j.job_id = any(
                    select job_id from job_project
                    where project_id = any(
                        select project_id from project
                        where name = any(v_projects)
                    )
            ) else true end
        and
            case when v_salary_min is not null then
                case
                    when j.salary is not null then j.salary >= v_salary_min
                    when j.salary_min is not null then j.salary_min >= v_salary_min
                    else false
                end
            else true end
        and
            case when v_seniority is not null then
                j.seniority = v_seniority
            else true end
        and
            case when cardinality(v_skills) > 0 then
                j.skills @> v_skills
            else true end
        and
            case when v_tsquery_with_prefix_matching is not null then
                v_tsquery_with_prefix_matching @@ j.tsdoc
            else true end
        and
            case when v_upstream_commitment is not null then
                j.upstream_commitment >= v_upstream_commitment
            else true end
        and
            case when cardinality(v_workplace) > 0 then
                j.workplace = any(v_workplace)
            else true end
    )
    select
        (
            select coalesce(json_agg(json_build_object(
                'job_id', job_id,
                'kind', kind,
                'published_at', published_at,
                'title', title,
                'workplace', workplace,
                'open_source', open_source,
                'salary', salary,
                'salary_currency', salary_currency,
                'salary_min', salary_min,
                'salary_max', salary_max,
                'salary_period', salary_period,
                'seniority', seniority,
                'updated_at', updated_at,
                'upstream_commitment', upstream_commitment,
                'employer', employer,
                'location', location,
                'projects', projects
            )), '[]')
            from (
                select *
                from filtered_jobs
                order by published_at desc
                limit v_limit
                offset v_offset
            ) filtered_jobs_page
        ),
        (
            select count(*) from filtered_jobs
        );
end
$$ language plpgsql;
