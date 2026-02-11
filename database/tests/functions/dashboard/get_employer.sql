-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set locationID '00000000-0000-0000-0000-000000000201'
\set memberCNCFID '00000000-0000-0000-0000-000000000301'
\set memberLFID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'locationID', 'CA');

insert into employer (
    company,
    description,
    employer_id,
    location_id,
    logo_id,
    public,
    website_url
) values
    (
        'Acme Corp',
        'Employer for get_employer tests',
        :'employerID',
        :'locationID',
        null,
        true,
        'https://acme.example.com'
    );

insert into member (foundation, level, logo_url, member_id, name) values
    ('lf', 'gold', 'https://example.com/lf.svg', :'memberLFID', 'LF Member'),
    ('cncf', 'platinum', 'https://example.com/cncf.svg', :'memberCNCFID', 'CNCF Member');

insert into employer_member (employer_id, member_id) values
    (:'employerID', :'memberLFID'),
    (:'employerID', :'memberCNCFID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full employer payload
select is(
    dashboard_employer_get_employer(:'employerID'::uuid)::jsonb,
    '{
        "company": "Acme Corp",
        "description": "Employer for get_employer tests",
        "location": {
            "city": "San Francisco",
            "country": "United States",
            "location_id": "00000000-0000-0000-0000-000000000201",
            "state": "CA"
        },
        "members": [
            {
                "foundation": "cncf",
                "level": "platinum",
                "logo_url": "https://example.com/cncf.svg",
                "member_id": "00000000-0000-0000-0000-000000000301",
                "name": "CNCF Member"
            },
            {
                "foundation": "lf",
                "level": "gold",
                "logo_url": "https://example.com/lf.svg",
                "member_id": "00000000-0000-0000-0000-000000000302",
                "name": "LF Member"
            }
        ],
        "public": true,
        "website_url": "https://acme.example.com"
    }'::jsonb,
    'Should return full employer payload'
);

-- Should return null for unknown employers
select is(
    dashboard_employer_get_employer('99999999-9999-9999-9999-999999999999'::uuid)::jsonb,
    null::jsonb,
    'Should return null for unknown employers'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
