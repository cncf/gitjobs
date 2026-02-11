-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set imageWithPngAndSvgID '00000000-0000-0000-0000-000000000101'
\set imageWithoutVersionsID '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Images
insert into image (image_id, created_by) values
    (:'imageWithPngAndSvgID', null),
    (:'imageWithoutVersionsID', null);

-- Image versions
insert into image_version (data, image_id, version) values
    (decode('0102', 'hex'), :'imageWithPngAndSvgID', 'small'),
    (decode('aa55', 'hex'), :'imageWithPngAndSvgID', 'svg');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return requested PNG version when it exists
select is(
    (
        select format || ':' || encode(data, 'hex')
        from img_get_image_version(:'imageWithPngAndSvgID'::uuid, 'small')
    ),
    'png:0102',
    'Should return requested PNG version when it exists'
);

-- Should fallback to SVG when requested version does not exist
select is(
    (
        select format || ':' || encode(data, 'hex')
        from img_get_image_version(:'imageWithPngAndSvgID'::uuid, 'large')
    ),
    'svg:aa55',
    'Should fallback to SVG when requested version does not exist'
);

-- Should return no rows when neither requested version nor SVG exists
select is(
    (
        select count(*)
        from img_get_image_version(:'imageWithoutVersionsID'::uuid, 'large')
    ),
    0::bigint,
    'Should return no rows when neither requested version nor SVG exists'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
