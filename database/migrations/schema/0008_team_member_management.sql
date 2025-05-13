alter table employer_team add column approved boolean not null default false;
update employer_team set approved = true;
alter table employer_team add column created_at timestamptz default current_timestamp;

alter table notification drop constraint notification_user_id_key;

insert into notification_kind (name) values ('team-invitation');

---- create above / drop below ----

alter table employer_team drop column approved;
alter table employer_team drop column created_at;

delete from notification_kind where name = 'team-invitation';
