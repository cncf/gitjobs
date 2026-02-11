-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set location1ID '00000000-0000-0000-0000-000000000101'
\set location2ID '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'location1ID', 'CA'),
    ('San Jose', 'United States', :'location2ID', 'CA');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full json payload with matching locations
select is(
    misc_search_locations_json('san')::jsonb,
    jsonb_build_array(
        jsonb_build_object(
            'city', 'San Francisco',
            'country', 'United States',
            'location_id', :'location1ID'::text,
            'state', 'CA'
        ),
        jsonb_build_object(
            'city', 'San Jose',
            'country', 'United States',
            'location_id', :'location2ID'::text,
            'state', 'CA'
        )
    ),
    'Should return full json payload with matching locations'
);

-- Should return an empty array when there are no matches
select is(
    misc_search_locations_json('tokyo')::jsonb,
    '[]'::jsonb,
    'Should return an empty array when there are no matches'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
