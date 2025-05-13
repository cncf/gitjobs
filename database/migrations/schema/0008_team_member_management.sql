alter table employer_team add column approved boolean not null default false;
update employer_team set approved = true;
alter table employer_team add column created_at timestamptz default current_timestamp;

---- create above / drop below ----

alter table employer_team drop column approved;
alter table employer_team drop column created_at;
