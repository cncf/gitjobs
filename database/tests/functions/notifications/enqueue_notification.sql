-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(8);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set userID1 '00000000-0000-0000-0000-000000000101'
\set userID2 '00000000-0000-0000-0000-000000000102'
\set userID3 '00000000-0000-0000-0000-000000000103'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID1', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'userID2', 'bob'),
    (decode('03', 'hex'), 'carol@example.com', 'Carol', :'userID3', 'carol');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should enqueue one notification per recipient without template data
select lives_ok(
    format(
        $$select enqueue_notification(
            'email-verification',
            null::jsonb,
            array[%L, %L]::uuid[]
        )$$,
        :'userID1',
        :'userID2'
    ),
    'Should enqueue one notification per recipient without template data'
);

-- Should create email-verification notifications for expected recipients
select results_eq(
    $$
    select
        n.user_id,

        n.notification_template_data_id
    from notification n
    where n.kind = 'email-verification'
    order by n.user_id
    $$,
    $$ values
        ('00000000-0000-0000-0000-000000000101'::uuid, null::uuid),
        ('00000000-0000-0000-0000-000000000102'::uuid, null::uuid)
    $$,
    'Should create email-verification notifications for expected recipients'
);

-- Should enqueue notifications with deduplicated template data
select lives_ok(
    format(
        $$select enqueue_notification(
            'team-invitation',
            '{"link":"https://example.com/invitations"}'::jsonb,
            array[%L, %L]::uuid[]
        )$$,
        :'userID1',
        :'userID3'
    ),
    'Should enqueue notifications with deduplicated template data'
);

-- Should insert template data with expected hash
select results_eq(
    $$
    select
        ntd.data,
        ntd.hash
    from notification_template_data ntd
    where ntd.data = '{"link":"https://example.com/invitations"}'::jsonb
    $$,
    $$ values (
        '{"link":"https://example.com/invitations"}'::jsonb,
        encode(
            digest(convert_to('{"link":"https://example.com/invitations"}'::jsonb::text, 'utf8'), 'sha256'),
            'hex'
        )
    ) $$,
    'Should insert template data with expected hash'
);

-- Should link team-invitation notifications to expected template data
select results_eq(
    $$
    select
        n.user_id,

        ntd.data
    from notification n
    join notification_template_data ntd using (notification_template_data_id)
    where n.kind = 'team-invitation'
    order by n.user_id
    $$,
    $$ values
        ('00000000-0000-0000-0000-000000000101'::uuid, '{"link":"https://example.com/invitations"}'::jsonb),
        ('00000000-0000-0000-0000-000000000103'::uuid, '{"link":"https://example.com/invitations"}'::jsonb)
    $$,
    'Should link team-invitation notifications to expected template data'
);

-- Should reuse template data row for repeated hash
select lives_ok(
    format(
        $$select enqueue_notification(
            'team-invitation',
            '{"link":"https://example.com/invitations"}'::jsonb,
            array[%L]::uuid[]
        )$$,
        :'userID2'
    ),
    'Should reuse template data row for repeated hash'
);

-- Should keep one template hash across team-invitation notifications
select results_eq(
    $$
    select
        n.user_id,

        ntd.hash
    from notification n
    join notification_template_data ntd using (notification_template_data_id)
    where n.kind = 'team-invitation'
    order by n.user_id
    $$,
    $$ values
        (
            '00000000-0000-0000-0000-000000000101'::uuid,
            encode(
                digest(convert_to('{"link":"https://example.com/invitations"}'::jsonb::text, 'utf8'), 'sha256'),
                'hex'
            )
        ),
        (
            '00000000-0000-0000-0000-000000000102'::uuid,
            encode(
                digest(convert_to('{"link":"https://example.com/invitations"}'::jsonb::text, 'utf8'), 'sha256'),
                'hex'
            )
        ),
        (
            '00000000-0000-0000-0000-000000000103'::uuid,
            encode(
                digest(convert_to('{"link":"https://example.com/invitations"}'::jsonb::text, 'utf8'), 'sha256'),
                'hex'
            )
        )
    $$,
    'Should keep one template hash across team-invitation notifications'
);

-- Should not fail when recipients list is empty
select lives_ok(
    $$
    select enqueue_notification(
        'email-verification',
        null::jsonb,
        '{}'::uuid[]
    )
    $$,
    'Should not fail when recipients list is empty'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
