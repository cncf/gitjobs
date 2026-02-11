-- Enqueues a notification for delivery.
create or replace function enqueue_notification(
    p_kind text,
    p_template_data jsonb,
    p_user_id uuid
)
returns void as $$
    insert into notification (kind, user_id, template_data)
    values (p_kind, p_user_id, p_template_data);
$$ language sql;
