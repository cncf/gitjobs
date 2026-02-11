-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

insert into member (foundation, level, logo_url, name) values
    ('cncf', 'platinum', 'https://example.com/acme.svg', 'Acme'),
    ('cncf', 'gold', 'https://example.com/beta.svg', 'Beta'),
    ('lf', 'silver', 'https://example.com/other.svg', 'Other');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full members payload for the requested foundation
select is(
    syncer_list_members('cncf')::jsonb,
    '[
        {
            "foundation": "cncf",
            "level": "platinum",
            "logo_url": "https://example.com/acme.svg",
            "name": "Acme"
        },
        {
            "foundation": "cncf",
            "level": "gold",
            "logo_url": "https://example.com/beta.svg",
            "name": "Beta"
        }
    ]'::jsonb,
    'Should return full members payload for the requested foundation'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
