-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set parisID '00000000-0000-0000-0000-000000000103'
\set sanFranciscoID '00000000-0000-0000-0000-000000000101'
\set sanJoseID '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Named locations
insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'sanFranciscoID', 'CA'),
    ('San Jose', 'United States', :'sanJoseID', 'CA'),
    ('Paris', 'France', :'parisID', null);

-- Additional locations to validate result limit
insert into location (city, country, location_id)
select
    'Matchville ' || g,
    'United States',
    gen_random_uuid()
from generate_series(1, 25) as g;

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should support prefix matching in full text search
select is(
    (
        select row_to_json(location_result)::jsonb
        from misc_search_locations('san fra') as location_result
        limit 1
    ),
    jsonb_build_object(
        'city', 'San Francisco',
        'country', 'United States',
        'location_id', :'sanFranciscoID',
        'state', 'CA'
    ),
    'Should support prefix matching in full text search'
);

-- Should cap the number of returned matches at 20
select is(
    (select count(*) from misc_search_locations('matchv')),
    20::bigint,
    'Should cap the number of returned matches at 20'
);

-- Should return no matches for unrelated queries
select is(
    (select count(*) from misc_search_locations('totally unrelated query')),
    0::bigint,
    'Should return no matches for unrelated queries'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
