alter table application add constraint application_job_seeker_profile_id_job_id_key unique (job_seeker_profile_id, job_id);

---- create above / drop below ----

alter table application drop constraint application_job_seeker_profile_id_job_id_key;
