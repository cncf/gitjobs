-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values
    ('test-foundation-a'),
    ('test-foundation-b');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return foundations in a json payload sorted by name
select is(
    (
        select coalesce(jsonb_agg(foundation order by foundation->>'name'), '[]'::jsonb)
        from jsonb_array_elements(
            get_jobs_filters_options()::jsonb->'foundations'
        ) foundation
        where foundation->>'name' like 'test-foundation-%'
    ),
    jsonb_build_array(
        jsonb_build_object('name', 'test-foundation-a'),
        jsonb_build_object('name', 'test-foundation-b')
    ),
    'Should return foundations in a json payload sorted by name'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
