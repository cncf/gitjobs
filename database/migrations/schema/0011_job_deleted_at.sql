alter table job add column deleted_at timestamptz;

---- create above / drop below ----

alter table job drop column deleted_at;