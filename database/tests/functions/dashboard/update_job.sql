-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set certOldID '00000000-0000-0000-0000-000000000501'
\set certNewID '00000000-0000-0000-0000-000000000502'
\set deletedJobID '00000000-0000-0000-0000-000000000302'
\set employerID '00000000-0000-0000-0000-000000000101'
\set location1ID '00000000-0000-0000-0000-000000000201'
\set location2ID '00000000-0000-0000-0000-000000000202'
\set projectNewID '00000000-0000-0000-0000-000000000402'
\set projectOldID '00000000-0000-0000-0000-000000000401'
\set updatableJobID '00000000-0000-0000-0000-000000000301'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for update_job tests', :'employerID');

insert into location (city, country, location_id, state) values
    ('Madrid', 'Spain', :'location1ID', null),
    ('Porto', 'Portugal', :'location2ID', null);

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/project-old.svg', 'graduated', 'Old Project', :'projectOldID'),
    ('cncf', 'https://example.com/project-new.svg', 'incubating', 'New Project', :'projectNewID');

insert into certification (
    certification_id,
    description,
    logo_url,
    name,
    provider,
    short_name,
    url
) values
    (
        :'certOldID',
        'Old cert',
        'https://example.com/cert-old.svg',
        'Old Cert',
        'Provider',
        'OLD',
        'https://example.com/cert-old'
    ),
    (
        :'certNewID',
        'New cert',
        'https://example.com/cert-new.svg',
        'New Cert',
        'Provider',
        'NEW',
        'https://example.com/cert-new'
    );

insert into job (
    description,
    employer_id,
    job_id,
    kind,
    location_id,
    status,
    title,
    workplace
) values
    (
        'Old description',
        :'employerID',
        :'updatableJobID',
        'full-time',
        :'location1ID',
        'draft',
        'Old title',
        'remote'
    ),
    (
        'Deleted description',
        :'employerID',
        :'deletedJobID',
        'full-time',
        :'location1ID',
        'deleted',
        'Deleted title',
        'remote'
    );

insert into job_project (job_id, project_id) values
    (:'updatableJobID', :'projectOldID'),
    (:'deletedJobID', :'projectOldID');

insert into job_certification (certification_id, job_id) values
    (:'certOldID', :'updatableJobID'),
    (:'certOldID', :'deletedJobID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update all fields for non-deleted jobs
select dashboard_employer_update_job(
    :'updatableJobID'::uuid,
    jsonb_build_object(
        'apply_instructions', 'Apply by email',
        'apply_url', 'https://example.com/apply',
        'benefits', jsonb_build_array('healthcare'),
        'certifications', jsonb_build_array(
            jsonb_build_object('certification_id', :'certNewID'::text)
        ),
        'description', 'New description',
        'kind', 'contractor',
        'location', jsonb_build_object('location_id', :'location2ID'::text),
        'open_source', 80,
        'projects', jsonb_build_array(jsonb_build_object('project_id', :'projectNewID'::text)),
        'qualifications', 'Rust experience',
        'responsibilities', 'Build APIs',
        'salary', 90000,
        'salary_currency', 'USD',
        'salary_max', 110000,
        'salary_max_usd_year', 110000,
        'salary_min', 80000,
        'salary_min_usd_year', 80000,
        'salary_period', 'year',
        'salary_usd_year', 90000,
        'seniority', 'mid',
        'skills', jsonb_build_array('rust'),
        'status', 'pending-approval',
        'title', 'New title',
        'tz_end', 'UTC+1',
        'tz_start', 'UTC-2',
        'upstream_commitment', 70,
        'workplace', 'hybrid'
    )
);

select ok(
    exists (
        select 1
        from job j
        where j.job_id = :'updatableJobID'::uuid
        and j.apply_instructions = 'Apply by email'
        and j.apply_url = 'https://example.com/apply'
        and j.benefits = array['healthcare']::text[]
        and j.description = 'New description'
        and j.kind = 'contractor'
        and j.location_id = :'location2ID'::uuid
        and j.open_source = 80
        and j.qualifications = 'Rust experience'
        and j.responsibilities = 'Build APIs'
        and j.salary = 90000
        and j.salary_currency = 'USD'
        and j.salary_max = 110000
        and j.salary_max_usd_year = 110000
        and j.salary_min = 80000
        and j.salary_min_usd_year = 80000
        and j.salary_period = 'year'
        and j.salary_usd_year = 90000
        and j.seniority = 'mid'
        and j.skills = array['rust']::text[]
        and j.status = 'pending-approval'
        and j.title = 'New title'
        and j.tz_end = 'UTC+1'
        and j.tz_start = 'UTC-2'
        and j.upstream_commitment = 70
        and j.workplace = 'hybrid'
        and j.updated_at is not null
    ),
    'Should update all fields for non-deleted jobs'
);

-- Should replace projects and certifications for updated jobs
select is(
    (
        select jsonb_build_object(
            'certification_ids',
            (
                select coalesce(
                    jsonb_agg(jc.certification_id::text order by jc.certification_id::text),
                    '[]'::jsonb
                )
                from job_certification jc
                where jc.job_id = :'updatableJobID'::uuid
            ),
            'project_ids',
            (
                select coalesce(
                    jsonb_agg(jp.project_id::text order by jp.project_id::text),
                    '[]'::jsonb
                )
                from job_project jp
                where jp.job_id = :'updatableJobID'::uuid
            )
        )
    ),
    jsonb_build_object(
        'certification_ids', jsonb_build_array(:'certNewID'::text),
        'project_ids', jsonb_build_array(:'projectNewID'::text)
    ),
    'Should replace projects and certifications for updated jobs'
);

-- Should ignore deleted jobs and keep their existing relations
select dashboard_employer_update_job(
    :'deletedJobID'::uuid,
    jsonb_build_object(
        'description', 'Should not update',
        'kind', 'part-time',
        'status', 'draft',
        'title', 'Should not update',
        'workplace', 'on-site'
    )
);

select ok(
    exists (
        select 1
        from job j
        where j.job_id = :'deletedJobID'::uuid
        and j.description = 'Deleted description'
        and j.kind = 'full-time'
        and j.status = 'deleted'
        and j.title = 'Deleted title'
        and j.workplace = 'remote'
    )
    and exists (
        select 1 from job_project where job_id = :'deletedJobID'::uuid and project_id = :'projectOldID'::uuid
    )
    and exists (
        select 1
        from job_certification
        where job_id = :'deletedJobID'::uuid
        and certification_id = :'certOldID'::uuid
    ),
    'Should ignore deleted jobs and keep their existing relations'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
