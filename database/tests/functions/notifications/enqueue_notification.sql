-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(1);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set userID '00000000-0000-0000-0000-000000000101'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID', 'alice');

-- ============================================================================
-- TESTS
-- ============================================================================

-- Should insert a notification with the provided payload
select notifications_enqueue_notification(
    'email-verification',
    jsonb_build_object('code', 'abc123'),
    :'userID'::uuid
);

select is(
    (
        select jsonb_build_object(
            'kind', kind,
            'processed', processed,
            'template_data', template_data,
            'user_id', user_id::text
        )
        from notification
        where user_id = :'userID'::uuid
    ),
    jsonb_build_object(
        'kind', 'email-verification',
        'processed', false,
        'template_data', jsonb_build_object('code', 'abc123'),
        'user_id', :'userID'
    ),
    'Should insert a notification with the provided payload'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
