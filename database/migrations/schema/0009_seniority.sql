create table seniority (
    seniority_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into seniority (name) values ('entry');
insert into seniority (name) values ('junior');
insert into seniority (name) values ('mid');
insert into seniority (name) values ('senior');
insert into seniority (name) values ('lead');

alter table job add column seniority text references seniority (name) on delete restrict;

---- create above / drop below ----

alter table job drop column seniority;
drop table seniority;
