-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set certAID '00000000-0000-0000-0000-000000000301'
\set certZID '00000000-0000-0000-0000-000000000302'

-- ============================================================================
-- SEED DATA
-- ============================================================================

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
        :'certZID',
        'Z certification',
        'https://example.com/z.svg',
        'Z Cert',
        'test-provider',
        'Z-CERT',
        'https://example.com/z'
    ),
    (
        :'certAID',
        'A certification',
        'https://example.com/a.svg',
        'A Cert',
        'test-provider',
        'A-CERT',
        'https://example.com/a'
    );

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full certifications payload sorted by name
select is(
    (
        select coalesce(jsonb_agg(certification order by certification->>'name'), '[]'::jsonb)
        from jsonb_array_elements(
            list_certifications()::jsonb
        ) certification
        where certification->>'provider' = 'test-provider'
    ),
    '[
        {
            "certification_id": "00000000-0000-0000-0000-000000000301",
            "description": "A certification",
            "logo_url": "https://example.com/a.svg",
            "name": "A Cert",
            "provider": "test-provider",
            "short_name": "A-CERT",
            "url": "https://example.com/a"
        },
        {
            "certification_id": "00000000-0000-0000-0000-000000000302",
            "description": "Z certification",
            "logo_url": "https://example.com/z.svg",
            "name": "Z Cert",
            "provider": "test-provider",
            "short_name": "Z-CERT",
            "url": "https://example.com/z"
        }
    ]'::jsonb,
    'Should return full certifications payload sorted by name'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
