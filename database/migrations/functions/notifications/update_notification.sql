-- Marks a notification as processed and stores the delivery error, if any.
create or replace function notifications_update_notification(
    p_notification_id uuid,
    p_error text
)
returns void as $$
    update notification
    set
        processed = true,
        processed_at = current_timestamp,
        error = p_error
    where notification_id = p_notification_id;
$$ language sql;
