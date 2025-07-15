-- job_project table
create index job_project_job_id_idx on job_project (job_id);
create index job_project_project_id_idx on job_project (project_id);

-- job table
create index job_published_at_idx on job (published_at);
create index job_published_at_desc_idx on job (published_at DESC) where status = 'published';
create index job_created_at_idx on job (created_at);
create index job_first_published_at_idx on job (first_published_at);
create index job_salary_max_usd_year_idx on job (salary_max_usd_year);
create index job_open_source_idx on job (open_source);
create index job_upstream_commitment_idx on job (upstream_commitment);
create index job_seniority_idx on job (seniority);

-- job_views table
create index job_views_job_id_idx on job_views (job_id);
create index job_views_day_idx on job_views (day);
create index job_views_job_id_day_idx on job_views (job_id, day);

-- search_appearances table
create index search_appearances_job_id_idx on search_appearances (job_id);
create index search_appearances_day_idx on search_appearances (day);
create index search_appearances_job_id_day_idx on search_appearances (job_id, day);

-- employer_team table
create index employer_team_approved_idx on employer_team (approved) where approved = false;
create index employer_team_user_id_approved_idx on employer_team (user_id, approved);

-- application table
create index application_created_at_idx on application (created_at);

-- session table
create index session_expires_at_idx on session (expires_at);

-- user table
create index user_email_verified_idx on "user" (email_verified) where email_verified = false;

---- create above / drop below ----

drop index if exists user_email_verified_idx;
drop index if exists session_expires_at_idx;
drop index if exists application_created_at_idx;
drop index if exists employer_team_user_id_approved_idx;
drop index if exists employer_team_approved_idx;
drop index if exists search_appearances_job_id_day_idx;
drop index if exists search_appearances_day_idx;
drop index if exists search_appearances_job_id_idx;
drop index if exists job_views_job_id_day_idx;
drop index if exists job_views_day_idx;
drop index if exists job_views_job_id_idx;
drop index if exists job_seniority_idx;
drop index if exists job_upstream_commitment_idx;
drop index if exists job_open_source_idx;
drop index if exists job_salary_max_usd_year_idx;
drop index if exists job_first_published_at_idx;
drop index if exists job_created_at_idx;
drop index if exists job_published_at_desc_idx;
drop index if exists job_published_at_idx;
drop index if exists job_project_project_id_idx;
drop index if exists job_project_job_id_idx;
