-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employer1ID '00000000-0000-0000-0000-000000000101'
\set employer2ID '00000000-0000-0000-0000-000000000102'
\set job1ID '00000000-0000-0000-0000-000000000301'
\set job2ID '00000000-0000-0000-0000-000000000302'
\set job3ID '00000000-0000-0000-0000-000000000303'
\set projectCNCFID '00000000-0000-0000-0000-000000000201'
\set projectLFID '00000000-0000-0000-0000-000000000202'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Foundations and projects
insert into foundation (name) values ('lf');

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/project-cncf.svg', 'graduated', 'Kubernetes', :'projectCNCFID'),
    ('lf', 'https://example.com/project-lf.svg', 'incubating', 'OpenTofu', :'projectLFID');

-- Employers and jobs
insert into employer (company, description, employer_id) values
    ('Stats One', 'Employer one for get_stats tests', :'employer1ID'),
    ('Stats Two', 'Employer two for get_stats tests', :'employer2ID');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    first_published_at,
    title,
    workplace,
    description
) values
    (
        :'job1ID',
        :'employer1ID',
        'full-time',
        'published',
        current_timestamp - interval '10 days',
        'Backend Engineer',
        'remote',
        'Job one'
    ),
    (
        :'job2ID',
        :'employer1ID',
        'full-time',
        'published',
        current_timestamp - interval '20 days',
        'Platform Engineer',
        'remote',
        'Job two'
    ),
    (
        :'job3ID',
        :'employer2ID',
        'full-time',
        'published',
        current_timestamp - interval '40 days',
        'Frontend Engineer',
        'hybrid',
        'Job three'
    );

insert into job_project (job_id, project_id) values
    (:'job1ID', :'projectCNCFID'),
    (:'job2ID', :'projectCNCFID'),
    (:'job3ID', :'projectLFID');

-- Views
insert into job_views (day, job_id, total) values
    ((current_date - interval '10 days')::date, :'job1ID', 3),
    ((current_date - interval '10 days')::date, :'job2ID', 2),
    ((current_date - interval '70 days')::date, :'job3ID', 7),
    ((current_date - interval '3 years')::date, :'job1ID', 100);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should aggregate published jobs per foundation
select is(
    (
        select stats->'jobs'->'published_per_foundation'
        from (
            select get_stats()::jsonb as stats
        ) t
    ),
    '[["cncf", 2], ["lf", 1]]'::jsonb,
    'Should aggregate published jobs per foundation'
);

-- Should include only last-month data in views_daily
select is(
    (
        select stats->'jobs'->'views_daily'
        from (
            select get_stats()::jsonb as stats
        ) t
    ),
    (
        select jsonb_build_array(
            jsonb_build_array(
                (extract(epoch from (current_date - interval '10 days')::date) * 1000)::bigint,
                5
            )
        )
    ),
    'Should include only last-month data in views_daily'
);

-- Should include only last-two-years data in views_monthly
select is(
    (
        select stats->'jobs'->'views_monthly'
        from (
            select get_stats()::jsonb as stats
        ) t
    ),
    (
        select jsonb_build_array(
            jsonb_build_array(
                (extract(epoch from date_trunc('month', (current_date - interval '70 days')::date)) * 1000)::bigint,
                7
            ),
            jsonb_build_array(
                (extract(epoch from date_trunc('month', (current_date - interval '10 days')::date)) * 1000)::bigint,
                5
            )
        )
    ),
    'Should include only last-two-years data in views_monthly'
);

-- Should return ordered timeline timestamps
select ok(
    (
        with stats as (
            select get_stats()::jsonb as s
        )
        select
            (s->>'ts_now')::bigint > (s->>'ts_one_month_ago')::bigint
            and (s->>'ts_one_month_ago')::bigint > (s->>'ts_two_years_ago')::bigint
        from stats
    ),
    'Should return ordered timeline timestamps'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
