-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerID '00000000-0000-0000-0000-000000000101'
\set location1ID '00000000-0000-0000-0000-000000000201'
\set location2ID '00000000-0000-0000-0000-000000000202'
\set logo1ID '00000000-0000-0000-0000-000000000301'
\set logo2ID '00000000-0000-0000-0000-000000000302'
\set member1ID '00000000-0000-0000-0000-000000000401'
\set member2ID '00000000-0000-0000-0000-000000000402'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into image (image_id) values
    (:'logo1ID'),
    (:'logo2ID');

insert into location (city, country, location_id, state) values
    ('Berlin', 'Germany', :'location1ID', null),
    ('Paris', 'France', :'location2ID', null);

insert into member (foundation, level, logo_url, member_id, name) values
    ('cncf', 'gold', 'https://example.com/member1.svg', :'member1ID', 'Alpha Member'),
    ('cncf', 'silver', 'https://example.com/member2.svg', :'member2ID', 'Beta Member');

insert into employer (company, description, employer_id, location_id, logo_id, public, website_url) values
    (
        'Old Corp',
        'Old description',
        :'employerID',
        :'location1ID',
        :'logo1ID',
        false,
        'https://old.example'
    );

insert into employer_member (employer_id, member_id) values
    (:'employerID', :'member1ID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update all employer fields from payload
select dashboard_employer_update_employer(
    :'employerID'::uuid,
    jsonb_build_object(
        'company', 'New Corp',
        'description', 'New description',
        'public', true,
        'location', jsonb_build_object('location_id', :'location2ID'::text),
        'logo_id', :'logo2ID'::text,
        'members', jsonb_build_array(
            jsonb_build_object('member_id', :'member2ID'::text)
        ),
        'website_url', 'https://new.example'
    )
);

select ok(
    exists (
        select 1
        from employer e
        where e.employer_id = :'employerID'::uuid
        and e.company = 'New Corp'
        and e.description = 'New description'
        and e.public = true
        and e.location_id = :'location2ID'::uuid
        and e.logo_id = :'logo2ID'::uuid
        and e.website_url = 'https://new.example'
        and e.updated_at is not null
    ),
    'Should update all employer fields from payload'
);

-- Should replace memberships with payload content
select is(
    (
        select coalesce(jsonb_agg(member_id::text order by member_id::text), '[]'::jsonb)
        from employer_member
        where employer_id = :'employerID'::uuid
    ),
    jsonb_build_array(:'member2ID'::text),
    'Should replace memberships with payload content'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
