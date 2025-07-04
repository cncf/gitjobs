create table if not exists search_appearances (
    job_id uuid references job on delete set null,
    day date not null,
    total integer not null,
    unique (job_id, day)
);

---- create above / drop below ----

drop table if exists search_appearances;