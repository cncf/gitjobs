-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into project (foundation, logo_url, maturity, name) values
    ('cncf', 'https://example.com/kube-old.svg', 'incubating', 'Kubernetes');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should update project maturity and logo URL
select update_project('cncf', 'Kubernetes', 'graduated', 'https://example.com/kube-new.svg');

select ok(
    exists (
        select 1
        from project
        where foundation = 'cncf'
        and name = 'Kubernetes'
        and maturity = 'graduated'
        and logo_url = 'https://example.com/kube-new.svg'
    ),
    'Should update project maturity and logo URL'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
