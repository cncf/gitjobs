-- Returns the applicants that match the filters provided.
create or replace function search_applicants(
    p_employer_id uuid,
    p_filters jsonb
)
returns table(applicants json, total bigint) as $$
declare
    v_job_id uuid := (p_filters->>'job_id')::uuid;
    v_limit int := coalesce((p_filters->>'limit')::int, 10);
    v_offset int := coalesce((p_filters->>'offset')::int, 0);
begin
    return query
    with filtered_applicants as (
        select
            a.applicant_id,
            a.created_at as applied_at,
            j.job_id,
            j.title as job_title,
            (
                select nullif(jsonb_strip_nulls(jsonb_build_object(
                    'location_id', l.location_id,
                    'city', l.city,
                    'country', l.country,
                    'state', l.state
                )), '{}'::jsonb)
            ) as job_location,
            p.job_seeker_profile_id,
            p.photo_id,
            p.name,
            (
                select format(
                    '%s at %s', experience->>'title', experience->>'company'
                ) as last_position
                from (
                    select jsonb_array_elements(p.experience) as experience
                )
                order by (experience->>'end_date')::date desc nulls first
                limit 1
            ) as last_position
        from applicant a
        join job j on a.job_id = j.job_id
        join job_seeker_profile p on a.job_seeker_profile_id = p.job_seeker_profile_id
        left join location l on j.location_id = l.location_id
        where j.employer_id = p_employer_id
        and
            case when v_job_id is not null then
            a.job_id = v_job_id else true end
    )
    select
        (
            select coalesce(json_agg(json_build_object(
                'applicant_id', applicant_id,
                'applied_at', applied_at,
                'job_id', job_id,
                'job_title', job_title,
                'job_location', job_location,
                'job_seeker_profile_id', job_seeker_profile_id,
                'photo_id', photo_id,
                'name', name,
                'last_position', last_position
            )), '[]')
            from (
                select *
                from filtered_applicants
                order by applied_at desc
                limit v_limit
                offset v_offset
            ) filtered_applicants_page
        ),
        (
            select count(*) from filtered_applicants
        );
end
$$ language plpgsql;
