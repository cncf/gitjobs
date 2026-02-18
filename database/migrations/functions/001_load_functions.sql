{{ template "auth/get_user_by_email.sql" }}
{{ template "auth/get_user_by_id_verified.sql" }}
{{ template "auth/get_user_by_username.sql" }}
{{ template "auth/get_user_password.sql" }}
{{ template "auth/is_image_public.sql" }}
{{ template "auth/sign_up_user.sql" }}
{{ template "auth/update_user_details.sql" }}
{{ template "auth/update_user_password.sql" }}
{{ template "auth/user_has_image_access.sql" }}
{{ template "auth/user_has_profile_access.sql" }}
{{ template "auth/user_owns_employer.sql" }}
{{ template "auth/user_owns_job.sql" }}
{{ template "auth/verify_email.sql" }}

{{ template "dashboard/accept_team_member_invitation.sql" }}
{{ template "dashboard/add_employer.sql" }}
{{ template "dashboard/add_job.sql" }}
{{ template "dashboard/add_team_member.sql" }}
{{ template "dashboard/approve_job.sql" }}
{{ template "dashboard/archive_job.sql" }}
{{ template "dashboard/cancel_application.sql" }}
{{ template "dashboard/delete_job.sql" }}
{{ template "dashboard/delete_team_member.sql" }}
{{ template "dashboard/get_applications_filters_options.sql" }}
{{ template "dashboard/get_employer.sql" }}
{{ template "dashboard/get_job_dashboard.sql" }}
{{ template "dashboard/get_job_salary.sql" }}
{{ template "dashboard/get_job_seeker_profile.sql" }}
{{ template "dashboard/get_job_seeker_user_id.sql" }}
{{ template "dashboard/get_job_stats.sql" }}
{{ template "dashboard/get_user_invitations_count.sql" }}
{{ template "dashboard/list_certifications.sql" }}
{{ template "dashboard/list_employer_jobs.sql" }}
{{ template "dashboard/list_employers.sql" }}
{{ template "dashboard/list_foundations.sql" }}
{{ template "dashboard/list_job_seeker_applications.sql" }}
{{ template "dashboard/list_jobs_for_moderation.sql" }}
{{ template "dashboard/list_team_members.sql" }}
{{ template "dashboard/list_user_invitations.sql" }}
{{ template "dashboard/publish_job.sql" }}
{{ template "dashboard/reject_job.sql" }}
{{ template "dashboard/search_applications.sql" }}
{{ template "dashboard/update_employer.sql" }}
{{ template "dashboard/update_job.sql" }}
{{ template "dashboard/upsert_job_seeker_profile.sql" }}

{{ template "img/get_image_version.sql" }}

{{ template "jobboard/apply_to_job.sql" }}
{{ template "jobboard/get_job_jobboard.sql" }}
{{ template "jobboard/get_jobs_filters_options.sql" }}
{{ template "jobboard/get_stats.sql" }}
{{ template "jobboard/search_jobs.sql" }}
{{ template "jobboard/update_jobs_views.sql" }}
{{ template "jobboard/update_search_appearances.sql" }}

{{ template "misc/search_locations.sql" }}
{{ template "misc/search_locations_json.sql" }}
{{ template "misc/search_members.sql" }}
{{ template "misc/search_projects.sql" }}

{{ template "notifications/enqueue_notification.sql" }}
{{ template "notifications/get_pending_notification.sql" }}
{{ template "notifications/update_notification.sql" }}

{{ template "syncer/add_member.sql" }}
{{ template "syncer/add_project.sql" }}
{{ template "syncer/list_foundations.sql" }}
{{ template "syncer/list_members.sql" }}
{{ template "syncer/list_projects.sql" }}
{{ template "syncer/remove_member.sql" }}
{{ template "syncer/remove_project.sql" }}
{{ template "syncer/update_member.sql" }}
{{ template "syncer/update_project.sql" }}

{{ template "workers/archive_expired_jobs.sql" }}

---- create above / drop below ----

-- Nothing to do
