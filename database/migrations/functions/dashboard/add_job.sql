-- Adds a job and its projects/certifications for an employer.
create or replace function add_job(p_employer_id uuid, p_job jsonb)
returns void as $$
declare
    v_job_id uuid;
begin
    -- Insert job base data
    insert into job (
        employer_id,
        kind,
        status,
        location_id,
        workplace,
        title,
        description,
        apply_instructions,
        apply_url,
        benefits,
        open_source,
        qualifications,
        responsibilities,
        salary,
        salary_usd_year,
        salary_currency,
        salary_min,
        salary_min_usd_year,
        salary_max,
        salary_max_usd_year,
        salary_period,
        seniority,
        skills,
        tz_end,
        tz_start,
        upstream_commitment
    ) values (
        p_employer_id,
        p_job->>'kind',
        p_job->>'status',
        ((p_job->'location')->>'location_id')::uuid,
        p_job->>'workplace',
        p_job->>'title',
        p_job->>'description',
        p_job->>'apply_instructions',
        p_job->>'apply_url',
        (
            case
                when p_job ? 'benefits' and jsonb_typeof(p_job->'benefits') <> 'null' then
                    array(select jsonb_array_elements_text(p_job->'benefits'))
                else null
            end
        ),
        (p_job->>'open_source')::int,
        p_job->>'qualifications',
        p_job->>'responsibilities',
        (p_job->>'salary')::bigint,
        (p_job->>'salary_usd_year')::bigint,
        p_job->>'salary_currency',
        (p_job->>'salary_min')::bigint,
        (p_job->>'salary_min_usd_year')::bigint,
        (p_job->>'salary_max')::bigint,
        (p_job->>'salary_max_usd_year')::bigint,
        p_job->>'salary_period',
        p_job->>'seniority',
        (
            case
                when p_job ? 'skills' and jsonb_typeof(p_job->'skills') <> 'null' then
                    array(select jsonb_array_elements_text(p_job->'skills'))
                else null
            end
        ),
        p_job->>'tz_end',
        p_job->>'tz_start',
        (p_job->>'upstream_commitment')::int
    )
    returning job_id into v_job_id;

    -- Insert related projects
    insert into job_project (job_id, project_id)
    select
        v_job_id,
        (project->>'project_id')::uuid
    from jsonb_array_elements(
        case
            when p_job ? 'projects' and jsonb_typeof(p_job->'projects') <> 'null' then
                p_job->'projects'
            else '[]'::jsonb
        end
    ) project;

    -- Insert required certifications
    insert into job_certification (job_id, certification_id)
    select
        v_job_id,
        (certification->>'certification_id')::uuid
    from jsonb_array_elements(
        case
            when p_job ? 'certifications' and jsonb_typeof(p_job->'certifications') <> 'null' then
                p_job->'certifications'
            else '[]'::jsonb
        end
    ) certification;
end
$$ language plpgsql;
