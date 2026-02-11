-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(5);

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
        '2026-01-01 10:00:00+00',
        'Build Kubernetes control plane components',
        '2026-01-03 10:00:00+00',
        150000,
        'USD',
        200000,
        200000,
        100000,
        100000,
        'year',
        array['rust', 'kubernetes'],
        'Kubernetes Platform Engineer',
        '2026-01-03 11:00:00+00',
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
        '2025-12-31 10:00:00+00',
        'Build frontend interfaces',
        '2026-01-02 10:00:00+00',
        70000,
        'USD',
        100000,
        100000,
        60000,
        60000,
        'year',
        array['javascript', 'react'],
        'Frontend Engineer',
        '2026-01-02 11:00:00+00',
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
        '2025-12-30 10:00:00+00',
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
        '2025-12-30 11:00:00+00',
        90,
        95
    );

insert into job_project (job_id, project_id) values
    (:'job1ID', :'projectCNCFID'),
    (:'job2ID', :'projectLFID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload for published jobs by default
select is(
    jobboard_search_jobs('{}'::jsonb)::jsonb,
    jsonb_build_object(
        'jobs',
        jsonb_build_array(
            jsonb_build_object(
                'employer',
                jsonb_build_object(
                    'company', 'Acme Corp',
                    'employer_id', :'employer1ID'::text,
                    'members',
                    jsonb_build_array(
                        jsonb_build_object(
                            'foundation', 'cncf',
                            'level', 'platinum',
                            'logo_url', 'https://example.com/member-cncf.svg',
                            'member_id', :'memberCNCFID'::text,
                            'name', 'Acme Foundation'
                        )
                    )
                ),
                'job_id', :'job1ID'::text,
                'kind', 'full-time',
                'location',
                jsonb_build_object(
                    'city', 'San Francisco',
                    'country', 'United States',
                    'location_id', :'location1ID'::text,
                    'state', 'CA'
                ),
                'open_source', 80,
                'projects',
                jsonb_build_array(
                    jsonb_build_object(
                        'foundation', 'cncf',
                        'logo_url', 'https://example.com/project-cncf.svg',
                        'maturity', 'graduated',
                        'name', 'Kubernetes',
                        'project_id', :'projectCNCFID'::text
                    )
                ),
                'published_at', to_jsonb('2026-01-03 10:00:00+00'::timestamptz),
                'salary', 150000,
                'salary_currency', 'USD',
                'salary_max', 200000,
                'salary_min', 100000,
                'salary_period', 'year',
                'seniority', 'senior',
                'skills', jsonb_build_array('rust', 'kubernetes'),
                'title', 'Kubernetes Platform Engineer',
                'updated_at', to_jsonb('2026-01-03 11:00:00+00'::timestamptz),
                'upstream_commitment', 70,
                'workplace', 'remote'
            ),
            jsonb_build_object(
                'employer',
                jsonb_build_object(
                    'company', 'Beta Inc',
                    'employer_id', :'employer2ID'::text,
                    'members',
                    jsonb_build_array(
                        jsonb_build_object(
                            'foundation', 'lf',
                            'level', 'gold',
                            'logo_url', 'https://example.com/member-lf.svg',
                            'member_id', :'memberLFID'::text,
                            'name', 'Beta Foundation'
                        )
                    )
                ),
                'job_id', :'job2ID'::text,
                'kind', 'full-time',
                'location',
                jsonb_build_object(
                    'city', 'New York',
                    'country', 'United States',
                    'location_id', :'location2ID'::text,
                    'state', 'NY'
                ),
                'open_source', 20,
                'projects',
                jsonb_build_array(
                    jsonb_build_object(
                        'foundation', 'lf',
                        'logo_url', 'https://example.com/project-lf.svg',
                        'maturity', 'incubating',
                        'name', 'OpenTofu',
                        'project_id', :'projectLFID'::text
                    )
                ),
                'published_at', to_jsonb('2026-01-02 10:00:00+00'::timestamptz),
                'salary', 70000,
                'salary_currency', 'USD',
                'salary_max', 100000,
                'salary_min', 60000,
                'salary_period', 'year',
                'seniority', 'junior',
                'skills', jsonb_build_array('javascript', 'react'),
                'title', 'Frontend Engineer',
                'updated_at', to_jsonb('2026-01-02 11:00:00+00'::timestamptz),
                'upstream_commitment', 30,
                'workplace', 'hybrid'
            )
        ),
        'total', 2
    ),
    'Should return full payload for published jobs by default'
);

-- Should filter by foundation
select is(
    (
        select total
        from (
            select (jobboard_search_jobs('{"foundation":"cncf"}'::jsonb)->>'total')::bigint as total
        ) t
    ),
    1::bigint,
    'Should filter by foundation'
);

-- Should filter by full text query with prefix matching
select is(
    (
        select total
        from (
            select (jobboard_search_jobs('{"ts_query":"kuber"}'::jsonb)->>'total')::bigint as total
        ) t
    ),
    1::bigint,
    'Should filter by full text query with prefix matching'
);

-- Should filter by employer membership foundation
select is(
    (
        select total
        from (
            select (jobboard_search_jobs('{"membership":"lf"}'::jsonb)->>'total')::bigint as total
        ) t
    ),
    1::bigint,
    'Should filter by employer membership foundation'
);

-- Should sort by salary when requested
select is(
    (
        select (jobs::jsonb->0->>'job_id')::uuid
        from (select jobboard_search_jobs('{"sort":"salary"}'::jsonb)->'jobs' as jobs) t
    ),
    :'job1ID'::uuid,
    'Should sort by salary when requested'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
