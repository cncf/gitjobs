-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(4);

-- ============================================================================
-- VARIABLES
-- ============================================================================

\set emailVerificationTemplateDataID '00000000-0000-0000-0000-000000000301'
\set notification1ID '00000000-0000-0000-0000-000000000201'
\set notification2ID '00000000-0000-0000-0000-000000000202'
\set processedNotificationID '00000000-0000-0000-0000-000000000205'
\set teamInvitationTemplateDataID '00000000-0000-0000-0000-000000000302'
\set unverifiedEmailVerificationNotificationID '00000000-0000-0000-0000-000000000204'
\set unverifiedTeamInvitationNotificationID '00000000-0000-0000-0000-000000000203'
\set userID1 '00000000-0000-0000-0000-000000000101'
\set userID2 '00000000-0000-0000-0000-000000000102'

-- ============================================================================
-- SEED DATA
-- ============================================================================

insert into "user" (auth_hash, email, name, user_id, username) values
    (decode('01', 'hex'), 'alice@example.com', 'Alice', :'userID1', 'alice'),
    (decode('02', 'hex'), 'bob@example.com', 'Bob', :'userID2', 'bob');

update "user"
set email_verified = true
where user_id = :'userID1'::uuid;

insert into notification_template_data (data, hash, notification_template_data_id) values
    (
        '{"link":"https://example.com/verify"}'::jsonb,
        encode(digest(convert_to('{"link":"https://example.com/verify"}'::jsonb::text, 'utf8'), 'sha256'), 'hex'),
        :'emailVerificationTemplateDataID'
    ),
    (
        '{"link":"https://example.com/team"}'::jsonb,
        encode(digest(convert_to('{"link":"https://example.com/team"}'::jsonb::text, 'utf8'), 'sha256'), 'hex'),
        :'teamInvitationTemplateDataID'
    );

insert into notification (
    created_at,
    kind,
    notification_id,
    notification_template_data_id,
    processed,
    user_id
) values
    (
        '2024-01-02 00:00:00+00',
        'team-invitation',
        :'notification2ID',
        :'teamInvitationTemplateDataID',
        false,
        :'userID1'
    ),
    (
        '2024-01-01 00:00:00+00',
        'team-invitation',
        :'notification1ID',
        :'teamInvitationTemplateDataID',
        false,
        :'userID1'
    ),
    (
        '2024-01-03 00:00:00+00',
        'team-invitation',
        :'unverifiedTeamInvitationNotificationID',
        :'teamInvitationTemplateDataID',
        false,
        :'userID2'
    ),
    (
        '2024-01-04 00:00:00+00',
        'email-verification',
        :'unverifiedEmailVerificationNotificationID',
        :'emailVerificationTemplateDataID',
        false,
        :'userID2'
    ),
    (
        '2024-01-05 00:00:00+00',
        'team-invitation',
        :'processedNotificationID',
        :'teamInvitationTemplateDataID',
        true,
        :'userID1'
    );

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
        from get_pending_notification()
    ),
    jsonb_build_object(
        'email', 'alice@example.com',
        'kind', 'team-invitation',
        'notification_id', :'notification1ID',
        'template_data', '{"link":"https://example.com/team"}'::jsonb
    ),
    'Should return full payload for the oldest pending notification'
);

-- Should ignore processed notifications
select is(
    (
        select count(*)
        from get_pending_notification()
        where notification_id = :'processedNotificationID'::uuid
    ),
    0::bigint,
    'Should ignore processed notifications'
);

update notification
set processed = true
where notification_id in (:'notification1ID'::uuid, :'notification2ID'::uuid);

-- Should allow email-verification notifications for users without verified email
select is(
    (
        select notification_id
        from get_pending_notification()
    ),
    :'unverifiedEmailVerificationNotificationID'::uuid,
    'Should allow email-verification notifications for users without verified email'
);

-- Should skip non-email-verification notifications for users without verified email
select is(
    (
        select count(*)
        from get_pending_notification()
        where notification_id = :'unverifiedTeamInvitationNotificationID'::uuid
    ),
    0::bigint,
    'Should skip non-email-verification notifications for users without verified email'
);

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
