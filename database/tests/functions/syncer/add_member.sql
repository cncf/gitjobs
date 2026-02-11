-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should add a member for the selected foundation
select add_member('cncf', 'Acme', 'platinum', 'https://example.com/acme.svg');

select ok(
    exists (
        select 1
        from member
        where foundation = 'cncf'
        and level = 'platinum'
        and logo_url = 'https://example.com/acme.svg'
        and name = 'Acme'
    ),
    'Should add a member for the selected foundation'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
