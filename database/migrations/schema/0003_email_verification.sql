create table if not exists email_verification_code (
    email_verification_code_id uuid primary key default gen_random_uuid(),
    user_id uuid not null unique references "user" on delete cascade,
    created_at timestamptz default current_timestamp not null
);

create index email_verification_code_user_id_index on email_verification_code(user_id);

---- create above / drop below ----

drop table email_verification_code;
