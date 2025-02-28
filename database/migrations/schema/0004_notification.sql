create table notification_kind (
    notification_kind_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into notification_kind (name) values ('email-verification');

create table notification (
    notification_id uuid primary key default gen_random_uuid(),
    kind text not null references notification_kind (name) on delete restrict,
    user_id uuid not null unique references "user" on delete cascade,
    processed boolean not null default false,
    created_at timestamptz default current_timestamp not null,

    error text check (error <> ''),
    processed_at timestamptz,
    template_data jsonb
);

create index notification_not_processed_idx on notification (notification_id) where processed = 'false';
create index notification_kind_idx on notification(kind);
create index notification_user_id_idx on notification(user_id);

---- create above / drop below ----

drop table notification;
drop table notification_kind;
