alter table job add column tz_start text;
alter table job add column tz_end text;

---- create above / drop below ----

alter table job drop column tz_start;
alter table job drop column tz_end;
