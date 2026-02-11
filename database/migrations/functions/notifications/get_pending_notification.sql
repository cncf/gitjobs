-- Returns the next pending notification and locks it for processing.
create or replace function get_pending_notification()
returns table(
    kind text,
    notification_id uuid,
    template_data jsonb,
    email text
) as $$
    select
        n.kind,
        n.notification_id,
        n.template_data,
        u.email
    from notification n
    join "user" u using (user_id)
    where n.processed = false
    order by n.notification_id asc
    limit 1
    for update of n skip locked;
$$ language sql;
