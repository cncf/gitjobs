-- Enqueues a notification for delivery.
create or replace function enqueue_notification(
    p_kind text,
    p_template_data jsonb,
    p_recipients uuid[]
)
returns void as $$
declare
    v_notification_template_data_id uuid;
    v_template_hash text;
begin
    -- Insert or reuse template data and get its ID
    if p_template_data is not null then
        v_template_hash := encode(digest(convert_to(p_template_data::text, 'utf8'), 'sha256'), 'hex');

        insert into notification_template_data (data, hash)
        values (p_template_data, v_template_hash)
        on conflict (hash) do update set hash = notification_template_data.hash
        returning notification_template_data_id into v_notification_template_data_id;
    end if;

    -- Insert one notification per recipient
    insert into notification (kind, notification_template_data_id, user_id)
    select p_kind, v_notification_template_data_id, unnest(p_recipients);
end;
$$ language plpgsql;
