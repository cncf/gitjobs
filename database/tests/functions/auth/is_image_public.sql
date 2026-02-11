-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set employerPublicID '00000000-0000-0000-0000-000000000101'
\set employerPrivateID '00000000-0000-0000-0000-000000000102'
\set imagePrivateID '00000000-0000-0000-0000-000000000201'
\set imagePublicID '00000000-0000-0000-0000-000000000202'
\set jobDraftID '00000000-0000-0000-0000-000000000301'
\set jobPublishedID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into image (image_id) values
    (:'imagePrivateID'),
    (:'imagePublicID');

insert into employer (company, description, employer_id, logo_id) values
    ('Private Corp', 'Private employer', :'employerPrivateID', :'imagePrivateID'),
    ('Public Corp', 'Public employer', :'employerPublicID', :'imagePublicID');

insert into job (
    description,
    employer_id,
    first_published_at,
    job_id,
    kind,
    status,
    title,
    workplace
) values
    (
        'Draft role',
        :'employerPrivateID',
        null,
        :'jobDraftID',
        'full-time',
        'draft',
        'Draft Job',
        'remote'
    ),
    (
        'Published role',
        :'employerPublicID',
        '2026-01-01 10:00:00+00',
        :'jobPublishedID',
        'full-time',
        'published',
        'Published Job',
        'remote'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return true for employer logos with published jobs
select is(
    auth_is_image_public(:'imagePublicID'::uuid),
    true,
    'Should return true for employer logos with published jobs'
);

-- Should return false for employer logos without published jobs
select is(
    auth_is_image_public(:'imagePrivateID'::uuid),
    false,
    'Should return false for employer logos without published jobs'
);

-- Should return false for unknown images
select is(
    auth_is_image_public('99999999-9999-9999-9999-999999999999'::uuid),
    false,
    'Should return false for unknown images'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
