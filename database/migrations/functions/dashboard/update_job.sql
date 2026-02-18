-- Updates a non-deleted job and refreshes its projects/certifications.
create or replace function update_job(p_job_id uuid, p_job jsonb)
returns void as $$
begin
    -- Update job fields from payload
    update job
    set
        kind = p_job->>'kind',
        status = p_job->>'status',
        location_id = ((p_job->'location')->>'location_id')::uuid,
        workplace = p_job->>'workplace',
        title = p_job->>'title',
        description = p_job->>'description',
        apply_instructions = p_job->>'apply_instructions',
        apply_url = p_job->>'apply_url',
        benefits = (
            case
                when p_job ? 'benefits' and jsonb_typeof(p_job->'benefits') <> 'null' then
                    array(select jsonb_array_elements_text(p_job->'benefits'))
                else null
            end
        ),
        open_source = (p_job->>'open_source')::int,
        qualifications = p_job->>'qualifications',
        responsibilities = p_job->>'responsibilities',
        salary = (p_job->>'salary')::bigint,
        salary_usd_year = (p_job->>'salary_usd_year')::bigint,
        salary_currency = p_job->>'salary_currency',
        salary_min = (p_job->>'salary_min')::bigint,
        salary_min_usd_year = (p_job->>'salary_min_usd_year')::bigint,
        salary_max = (p_job->>'salary_max')::bigint,
        salary_max_usd_year = (p_job->>'salary_max_usd_year')::bigint,
        salary_period = p_job->>'salary_period',
        seniority = p_job->>'seniority',
        skills = (
            case
                when p_job ? 'skills' and jsonb_typeof(p_job->'skills') <> 'null' then
                    array(select jsonb_array_elements_text(p_job->'skills'))
                else null
            end
        ),
        tz_end = p_job->>'tz_end',
        tz_start = p_job->>'tz_start',
        upstream_commitment = (p_job->>'upstream_commitment')::int,
        updated_at = current_timestamp
    where job_id = p_job_id
    and status <> 'deleted';

    if found then
        -- Replace projects with payload values
        delete from job_project
        where job_id = p_job_id;

        insert into job_project (job_id, project_id)
        select
            p_job_id,
            (project->>'project_id')::uuid
        from jsonb_array_elements(
            case
                when p_job ? 'projects' and jsonb_typeof(p_job->'projects') <> 'null' then
                    p_job->'projects'
                else '[]'::jsonb
            end
        ) project;

        -- Replace certifications with payload values
        delete from job_certification
        where job_id = p_job_id;

        insert into job_certification (job_id, certification_id)
        select
            p_job_id,
            (certification->>'certification_id')::uuid
        from jsonb_array_elements(
            case
                when p_job ? 'certifications' and jsonb_typeof(p_job->'certifications') <> 'null' then
                    p_job->'certifications'
                else '[]'::jsonb
            end
        ) certification;
    end if;
end
$$ language plpgsql;
