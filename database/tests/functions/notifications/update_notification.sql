-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set notificationID '00000000-0000-0000-0000-000000000201'
\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into notification (kind, notification_id, processed, user_id) values
    ('email-verification', :'notificationID', false, :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should mark notification as processed and persist error details
select update_notification(:'notificationID'::uuid, 'smtp timeout');

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
        where notification_id = :'notificationID'::uuid
    ),
    'Should mark notification as processed and persist error details'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
