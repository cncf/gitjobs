insert into job_status (name) values ('pending-approval');
insert into job_status (name) values ('rejected');

alter table "user" add moderator boolean not null default false;

alter table job add review_notes text;
alter table job add reviewed_by uuid references "user" (user_id) on delete set null;
alter table job add reviewed_at timestamptz;

create index job_reviewed_by_idx on job (reviewed_by);

---- create above / drop below ----

alter table job drop column review_notes;
alter table job drop column reviewed_by;
alter table job drop column reviewed_at;

alter table "user" drop column moderator;

delete from job_status where name = 'pending-approval';
delete from job_status where name = 'rejected';
