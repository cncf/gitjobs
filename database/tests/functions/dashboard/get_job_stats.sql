-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000201'
\set otherJobID '00000000-0000-0000-0000-000000000202'
\set unknownJobID '00000000-0000-0000-0000-999999999999'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Employer and jobs
insert into employer (employer_id, company, description) values
    (:'employerID', 'Stats Employer', 'Employer for get_job_stats tests');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    title,
    workplace,
    description
) values
    (:'jobID', :'employerID', 'full-time', 'published', 'Primary Job', 'remote', 'Primary job'),
    (:'otherJobID', :'employerID', 'full-time', 'published', 'Other Job', 'remote', 'Other job');

-- Views and search appearances
insert into job_views (day, job_id, total) values
    ((current_date - interval '40 days')::date, :'jobID', 100),
    ((current_date - interval '20 days')::date, :'jobID', 2),
    ((current_date - interval '5 days')::date, :'jobID', 3),
    ((current_date - interval '3 days')::date, :'otherJobID', 50);

insert into search_appearances (day, job_id, total) values
    ((current_date - interval '40 days')::date, :'jobID', 30),
    ((current_date - interval '20 days')::date, :'jobID', 7),
    ((current_date - interval '5 days')::date, :'jobID', 1),
    ((current_date - interval '3 days')::date, :'otherJobID', 60);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return aggregated stats for the selected job and last month only
select is(
    get_job_stats(:'jobID'::uuid)::jsonb,
    (
        select jsonb_build_object(
            'search_appearances_daily', jsonb_build_array(
                jsonb_build_array(
                    (extract(epoch from (current_date - interval '20 days')::date) * 1000)::bigint,
                    7
                ),
                jsonb_build_array(
                    (extract(epoch from (current_date - interval '5 days')::date) * 1000)::bigint,
                    1
                )
            ),
            'search_appearances_total_last_month', 8,
            'views_daily', jsonb_build_array(
                jsonb_build_array(
                    (extract(epoch from (current_date - interval '20 days')::date) * 1000)::bigint,
                    2
                ),
                jsonb_build_array(
                    (extract(epoch from (current_date - interval '5 days')::date) * 1000)::bigint,
                    3
                )
            ),
            'views_total_last_month', 5
        )
    ),
    'Should return aggregated stats for the selected job and last month only'
);

-- Should return empty stats when the job has no counters
select is(
    get_job_stats(:'unknownJobID'::uuid)::jsonb,
    '{
        "search_appearances_daily": [],
        "search_appearances_total_last_month": 0,
        "views_daily": [],
        "views_total_last_month": 0
    }'::jsonb,
    'Should return empty stats when the job has no counters'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
