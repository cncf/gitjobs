-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set certificationID '00000000-0000-0000-0000-000000000401'
\set employerID '00000000-0000-0000-0000-000000000101'
\set locationID '00000000-0000-0000-0000-000000000201'
\set projectID '00000000-0000-0000-0000-000000000301'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for add_job tests', :'employerID');

insert into location (city, country, location_id, state) values
    ('Lisbon', 'Portugal', :'locationID', null);

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/project.svg', 'graduated', 'Kubernetes', :'projectID');

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
        :'certificationID',
        'Certification description',
        'https://example.com/cert.svg',
        'Certification',
        'Provider',
        'CERT',
        'https://example.com/cert'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should insert all job fields from payload
select add_job(
    :'employerID'::uuid,
    jsonb_build_object(
        'apply_instructions', 'Apply with resume',
        'apply_url', 'https://example.com/apply',
        'benefits', jsonb_build_array('healthcare', 'remote'),
        'certifications', jsonb_build_array(
            jsonb_build_object('certification_id', :'certificationID'::text)
        ),
        'description', 'Build platform services',
        'kind', 'full-time',
        'location', jsonb_build_object('location_id', :'locationID'::text),
        'open_source', 75,
        'projects', jsonb_build_array(jsonb_build_object('project_id', :'projectID'::text)),
        'qualifications', 'Strong Rust experience',
        'responsibilities', 'Build and maintain APIs',
        'salary', 120000,
        'salary_currency', 'USD',
        'salary_max', 150000,
        'salary_max_usd_year', 150000,
        'salary_min', 100000,
        'salary_min_usd_year', 100000,
        'salary_period', 'year',
        'salary_usd_year', 120000,
        'seniority', 'senior',
        'skills', jsonb_build_array('rust', 'postgresql'),
        'status', 'draft',
        'title', 'Platform Engineer',
        'tz_end', 'UTC+2',
        'tz_start', 'UTC-3',
        'upstream_commitment', 80,
        'workplace', 'remote'
    )
);

select is(
    (
        select jsonb_build_object(
            'apply_instructions', apply_instructions,
            'apply_url', apply_url,
            'benefits', to_jsonb(benefits),
            'description', description,
            'employer_id', employer_id::text,
            'kind', kind,
            'location_id', location_id::text,
            'open_source', open_source,
            'qualifications', qualifications,
            'responsibilities', responsibilities,
            'salary', salary,
            'salary_currency', salary_currency,
            'salary_max', salary_max,
            'salary_max_usd_year', salary_max_usd_year,
            'salary_min', salary_min,
            'salary_min_usd_year', salary_min_usd_year,
            'salary_period', salary_period,
            'salary_usd_year', salary_usd_year,
            'seniority', seniority,
            'skills', to_jsonb(skills),
            'status', status,
            'title', title,
            'tz_end', tz_end,
            'tz_start', tz_start,
            'upstream_commitment', upstream_commitment,
            'workplace', workplace
        )
        from job
        where employer_id = :'employerID'::uuid
    ),
    jsonb_build_object(
        'apply_instructions', 'Apply with resume',
        'apply_url', 'https://example.com/apply',
        'benefits', jsonb_build_array('healthcare', 'remote'),
        'description', 'Build platform services',
        'employer_id', :'employerID'::text,
        'kind', 'full-time',
        'location_id', :'locationID'::text,
        'open_source', 75,
        'qualifications', 'Strong Rust experience',
        'responsibilities', 'Build and maintain APIs',
        'salary', 120000,
        'salary_currency', 'USD',
        'salary_max', 150000,
        'salary_max_usd_year', 150000,
        'salary_min', 100000,
        'salary_min_usd_year', 100000,
        'salary_period', 'year',
        'salary_usd_year', 120000,
        'seniority', 'senior',
        'skills', jsonb_build_array('rust', 'postgresql'),
        'status', 'draft',
        'title', 'Platform Engineer',
        'tz_end', 'UTC+2',
        'tz_start', 'UTC-3',
        'upstream_commitment', 80,
        'workplace', 'remote'
    ),
    'Should insert all job fields from payload'
);

-- Should insert job projects and certifications from payload
select is(
    (
        with inserted_job as (
            select job_id
            from job
            where employer_id = :'employerID'::uuid
        )
        select jsonb_build_object(
            'certification_ids',
            (
                select coalesce(
                    jsonb_agg(jc.certification_id::text order by jc.certification_id::text),
                    '[]'::jsonb
                )
                from job_certification jc
                join inserted_job i on jc.job_id = i.job_id
            ),
            'project_ids',
            (
                select coalesce(jsonb_agg(jp.project_id::text order by jp.project_id::text), '[]'::jsonb)
                from job_project jp
                join inserted_job i on jp.job_id = i.job_id
            )
        )
    ),
    jsonb_build_object(
        'certification_ids', jsonb_build_array(:'certificationID'::text),
        'project_ids', jsonb_build_array(:'projectID'::text)
    ),
    'Should insert job projects and certifications from payload'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
