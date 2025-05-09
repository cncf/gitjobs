alter table employer_team add column invitation_accepted boolean not null default false;

---- create above / drop below ----

alter table employer_team drop column invitation_accepted;
