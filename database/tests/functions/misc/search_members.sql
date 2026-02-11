-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set member1ID '00000000-0000-0000-0000-000000000401'
\set member2ID '00000000-0000-0000-0000-000000000402'
\set member3ID '00000000-0000-0000-0000-000000000403'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

insert into member (foundation, level, logo_url, member_id, name) values
    ('cncf', 'platinum', 'https://example.com/acme.svg', :'member1ID', 'Acme'),
    ('cncf', 'gold', 'https://example.com/byte.svg', :'member2ID', 'Byte Corp'),
    ('lf', 'silver', 'https://example.com/other.svg', :'member3ID', 'Other Org');

insert into member (foundation, level, logo_url, name)
select
    'cncf',
    'silver',
    'https://example.com/member-' || i || '.svg',
    'Generated Member ' || i
from generate_series(1, 25) as i;

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return only members from the selected foundation
select is(
    misc_search_members('lf', '')::jsonb,
    '[
        {
            "foundation": "lf",
            "level": "silver",
            "logo_url": "https://example.com/other.svg",
            "member_id": "00000000-0000-0000-0000-000000000403",
            "name": "Other Org"
        }
    ]'::jsonb,
    'Should return only members from the selected foundation'
);

-- Should filter members using case-insensitive partial matching
select is(
    misc_search_members('cncf', 'acm')::jsonb,
    '[
        {
            "foundation": "cncf",
            "level": "platinum",
            "logo_url": "https://example.com/acme.svg",
            "member_id": "00000000-0000-0000-0000-000000000401",
            "name": "Acme"
        }
    ]'::jsonb,
    'Should filter members using case-insensitive partial matching'
);

-- Should cap returned members at 20
select is(
    (
        select jsonb_array_length(misc_search_members('cncf', 'member')::jsonb)
    ),
    20,
    'Should cap returned members at 20'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
