alter table job add column salary_usd_year bigint check (salary_usd_year >= 0);
alter table job add column salary_min_usd_year bigint check (salary_min_usd_year >= 0);
alter table job add column salary_max_usd_year bigint check (salary_max_usd_year >= 0);

---- create above / drop below ----

alter table job drop column salary_usd_year;
alter table job drop column salary_min_usd_year;
alter table job drop column salary_max_usd_year;
