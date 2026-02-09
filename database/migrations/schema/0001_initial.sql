create extension if not exists pgcrypto;
create extension if not exists postgis;
create extension if not exists pg_trgm;

create or replace function i_array_to_string(text[], text)
returns text language sql immutable as $$select array_to_string($1, $2)$$;

create table if not exists foundation (
    foundation_id uuid primary key default gen_random_uuid(),
    name text not null check (name <> '') unique
);

insert into foundation (name) values ('cncf');

create table member (
    member_id uuid primary key default gen_random_uuid(),
    foundation text not null references foundation (name) on delete restrict,

    name text not null check (name <> ''),
    level text not null check (level <> ''),
    logo_url text not null check (logo_url <> ''),

    unique (name, foundation)
);

create index member_foundation_idx on member (foundation);

create table project (
    project_id uuid primary key default gen_random_uuid(),
    foundation text not null references foundation (name) on delete restrict,

    name text not null check (name <> ''),
    maturity text not null check (maturity <> ''),
    logo_url text not null check (logo_url <> ''),

    unique (name, foundation)
);

create index project_foundation_idx on project (foundation);

create table location (
    location_id uuid primary key,

    city text not null check (city <> ''),
    country text not null check (country <> ''),
    tsdoc tsvector not null
        generated always as (
            setweight(to_tsvector('simple', city), 'A') ||
            setweight(to_tsvector('simple', country), 'B') ||
            setweight(to_tsvector('simple', coalesce(state, '')), 'B')
        ) stored,

    coordinates geography(point, 4326),
    state text check (state <> '')
);

create index location_coordinates_idx on location using gist (coordinates);
create index location_tsdoc_idx on location using gin (tsdoc);

create table "user" (
    user_id uuid primary key default gen_random_uuid(),

    auth_hash bytea not null check (auth_hash <> ''),
    created_at timestamptz not null default current_timestamp,
    email text not null check (email <> '') unique,
    email_verified boolean not null default false,
    name text not null check (name <> ''),
    username text not null check (username <> '') unique,

    password text check (password <> '')
);

create table image (
    image_id uuid not null primary key,
    created_by uuid references "user" on delete set null
);

create index image_created_by_idx on image (created_by);

create table image_version (
    image_id uuid not null references image on delete cascade,
    version text not null check (version <> ''),
    data bytea not null,
    primary key (image_id, version)
);

create table job_seeker_profile (
    job_seeker_profile_id uuid primary key default gen_random_uuid(),
    user_id uuid not null unique references "user" on delete cascade,
    location_id uuid references location on delete set null,
    photo_id uuid references image (image_id) on delete set null,

    email text not null check (email <> ''),
    name text not null check (name <> ''),
    public boolean not null default false,
    summary text not null check (summary <> ''),

    certifications jsonb,
    education jsonb,
    experience jsonb,
    facebook_url text check (facebook_url <> ''),
    github_url text check (github_url <> ''),
    linkedin_url text check (linkedin_url <> ''),
    open_to_relocation boolean,
    open_to_remote boolean,
    phone text check (phone <> ''),
    projects jsonb,
    resume_url text check (resume_url <> ''),
    skills text[],
    twitter_url text check (twitter_url <> ''),
    website_url text check (website_url <> '')
);

create index job_seeker_profile_location_id_idx on job_seeker_profile (location_id);
create index job_seeker_profile_photo_id_idx on job_seeker_profile (photo_id);
create index job_seeker_profile_user_id_idx on job_seeker_profile (user_id);

create table employer (
    employer_id uuid primary key default gen_random_uuid(),
    location_id uuid references location on delete set null,
    logo_id uuid references image (image_id) on delete set null,
    member_id uuid references member on delete set null,

    company text not null check (company <> ''),
    created_at timestamptz not null default current_timestamp,
    description text not null check (description <> ''),
    public boolean not null default false,

    updated_at timestamptz,
    website_url text check (website_url <> '')
);

create index employer_member_id_idx on employer (member_id);
create index employer_location_id_idx on employer (location_id);
create index employer_logo_id_idx on employer (logo_id);

create table employer_team (
    employer_id uuid not null references employer on delete cascade,
    user_id uuid not null references "user" on delete cascade,

    primary key (employer_id, user_id)
);

create index employer_team_employer_id_idx on employer_team (employer_id);
create index employer_team_user_id_idx on employer_team (user_id);

create table job_kind (
    job_kind_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into job_kind (name) values ('full-time');
insert into job_kind (name) values ('part-time');
insert into job_kind (name) values ('contractor');
insert into job_kind (name) values ('internship');

create table job_status (
    job_status_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into job_status (name) values ('archived');
insert into job_status (name) values ('draft');
insert into job_status (name) values ('published');

create table seniority (
    seniority_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into seniority (name) values ('entry');
insert into seniority (name) values ('junior');
insert into seniority (name) values ('mid');
insert into seniority (name) values ('senior');
insert into seniority (name) values ('lead');

create table workplace (
    workplace_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into workplace (name) values ('hybrid');
insert into workplace (name) values ('on-site');
insert into workplace (name) values ('remote');

create table job (
    job_id uuid primary key default gen_random_uuid(),
    employer_id uuid not null references employer on delete cascade,
    kind text not null references job_kind (name) on delete restrict,
    seniority text references seniority (name) on delete restrict,
    status text not null references job_status (name) on delete restrict,
    location_id uuid references location on delete set null,
    workplace text not null references workplace (name) on delete restrict,

    created_at timestamptz not null default current_timestamp,
    description text not null check (description <> ''),
    title text not null check (title <> ''),
    tsdoc tsvector not null
        generated always as (
            setweight(to_tsvector('simple', title), 'A') ||
            setweight(to_tsvector('simple', i_array_to_string(coalesce(skills, '{}'), ' ')), 'B') ||
            setweight(to_tsvector('simple', description), 'C')
        ) stored,

    apply_instructions text check (apply_instructions <> ''),
    apply_url text check (apply_url <> ''),
    archived_at timestamptz,
    benefits text[],
    open_source int check (open_source >= 0 and open_source <= 100),
    published_at timestamptz,
    qualifications text check (qualifications <> ''),
    responsibilities text check (responsibilities <> ''),
    salary bigint check (salary >= 0),
    salary_currency text check (salary_currency <> ''),
    salary_max bigint check (salary_max >= 0),
    salary_min bigint check (salary_min >= 0),
    salary_period text check (salary_period <> ''),
    skills text[],
    updated_at timestamptz,
    upstream_commitment int check (upstream_commitment >= 0 and upstream_commitment <= 100)
);

create index job_employer_id_idx on job (employer_id);
create index job_kind_idx on job (kind);
create index job_location_id_idx on job (location_id);
create index job_status_idx on job (status);
create index job_workplace_idx on job (workplace);

create table job_project (
    job_id uuid not null references job on delete cascade,
    project_id uuid not null references project on delete cascade,

    primary key (job_id, project_id)
);

create table application (
    application_id uuid primary key default gen_random_uuid(),
    job_seeker_profile_id uuid not null references job_seeker_profile on delete cascade,
    job_id uuid not null references job on delete cascade,

    created_at timestamptz not null default current_timestamp,

    cover_letter text check (cover_letter <> ''),
    updated_at timestamptz
);

create index application_job_seeker_profile_id_idx on application (job_seeker_profile_id);
create index application_job_id_idx on application (job_id);

create table faq (
    faq_id uuid primary key default gen_random_uuid(),

    answer text not null check (answer <> ''),
    question text not null check (question <> '')
);

create table session (
    session_id text primary key,

    data jsonb not null,
    expires_at timestamptz not null
);

create table if not exists email_verification_code (
    email_verification_code_id uuid primary key default gen_random_uuid(),
    user_id uuid not null unique references "user" on delete cascade,
    created_at timestamptz default current_timestamp not null
);

create index email_verification_code_user_id_index on email_verification_code(user_id);

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
