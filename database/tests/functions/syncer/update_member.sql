-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into member (foundation, level, logo_url, name) values
    ('cncf', 'silver', 'https://example.com/acme-old.svg', 'Acme');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update member level and logo URL
select syncer_update_member('cncf', 'Acme', 'platinum', 'https://example.com/acme-new.svg');

select ok(
    exists (
        select 1
        from member
        where foundation = 'cncf'
        and name = 'Acme'
        and level = 'platinum'
        and logo_url = 'https://example.com/acme-new.svg'
    ),
    'Should update member level and logo URL'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
