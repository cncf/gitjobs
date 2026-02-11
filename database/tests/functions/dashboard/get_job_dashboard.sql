-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set certificationID '00000000-0000-0000-0000-000000000401'
\set deletedJobID '00000000-0000-0000-0000-000000000302'
\set employerID '00000000-0000-0000-0000-000000000101'
\set jobID '00000000-0000-0000-0000-000000000301'
\set locationID '00000000-0000-0000-0000-000000000201'
\set projectID '00000000-0000-0000-0000-000000000501'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into location (city, country, location_id, state) values
    ('San Francisco', 'United States', :'locationID', 'CA');

insert into employer (company, description, employer_id) values
    ('Acme Corp', 'Employer for get_job_dashboard tests', :'employerID');

insert into job (
    job_id,
    employer_id,
    kind,
    open_source,
    status,
    location_id,
    upstream_commitment,
    workplace,
    title,
    description
) values
    (
        :'jobID',
        :'employerID',
        'full-time',
        0,
        'published',
        :'locationID',
        0,
        'remote',
        'Platform Engineer',
        'Build platform services'
    ),
    (
        :'deletedJobID',
        :'employerID',
        'full-time',
        0,
        'deleted',
        :'locationID',
        0,
        'remote',
        'Deleted Role',
        'Deleted description'
    );

insert into project (foundation, logo_url, maturity, name, project_id) values
    ('cncf', 'https://example.com/kubernetes.svg', 'graduated', 'Kubernetes', :'projectID');

insert into certification (
    certification_id,
    description,
    logo_url,
    name,
    provider,
    short_name,
    url
) values
    (
        :'certificationID',
        'CKA certification',
        'https://example.com/cka.svg',
        'Certified Kubernetes Administrator Test',
        'CNCF',
        'CKA-TEST',
        'https://example.com/cka'
    );

insert into job_project (job_id, project_id) values
    (:'jobID', :'projectID');

insert into job_certification (job_id, certification_id) values
    (:'jobID', :'certificationID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload for non-deleted jobs
select is(
    (
        select dashboard_employer_get_job_dashboard(:'jobID'::uuid)::jsonb - 'updated_at'
    ),
    '{
        "certifications": [
            {
                "certification_id": "00000000-0000-0000-0000-000000000401",
                "description": "CKA certification",
                "logo_url": "https://example.com/cka.svg",
                "name": "Certified Kubernetes Administrator Test",
                "provider": "CNCF",
                "short_name": "CKA-TEST",
                "url": "https://example.com/cka"
            }
        ],
        "description": "Build platform services",
        "job_id": "00000000-0000-0000-0000-000000000301",
        "kind": "full-time",
        "location": {
            "city": "San Francisco",
            "country": "United States",
            "location_id": "00000000-0000-0000-0000-000000000201",
            "state": "CA"
        },
        "open_source": 0,
        "projects": [
            {
                "foundation": "cncf",
                "logo_url": "https://example.com/kubernetes.svg",
                "maturity": "graduated",
                "name": "Kubernetes",
                "project_id": "00000000-0000-0000-0000-000000000501"
            }
        ],
        "status": "published",
        "title": "Platform Engineer",
        "upstream_commitment": 0,
        "workplace": "remote"
    }'::jsonb,
    'Should return full payload for non-deleted jobs'
);

-- Should return null for deleted jobs
select is(
    dashboard_employer_get_job_dashboard(:'deletedJobID'::uuid)::jsonb,
    null::jsonb,
    'Should return null for deleted jobs'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
