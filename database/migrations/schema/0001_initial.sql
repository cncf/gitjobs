create extension pgcrypto;
create extension postgis;
create extension pg_trgm;

create table job_board (
    job_board_id uuid primary key default gen_random_uuid(),

    active boolean not null default false,
    benefits text[],
    created_at timestamptz not null default current_timestamp,
    description text not null check (description <> ''),
    display_name text not null unique check (display_name <> ''),
    header_logo_url text not null check (header_logo_url <> ''),
    host text not null unique check (host <> ''),
    name text not null unique check (name <> ''),
    theme jsonb not null,
    title text not null check (title <> ''),
    skills text[],

    about_intro text check (about_intro <> ''),
    extra_links jsonb,
    footer_logo_url text check (footer_logo_url <> ''),
    updated_at timestamptz
);

create table "user" (
    user_id uuid primary key default gen_random_uuid(),
    job_board_id uuid not null references job_board,

    auth_hash bytea not null check (auth_hash <> ''),
    created_at timestamptz not null default current_timestamp,
    email text not null check (email <> ''),
    email_verified boolean not null default false,
    name text not null check (name <> ''),
    username text not null check (username <> ''),

    password text check (password <> ''),

    unique (email, job_board_id),
    unique (username, job_board_id)
);

create index user_job_board_id_idx on "user" (job_board_id);

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

create table image (
    image_id uuid not null primary key
);

create table image_version (
    image_id uuid not null references image on delete cascade,
    version text not null check (version <> ''),
    data bytea not null,
    primary key (image_id, version)
);

create table job_seeker_profile (
    job_seeker_profile_id uuid primary key default gen_random_uuid(),
    user_id uuid not null unique references "user",
    location_id uuid references location,
    photo_id uuid references image (image_id),

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
    photo_url text check (photo_url <> ''),
    projects jsonb,
    resume_url text check (resume_url <> ''),
    skills text[],
    twitter_url text check (twitter_url <> ''),
    website_url text check (website_url <> '')
);

create index job_seeker_profile_location_id_idx on job_seeker_profile (location_id);
create index job_seeker_profile_photo_id_idx on job_seeker_profile (photo_id);
create index job_seeker_profile_user_id_idx on job_seeker_profile (user_id);

create table employer_tier (
    employer_tier_id uuid primary key default gen_random_uuid(),
    job_board_id uuid not null references job_board,

    name text not null unique check (name <> ''),
    highlight boolean not null default false,
    priority int not null default 0
);

create index tier_job_board_id_idx on employer_tier (job_board_id);

create table employer (
    employer_id uuid primary key default gen_random_uuid(),
    job_board_id uuid not null references job_board,
    employer_tier_id uuid references employer_tier,
    location_id uuid references location,
    logo_id uuid references image (image_id),

    company text not null check (company <> ''),
    created_at timestamptz not null default current_timestamp,
    description text not null check (description <> ''),
    public boolean not null default false,

    updated_at timestamptz,
    website_url text check (website_url <> '')
);

create index employer_job_board_id_idx on employer (job_board_id);
create index employer_employer_tier_id_idx on employer (employer_tier_id);
create index employer_location_id_idx on employer (location_id);
create index employer_logo_id_idx on employer (logo_id);

create table employer_team (
    employer_id uuid not null references employer on delete cascade,
    user_id uuid not null references "user",

    primary key (employer_id, user_id)
);

create index employer_team_employer_id_idx on employer_team (employer_id);
create index employer_team_user_id_idx on employer_team (user_id);

create table job_type (
    job_type_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into job_type (name) values ('full-time');
insert into job_type (name) values ('part-time');
insert into job_type (name) values ('contractor');
insert into job_type (name) values ('internship');

create table job_status (
    job_status_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into job_status (name) values ('archived');
insert into job_status (name) values ('draft');
insert into job_status (name) values ('published');

create table workplace (
    workplace_id uuid primary key default gen_random_uuid(),

    name text not null unique check (name <> '')
);

insert into workplace (name) values ('hybrid');
insert into workplace (name) values ('on-site');
insert into workplace (name) values ('remote');

create table job (
    job_id uuid primary key default gen_random_uuid(),
    employer_id uuid not null references employer,
    type text not null references job_type (name),
    status text not null references job_status (name),
    location_id uuid references location,
    workplace text not null references workplace (name),

    created_at timestamptz not null default current_timestamp,
    title text not null check (title <> ''),
    description text not null check (description <> ''),

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
create index job_location_id_idx on job (location_id);
create index job_type_idx on job (type);
create index job_status_idx on job (status);
create index job_workplace_idx on job (workplace);

create table applicant (
    applicant_id uuid primary key default gen_random_uuid(),
    job_seeker_profile_id uuid not null references job_seeker_profile,
    job_id uuid not null references job,

    cover_letter text not null check (cover_letter <> ''),
    created_at timestamptz not null default current_timestamp,

    updated_at timestamptz
);

create index applicant_job_seeker_profile_id_idx on applicant (job_seeker_profile_id);
create index applicant_job_id_idx on applicant (job_id);

create table faq (
    faq_id uuid primary key default gen_random_uuid(),
    job_board_id uuid not null references job_board,

    answer text not null check (answer <> ''),
    question text not null check (question <> '')
);

create index faq_job_board_id_idx on faq (job_board_id);

create table session (
    session_id text primary key,

    data jsonb not null,
    expires_at timestamptz not null
);
