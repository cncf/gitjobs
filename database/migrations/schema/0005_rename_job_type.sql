alter table job_type rename to job_kind;
alter table job rename column type to kind;
alter index job_type_idx rename to job_kind_idx;

---- create above / drop below ----

alter index job_kind_idx rename to job_type_idx;
alter table job rename column kind to type;
alter table job_kind rename to job_type;
