-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set notificationIDWithError '00000000-0000-0000-0000-000000000201'
\set notificationIDWithoutError '00000000-0000-0000-0000-000000000202'
\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into notification (kind, notification_id, processed, user_id) values
    ('email-verification', :'notificationIDWithError', false, :'userID'),
    ('email-verification', :'notificationIDWithoutError', false, :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark notification as processed and persist error details
select update_notification(:'notificationIDWithError'::uuid, 'smtp timeout');

select ok(
    (
        select jsonb_build_object(
            'error', error,
            'kind', kind,
            'processed', processed,
            'user_id', user_id::text
        ) = jsonb_build_object(
            'error', 'smtp timeout',
            'kind', 'email-verification',
            'processed', true,
            'user_id', :'userID'
        )
        and processed_at is not null
        from notification
        where notification_id = :'notificationIDWithError'::uuid
    ),
    'Should mark notification as processed and persist error details'
);

-- Should mark notification as processed and keep error null when no error is provided
select update_notification(:'notificationIDWithoutError'::uuid, null);

select ok(
    (
        select jsonb_build_object(
            'error', error,
            'kind', kind,
            'processed', processed,
            'user_id', user_id::text
        ) = jsonb_build_object(
            'error', null,
            'kind', 'email-verification',
            'processed', true,
            'user_id', :'userID'
        )
        and processed_at is not null
        from notification
        where notification_id = :'notificationIDWithoutError'::uuid
    ),
    'Should mark notification as processed and keep error null when no error is provided'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
