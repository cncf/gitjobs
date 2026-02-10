-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(6);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set draftJobID '00000000-0000-0000-0000-000000000303'
\set employer1ID '00000000-0000-0000-0000-000000000101'
\set employer2ID '00000000-0000-0000-0000-000000000102'
\set job1ID '00000000-0000-0000-0000-000000000301'
\set job2ID '00000000-0000-0000-0000-000000000302'
\set location1ID '00000000-0000-0000-0000-000000000201'
\set location2ID '00000000-0000-0000-0000-000000000202'
\set memberCNCFID '00000000-0000-0000-0000-000000000401'
\set memberLFID '00000000-0000-0000-0000-000000000402'
\set projectCNCFID '00000000-0000-0000-0000-000000000501'
\set projectLFID '00000000-0000-0000-0000-000000000502'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Foundations and memberships
insert into foundation (name) values ('lf');

insert into member (foundation, level, logo_url, member_id, name) values
    ('cncf', 'platinum', 'https://example.com/member-cncf.svg', :'memberCNCFID', 'Acme Foundation'),
    ('lf', 'gold', 'https://example.com/member-lf.svg', :'memberLFID', 'Beta Foundation');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer one for search_jobs tests', :'employer1ID'),
    ('Beta Inc', 'Employer two for search_jobs tests', :'employer2ID');

insert into employer_member (employer_id, member_id) values
    (:'employer1ID', :'memberCNCFID'),
    (:'employer2ID', :'memberLFID');

-- Locations and projects
insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'location1ID', 'CA'),
    ('New York', 'United States', :'location2ID', 'NY');

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/project-cncf.svg', 'graduated', 'Kubernetes', :'projectCNCFID'),
    ('lf', 'https://example.com/project-lf.svg', 'incubating', 'OpenTofu', :'projectLFID');

-- Jobs
insert into job (
    job_id,
    employer_id,
    kind,
    seniority,
    status,
    location_id,
    workplace,
    created_at,
    description,
    published_at,
    salary,
    salary_currency,
    salary_max,
    salary_max_usd_year,
    salary_min,
    salary_min_usd_year,
    salary_period,
    skills,
    title,
    updated_at,
    upstream_commitment,
    open_source
) values
    (
        :'job1ID',
        :'employer1ID',
        'full-time',
        'senior',
        'published',
        :'location1ID',
        'remote',
        current_timestamp - interval '2 days',
        'Build Kubernetes control plane components',
        current_timestamp - interval '1 day',
        150000,
        'USD',
        200000,
        200000,
        100000,
        100000,
        'year',
        array['rust', 'kubernetes'],
        'Kubernetes Platform Engineer',
        current_timestamp - interval '1 day',
        70,
        80
    ),
    (
        :'job2ID',
        :'employer2ID',
        'full-time',
        'junior',
        'published',
        :'location2ID',
        'hybrid',
        current_timestamp - interval '3 days',
        'Build frontend interfaces',
        current_timestamp - interval '2 days',
        70000,
        'USD',
        100000,
        100000,
        60000,
        60000,
        'year',
        array['javascript', 'react'],
        'Frontend Engineer',
        current_timestamp - interval '2 days',
        30,
        20
    ),
    (
        :'draftJobID',
        :'employer1ID',
        'full-time',
        'lead',
        'draft',
        :'location1ID',
        'remote',
        current_timestamp - interval '1 day',
        'Draft role that must not appear',
        null,
        300000,
        'USD',
        350000,
        350000,
        250000,
        250000,
        'year',
        array['go'],
        'Draft Principal Engineer',
        current_timestamp - interval '1 day',
        90,
        95
    );

insert into job_project (job_id, project_id) values
    (:'job1ID', :'projectCNCFID'),
    (:'job2ID', :'projectLFID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return only published jobs by default
select is(
    (select total from search_jobs('{}'::jsonb)),
    2::bigint,
    'Should return only published jobs by default'
);

-- Should sort by published_at descending by default
select is(
    (
        select (jobs::jsonb->0->>'job_id')::uuid
        from search_jobs('{}'::jsonb)
    ),
    :'job1ID'::uuid,
    'Should sort by published_at descending by default'
);

-- Should filter by foundation
select is(
    (
        select total
        from search_jobs('{"foundation":"cncf"}'::jsonb)
    ),
    1::bigint,
    'Should filter by foundation'
);

-- Should filter by full text query with prefix matching
select is(
    (
        select total
        from search_jobs('{"ts_query":"kuber"}'::jsonb)
    ),
    1::bigint,
    'Should filter by full text query with prefix matching'
);

-- Should filter by employer membership foundation
select is(
    (
        select total
        from search_jobs('{"membership":"lf"}'::jsonb)
    ),
    1::bigint,
    'Should filter by employer membership foundation'
);

-- Should sort by salary when requested
select is(
    (
        select (jobs::jsonb->0->>'job_id')::uuid
        from search_jobs('{"sort":"salary"}'::jsonb)
    ),
    :'job1ID'::uuid,
    'Should sort by salary when requested'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
