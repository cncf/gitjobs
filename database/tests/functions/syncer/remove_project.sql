-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into project (foundation, logo_url, maturity, name) values
    ('cncf', 'https://example.com/kube.svg', 'graduated', 'Kubernetes');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should remove the selected project
select remove_project('cncf', 'Kubernetes');

select is(
    (
        select count(*)
        from project
        where foundation = 'cncf'
        and name = 'Kubernetes'
    ),
    0::bigint,
    'Should remove the selected project'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
