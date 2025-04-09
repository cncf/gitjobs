insert into job_status (name) values ('pending-approval');
insert into job_status (name) values ('rejected');

---- create above / drop below ----

delete from job_status where name = 'pending-approval';
delete from job_status where name = 'rejected';
