insert into job_status (name) values ('deleted');

alter table job add column deleted_at timestamptz;

---- create above / drop below ----

delete from job_status where name = 'deleted';

alter table job drop column deleted_at;
