create table if not exists job_views (
     job_id uuid references job on delete set null,
     day date not null,
     total integer not null,
     unique (job_id, day)
 );

---- create above / drop below ----

drop table if exists job_views;
