alter table image add column created_by uuid references "user" on delete set null;
create index image_created_by_idx on image (created_by);

---- create above / drop below ----

alter table image drop column created_by;
drop index if exists image_created_by_idx;
