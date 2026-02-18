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
values ('Appearances Employer', 'Employer for update_search_appearances tests', :'employerID');

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
select update_search_appearances(
    42,
    jsonb_build_array(
        jsonb_build_array(:'publishedJobID'::text, current_date::text, 6),
        jsonb_build_array(:'draftJobID'::text, current_date::text, 4),
        jsonb_build_array(:'unknownJobID'::text, current_date::text, 9)
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
        from search_appearances
    ),
    jsonb_build_array(
        jsonb_build_object(
            'day', current_date::text,
            'job_id', :'publishedJobID',
            'total', 6
        )
    ),
    'Should insert counters only for published jobs'
);

-- Should ignore counters for non-published or unknown jobs
select is(
    (select count(*) from search_appearances),
    1::bigint,
    'Should ignore counters for non-published or unknown jobs'
);

-- Should increment existing counters on conflict
select update_search_appearances(
    43,
    jsonb_build_array(
        jsonb_build_array(:'publishedJobID'::text, current_date::text, 5)
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
        from search_appearances
    ),
    jsonb_build_array(
        jsonb_build_object(
            'day', current_date::text,
            'job_id', :'publishedJobID',
            'total', 11
        )
    ),
    'Should increment existing counters on conflict'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
