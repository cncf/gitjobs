-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(2);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set notification1ID '00000000-0000-0000-0000-000000000201'
\set notification2ID '00000000-0000-0000-0000-000000000202'
\set processedNotificationID '00000000-0000-0000-0000-000000000203'
\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

insert into notification (kind, notification_id, processed, user_id) values
    ('email-verification', :'notification2ID', false, :'userID'),
    ('email-verification', :'notification1ID', false, :'userID'),
    ('email-verification', :'processedNotificationID', true, :'userID');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should return full payload for the oldest pending notification
select is(
    (
        select jsonb_build_object(
            'email', email,
            'kind', kind,
            'notification_id', notification_id::text,
            'template_data', template_data
        )
        from notifications_get_pending_notification()
    ),
    jsonb_build_object(
        'email', 'alice@example.com',
        'kind', 'email-verification',
        'notification_id', :'notification1ID',
        'template_data', null
    ),
    'Should return full payload for the oldest pending notification'
);

-- Should ignore processed notifications
select is(
    (
        select count(*)
        from notifications_get_pending_notification()
        where notification_id = :'processedNotificationID'::uuid
    ),
    0::bigint,
    'Should ignore processed notifications'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
