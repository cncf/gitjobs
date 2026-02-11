-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

update foundation
set landscape_url = 'https://landscape.cncf.io'
where name = 'cncf';

update foundation
set landscape_url = 'https://landscape.lf.example.com'
where name = 'lf';

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full foundations payload with configured landscape URLs
select is(
    syncer_list_foundations()::jsonb,
    '[
        {
            "landscape_url": "https://landscape.cncf.io",
            "name": "cncf"
        },
        {
            "landscape_url": "https://landscape.lf.example.com",
            "name": "lf"
        }
    ]'::jsonb,
    'Should return full foundations payload with configured landscape URLs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
