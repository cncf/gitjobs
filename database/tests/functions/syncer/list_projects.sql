-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

insert into project (foundation, logo_url, maturity, name) values
    ('cncf', 'https://example.com/kube.svg', 'graduated', 'Kubernetes'),
    ('cncf', 'https://example.com/envoy.svg', 'incubating', 'Envoy'),
    ('lf', 'https://example.com/opentofu.svg', 'incubating', 'OpenTofu');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full projects payload for the requested foundation
select is(
    list_projects('cncf')::jsonb,
    '[
        {
            "foundation": "cncf",
            "logo_url": "https://example.com/envoy.svg",
            "maturity": "incubating",
            "name": "Envoy"
        },
        {
            "foundation": "cncf",
            "logo_url": "https://example.com/kube.svg",
            "maturity": "graduated",
            "name": "Kubernetes"
        }
    ]'::jsonb,
    'Should return full projects payload for the requested foundation'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
