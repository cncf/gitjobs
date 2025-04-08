alter table "user" add moderator boolean not null default false;

---- create above / drop below ----

alter table "user" drop column moderator;
