alter table notification drop constraint notification_user_id_key;

---- create above / drop below ----

alter table notification add constraint notification_user_id_key unique (user_id);
