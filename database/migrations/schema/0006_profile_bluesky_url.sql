alter table job_seeker_profile add column bluesky_url text check (bluesky_url <> '');

---- create above / drop below ----

alter table job_seeker_profile drop column bluesky_url;
