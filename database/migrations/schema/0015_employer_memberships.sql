-- Create a junction table to support many-to-many employer-member relationships
create table employer_member (
    employer_id uuid not null references employer on delete cascade,
    member_id uuid not null references member on delete cascade,

    primary key (employer_id, member_id)
);

-- Add lookup indexes for both relationship directions
create index employer_member_employer_id_idx on employer_member (employer_id);
create index employer_member_member_id_idx on employer_member (member_id);

-- Backfill the junction table from the legacy single-member column
insert into employer_member (employer_id, member_id)
select employer_id, member_id
from employer
where member_id is not null;

-- Remove obsolete index and legacy column from employer
drop index if exists employer_member_id_idx;
alter table employer drop column member_id;
