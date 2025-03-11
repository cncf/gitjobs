alter table applicant rename to application;
alter table application rename column applicant_id to application_id;
alter index applicant_job_seeker_profile_id_idx rename to application_job_seeker_profile_id_idx;
alter index applicant_job_id_idx rename to application_job_id_idx;

---- create above / drop below ----

alter index application_job_id_idx rename to applicant_job_id_idx;
alter index application_job_seeker_profile_id_idx rename to applicant_job_seeker_profile_id_idx;
alter table application rename column application_id to applicant_id;
alter table application rename to applicant;
