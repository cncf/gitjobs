-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set locationID '00000000-0000-0000-0000-000000000101'
\set logoID '00000000-0000-0000-0000-000000000201'
\set member1ID '00000000-0000-0000-0000-000000000301'
\set member2ID '00000000-0000-0000-0000-000000000302'
\set userID '00000000-0000-0000-0000-000000000401'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'owner@example.com', 'Owner', :'userID', 'owner');

insert into image (created_by, image_id) values
    (:'userID', :'logoID');

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'locationID', 'CA');

insert into member (foundation, level, logo_url, member_id, name) values
    ('cncf', 'gold', 'https://example.com/member1.svg', :'member1ID', 'Alpha Member'),
    ('cncf', 'silver', 'https://example.com/member2.svg', :'member2ID', 'Beta Member');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should create employer and return a generated id
select ok(
    (
        select add_employer(
            :'userID'::uuid,
            jsonb_build_object(
                'company', 'Acme Corp',
                'description', 'Employer profile',
                'public', true,
                'location', jsonb_build_object('location_id', :'locationID'::text),
                'logo_id', :'logoID'::text,
                'members', jsonb_build_array(
                    jsonb_build_object('member_id', :'member1ID'::text),
                    jsonb_build_object('member_id', :'member2ID'::text)
                ),
                'website_url', 'https://acme.example'
            )
        ) is not null
    ),
    'Should create employer and return a generated id'
);

-- Should persist all employer fields from payload
select is(
    (
        select jsonb_build_object(
            'company', e.company,
            'description', e.description,
            'location_id', e.location_id::text,
            'logo_id', e.logo_id::text,
            'public', e.public,
            'website_url', e.website_url
        )
        from employer e
        where e.company = 'Acme Corp'
    ),
    jsonb_build_object(
        'company', 'Acme Corp',
        'description', 'Employer profile',
        'location_id', :'locationID'::text,
        'logo_id', :'logoID'::text,
        'public', true,
        'website_url', 'https://acme.example'
    ),
    'Should persist all employer fields from payload'
);

-- Should persist all employer memberships from payload
select is(
    (
        select coalesce(jsonb_agg(em.member_id::text order by em.member_id::text), '[]'::jsonb)
        from employer_member em
        join employer e on em.employer_id = e.employer_id
        where e.company = 'Acme Corp'
    ),
    jsonb_build_array(:'member1ID'::text, :'member2ID'::text),
    'Should persist all employer memberships from payload'
);

-- Should add the creator as an approved team member
select ok(
    exists (
        select 1
        from employer_team et
        join employer e on et.employer_id = e.employer_id
        where e.company = 'Acme Corp'
        and et.user_id = :'userID'::uuid
        and et.approved = true
    ),
    'Should add the creator as an approved team member'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
