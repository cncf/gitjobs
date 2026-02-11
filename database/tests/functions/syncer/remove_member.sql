-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into member (foundation, level, logo_url, name) values
    ('cncf', 'platinum', 'https://example.com/acme.svg', 'Acme');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should remove the selected member
select syncer_remove_member('cncf', 'Acme');

select is(
    (
        select count(*)
        from member
        where foundation = 'cncf'
        and name = 'Acme'
    ),
    0::bigint,
    'Should remove the selected member'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
