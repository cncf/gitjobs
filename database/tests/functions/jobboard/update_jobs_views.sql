-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set draftJobID '00000000-0000-0000-0000-000000000302'
\set employerID '00000000-0000-0000-0000-000000000101'
\set publishedJobID '00000000-0000-0000-0000-000000000301'
\set unknownJobID '00000000-0000-0000-0000-999999999999'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Employer and jobs
insert into employer (company, description, employer_id)
values ('Views Employer', 'Employer for update_jobs_views tests', :'employerID');

insert into job (
    job_id,
    employer_id,
    kind,
    status,
    title,
    workplace,
    description
) values
    (:'publishedJobID', :'employerID', 'full-time', 'published', 'Published Job', 'remote', 'Published role'),
    (:'draftJobID', :'employerID', 'full-time', 'draft', 'Draft Job', 'remote', 'Draft role');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should insert counters only for published jobs
select jobboard_update_jobs_views(
    42,
    jsonb_build_array(
        jsonb_build_array(:'publishedJobID'::text, current_date::text, 3),
        jsonb_build_array(:'draftJobID'::text, current_date::text, 5),
        jsonb_build_array(:'unknownJobID'::text, current_date::text, 8)
    )
);

select is(
    (
        select jsonb_agg(
            jsonb_build_object(
                'day', day::text,
                'job_id', job_id::text,
                'total', total
            )
            order by day, job_id
        )
        from job_views
    ),
    jsonb_build_array(
        jsonb_build_object(
            'day', current_date::text,
            'job_id', :'publishedJobID',
            'total', 3
        )
    ),
    'Should insert counters only for published jobs'
);

-- Should ignore counters for non-published or unknown jobs
select is(
    (select count(*) from job_views),
    1::bigint,
    'Should ignore counters for non-published or unknown jobs'
);

-- Should increment existing counters on conflict
select jobboard_update_jobs_views(
    43,
    jsonb_build_array(
        jsonb_build_array(:'publishedJobID'::text, current_date::text, 4)
    )
);

select is(
    (
        select jsonb_agg(
            jsonb_build_object(
                'day', day::text,
                'job_id', job_id::text,
                'total', total
            )
            order by day, job_id
        )
        from job_views
    ),
    jsonb_build_array(
        jsonb_build_object(
            'day', current_date::text,
            'job_id', :'publishedJobID',
            'total', 7
        )
    ),
    'Should increment existing counters on conflict'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
