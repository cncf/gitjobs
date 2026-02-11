-- Create a table to deduplicate notification template payloads by content hash
create table notification_template_data (
    notification_template_data_id uuid primary key default gen_random_uuid(),
    created_at timestamptz default current_timestamp not null,
    data jsonb not null,
    hash text not null constraint notification_template_data_hash_idx unique
);

-- Add a foreign key from notification rows to normalized template payloads
alter table notification add column notification_template_data_id uuid references notification_template_data;

-- Backfill deduplicated template payload rows from existing notification data
with template_data as (
    select distinct
        n.template_data as data,
        encode(digest(convert_to(n.template_data::text, 'utf8'), 'sha256'), 'hex') as hash
    from notification n
    where n.template_data is not null
)
insert into notification_template_data (data, hash)
select
    t.data,
    t.hash
from template_data t
on conflict (hash) do nothing;

-- Link existing notifications to their deduplicated template payload rows
update notification n
set notification_template_data_id = ntd.notification_template_data_id
from notification_template_data ntd
where n.template_data is not null
and ntd.hash = encode(digest(convert_to(n.template_data::text, 'utf8'), 'sha256'), 'hex');

-- Drop inline template payload data after migration to normalized storage
alter table notification drop column template_data;

-- Remove legacy function signatures before reloading updated definitions
drop function if exists enqueue_notification(text, jsonb, uuid);
drop function if exists search_applications(uuid, jsonb);
drop function if exists search_jobs(jsonb);
