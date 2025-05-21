alter table job add column first_published_at timestamptz;

---- create above / drop below ----

alter table job drop column first_published_at;
