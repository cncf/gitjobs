alter table job add review_notes text;
alter table job add reviewed_by uuid references "user" (user_id) on delete set null;
alter table job add reviewed_at timestamptz;

create index job_reviewed_by_idx on job (reviewed_by);

---- create above / drop below ----

alter table job drop column review_notes;
alter table job drop column reviewed_by;
alter table job drop column reviewed_at;
