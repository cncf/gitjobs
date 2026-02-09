create table employer_member (
    employer_id uuid not null references employer on delete cascade,
    member_id uuid not null references member on delete cascade,

    primary key (employer_id, member_id)
);

create index employer_member_employer_id_idx on employer_member (employer_id);
create index employer_member_member_id_idx on employer_member (member_id);

insert into employer_member (employer_id, member_id)
select employer_id, member_id
from employer
where member_id is not null;

drop index if exists employer_member_id_idx;
alter table employer drop column member_id;
