-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should add a project for the selected foundation
select add_project('cncf', 'Kubernetes', 'graduated', 'https://example.com/kube.svg');

select ok(
    exists (
        select 1
        from project
        where foundation = 'cncf'
        and logo_url = 'https://example.com/kube.svg'
        and maturity = 'graduated'
        and name = 'Kubernetes'
    ),
    'Should add a project for the selected foundation'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
