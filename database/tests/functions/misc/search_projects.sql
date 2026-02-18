-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(3);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set project1ID '00000000-0000-0000-0000-000000000501'
\set project2ID '00000000-0000-0000-0000-000000000502'
\set project3ID '00000000-0000-0000-0000-000000000503'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into foundation (name) values ('lf');

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/kube.svg', 'graduated', 'Kubernetes', :'project1ID'),
    ('cncf', 'https://example.com/envoy.svg', 'graduated', 'Envoy', :'project2ID'),
    ('lf', 'https://example.com/opentofu.svg', 'incubating', 'OpenTofu', :'project3ID');

insert into project (foundation, logo_url, maturity, name)
select
    'cncf',
    'https://example.com/project-' || i || '.svg',
    'sandbox',
    'Generated Project ' || i
from generate_series(1, 25) as i;

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return only projects from the selected foundation
select is(
    search_projects('lf', '')::jsonb,
    '[
        {
            "foundation": "lf",
            "logo_url": "https://example.com/opentofu.svg",
            "maturity": "incubating",
            "name": "OpenTofu",
            "project_id": "00000000-0000-0000-0000-000000000503"
        }
    ]'::jsonb,
    'Should return only projects from the selected foundation'
);

-- Should filter projects using case-insensitive partial matching
select is(
    search_projects('cncf', 'kuber')::jsonb,
    '[
        {
            "foundation": "cncf",
            "logo_url": "https://example.com/kube.svg",
            "maturity": "graduated",
            "name": "Kubernetes",
            "project_id": "00000000-0000-0000-0000-000000000501"
        }
    ]'::jsonb,
    'Should filter projects using case-insensitive partial matching'
);

-- Should cap returned projects at 20
select is(
    (
        select jsonb_array_length(search_projects('cncf', 'project')::jsonb)
    ),
    20,
    'Should cap returned projects at 20'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
