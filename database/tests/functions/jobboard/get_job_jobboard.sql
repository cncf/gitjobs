-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set certificationID '00000000-0000-0000-0000-000000000601'
\set employerID '00000000-0000-0000-0000-000000000101'
\set jobDraftID '00000000-0000-0000-0000-000000000302'
\set jobPublishedID '00000000-0000-0000-0000-000000000301'
\set locationID '00000000-0000-0000-0000-000000000201'
\set memberID '00000000-0000-0000-0000-000000000401'
\set projectID '00000000-0000-0000-0000-000000000501'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into location (city, country, location_id, state) values
    ('Lisbon', 'Portugal', :'locationID', null);

insert into member (foundation, level, logo_url, member_id, name) values
    ('cncf', 'gold', 'https://example.com/member.svg', :'memberID', 'Acme Member');

insert into employer (company, description, employer_id, location_id, website_url) values
    (
        'Acme Corp',
        'Employer description',
        :'employerID',
        :'locationID',
        'https://acme.example'
    );

insert into employer_member (employer_id, member_id) values
    (:'employerID', :'memberID');

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

insert into job (
    apply_instructions,
    apply_url,
    benefits,
    description,
    employer_id,
    job_id,
    kind,
    location_id,
    open_source,
    published_at,
    qualifications,
    responsibilities,
    salary,
    salary_currency,
    salary_max,
    salary_min,
    salary_period,
    seniority,
    skills,
    status,
    title,
    tz_end,
    tz_start,
    updated_at,
    upstream_commitment,
    workplace
) values
    (
        'Apply by email',
        'https://example.com/apply',
        array['healthcare', 'remote'],
        'Published role',
        :'employerID',
        :'jobPublishedID',
        'full-time',
        :'locationID',
        80,
        '2026-01-01 10:00:00+00',
        'Rust experience',
        'Build APIs',
        120000,
        'USD',
        150000,
        100000,
        'year',
        'senior',
        array['rust', 'postgresql'],
        'published',
        'Platform Engineer',
        'UTC+2',
        'UTC-3',
        '2026-01-01 11:00:00+00',
        70,
        'remote'
    ),
    (
        null,
        null,
        null,
        'Draft role',
        :'employerID',
        :'jobDraftID',
        'full-time',
        :'locationID',
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        null,
        'draft',
        'Draft Job',
        null,
        null,
        null,
        null,
        'remote'
    );

insert into job_project (job_id, project_id) values
    (:'jobPublishedID', :'projectID');

insert into job_certification (certification_id, job_id) values
    (:'certificationID', :'jobPublishedID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload for published jobs
select is(
    get_job_jobboard(:'jobPublishedID'::uuid)::jsonb,
    jsonb_build_object(
        'apply_instructions', 'Apply by email',
        'apply_url', 'https://example.com/apply',
        'benefits', jsonb_build_array('healthcare', 'remote'),
        'certifications', jsonb_build_array(
            jsonb_build_object(
                'certification_id', :'certificationID'::text,
                'description', 'Certification description',
                'logo_url', 'https://example.com/cert.svg',
                'name', 'Certification',
                'provider', 'Provider',
                'short_name', 'CERT',
                'url', 'https://example.com/cert'
            )
        ),
        'description', 'Published role',
        'employer', jsonb_build_object(
            'company', 'Acme Corp',
            'description', 'Employer description',
            'employer_id', :'employerID'::text,
            'members', jsonb_build_array(
                jsonb_build_object(
                    'foundation', 'cncf',
                    'level', 'gold',
                    'logo_url', 'https://example.com/member.svg',
                    'member_id', :'memberID'::text,
                    'name', 'Acme Member'
                )
            ),
            'website_url', 'https://acme.example'
        ),
        'job_id', :'jobPublishedID'::text,
        'kind', 'full-time',
        'location', jsonb_build_object(
            'city', 'Lisbon',
            'country', 'Portugal',
            'location_id', :'locationID'::text
        ),
        'open_source', 80,
        'projects', jsonb_build_array(
            jsonb_build_object(
                'foundation', 'cncf',
                'logo_url', 'https://example.com/project.svg',
                'maturity', 'graduated',
                'name', 'Kubernetes',
                'project_id', :'projectID'::text
            )
        ),
        'published_at', '2026-01-01 10:00:00+00'::timestamptz,
        'qualifications', 'Rust experience',
        'responsibilities', 'Build APIs',
        'salary', 120000,
        'salary_currency', 'USD',
        'salary_max', 150000,
        'salary_min', 100000,
        'salary_period', 'year',
        'seniority', 'senior',
        'skills', jsonb_build_array('rust', 'postgresql'),
        'title', 'Platform Engineer',
        'tz_end', 'UTC+2',
        'tz_start', 'UTC-3',
        'updated_at', '2026-01-01 11:00:00+00'::timestamptz,
        'upstream_commitment', 70,
        'workplace', 'remote'
    ),
    'Should return full payload for published jobs'
);

-- Should return null for non-published jobs
select is(
    get_job_jobboard(:'jobDraftID'::uuid)::jsonb,
    null::jsonb,
    'Should return null for non-published jobs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
