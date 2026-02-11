-- ============================================================================
-- SETUP
-- ============================================================================

begin;
select plan(154);

-- ============================================================================
-- TESTS
-- ============================================================================

-- Test: check expected extensions exist
select has_extension('pgcrypto');
select has_extension('pg_trgm');
select has_extension('postgis');

-- Test: check expected tables exist
select has_table('application');
select has_table('certification');
select has_table('email_verification_code');
select has_table('employer');
select has_table('employer_member');
select has_table('employer_team');
select has_table('faq');
select has_table('foundation');
select has_table('image');
select has_table('image_version');
select has_table('job');
select has_table('job_certification');
select has_table('job_kind');
select has_table('job_project');
select has_table('job_seeker_profile');
select has_table('job_status');
select has_table('job_views');
select has_table('location');
select has_table('member');
select has_table('notification');
select has_table('notification_kind');
select has_table('project');
select has_table('search_appearances');
select has_table('seniority');
select has_table('session');
select has_table('user');
select has_table('workplace');

-- Test: application columns should match expected
select columns_are('application', array[
    'application_id',
    'job_seeker_profile_id',
    'job_id',
    'created_at',
    'cover_letter',
    'updated_at'
]);

-- Test: certification columns should match expected
select columns_are('certification', array[
    'certification_id',
    'name',
    'provider',
    'short_name',
    'description',
    'logo_url',
    'url'
]);

-- Test: email_verification_code columns should match expected
select columns_are('email_verification_code', array[
    'email_verification_code_id',
    'user_id',
    'created_at'
]);

-- Test: employer columns should match expected
select columns_are('employer', array[
    'employer_id',
    'location_id',
    'logo_id',
    'company',
    'created_at',
    'description',
    'public',
    'updated_at',
    'website_url'
]);

-- Test: employer_member columns should match expected
select columns_are('employer_member', array[
    'employer_id',
    'member_id'
]);

-- Test: employer_team columns should match expected
select columns_are('employer_team', array[
    'employer_id',
    'user_id',
    'approved',
    'created_at'
]);

-- Test: faq columns should match expected
select columns_are('faq', array[
    'faq_id',
    'answer',
    'question'
]);

-- Test: foundation columns should match expected
select columns_are('foundation', array[
    'foundation_id',
    'name',
    'landscape_url'
]);

-- Test: image columns should match expected
select columns_are('image', array[
    'image_id',
    'created_by'
]);

-- Test: image_version columns should match expected
select columns_are('image_version', array[
    'image_id',
    'version',
    'data'
]);

-- Test: job columns should match expected
select columns_are('job', array[
    'job_id',
    'employer_id',
    'kind',
    'seniority',
    'status',
    'location_id',
    'workplace',
    'created_at',
    'description',
    'title',
    'tsdoc',
    'apply_instructions',
    'apply_url',
    'archived_at',
    'benefits',
    'open_source',
    'published_at',
    'qualifications',
    'responsibilities',
    'salary',
    'salary_currency',
    'salary_max',
    'salary_min',
    'salary_period',
    'skills',
    'updated_at',
    'upstream_commitment',
    'tz_start',
    'tz_end',
    'salary_usd_year',
    'salary_min_usd_year',
    'salary_max_usd_year',
    'review_notes',
    'reviewed_by',
    'reviewed_at',
    'first_published_at',
    'deleted_at'
]);

-- Test: job_certification columns should match expected
select columns_are('job_certification', array[
    'job_id',
    'certification_id'
]);

-- Test: job_kind columns should match expected
select columns_are('job_kind', array[
    'job_kind_id',
    'name'
]);

-- Test: job_project columns should match expected
select columns_are('job_project', array[
    'job_id',
    'project_id'
]);

-- Test: job_seeker_profile columns should match expected
select columns_are('job_seeker_profile', array[
    'job_seeker_profile_id',
    'user_id',
    'location_id',
    'photo_id',
    'email',
    'name',
    'public',
    'summary',
    'certifications',
    'education',
    'experience',
    'facebook_url',
    'github_url',
    'linkedin_url',
    'open_to_relocation',
    'open_to_remote',
    'phone',
    'projects',
    'resume_url',
    'skills',
    'twitter_url',
    'website_url',
    'bluesky_url'
]);

-- Test: job_status columns should match expected
select columns_are('job_status', array[
    'job_status_id',
    'name'
]);

-- Test: job_views columns should match expected
select columns_are('job_views', array[
    'job_id',
    'day',
    'total'
]);

-- Test: location columns should match expected
select columns_are('location', array[
    'location_id',
    'city',
    'country',
    'tsdoc',
    'coordinates',
    'state'
]);

-- Test: member columns should match expected
select columns_are('member', array[
    'member_id',
    'foundation',
    'name',
    'level',
    'logo_url'
]);

-- Test: notification columns should match expected
select columns_are('notification', array[
    'notification_id',
    'kind',
    'user_id',
    'processed',
    'created_at',
    'error',
    'processed_at',
    'template_data'
]);

-- Test: notification_kind columns should match expected
select columns_are('notification_kind', array[
    'notification_kind_id',
    'name'
]);

-- Test: project columns should match expected
select columns_are('project', array[
    'project_id',
    'foundation',
    'name',
    'maturity',
    'logo_url'
]);

-- Test: search_appearances columns should match expected
select columns_are('search_appearances', array[
    'job_id',
    'day',
    'total'
]);

-- Test: seniority columns should match expected
select columns_are('seniority', array[
    'seniority_id',
    'name'
]);

-- Test: session columns should match expected
select columns_are('session', array[
    'session_id',
    'data',
    'expires_at'
]);

-- Test: user columns should match expected
select columns_are('user', array[
    'user_id',
    'auth_hash',
    'created_at',
    'email',
    'email_verified',
    'name',
    'username',
    'password',
    'moderator'
]);

-- Test: workplace columns should match expected
select columns_are('workplace', array[
    'workplace_id',
    'name'
]);

-- Test: check expected functions exist
select has_function('auth_get_user_by_email');
select has_function('auth_get_user_by_id_verified');
select has_function('auth_get_user_by_username');
select has_function('auth_get_user_password');
select has_function('auth_is_image_public');
select has_function('auth_sign_up_user');
select has_function('auth_update_user_details');
select has_function('auth_update_user_password');
select has_function('auth_user_has_image_access');
select has_function('auth_user_has_profile_access');
select has_function('auth_user_owns_employer');
select has_function('auth_user_owns_job');
select has_function('auth_verify_email');
select has_function('dashboard_employer_accept_team_member_invitation');
select has_function('dashboard_employer_add_employer');
select has_function('dashboard_employer_add_job');
select has_function('dashboard_employer_add_team_member');
select has_function('dashboard_employer_archive_job');
select has_function('dashboard_employer_delete_job');
select has_function('dashboard_employer_delete_team_member');
select has_function('dashboard_employer_get_applications_filters_options');
select has_function('dashboard_employer_get_employer');
select has_function('dashboard_employer_get_job_dashboard');
select has_function('dashboard_employer_get_job_salary');
select has_function('dashboard_employer_get_job_seeker_user_id');
select has_function('dashboard_employer_get_job_stats');
select has_function('dashboard_employer_get_user_invitations_count');
select has_function('dashboard_employer_list_certifications');
select has_function('dashboard_employer_list_employer_jobs');
select has_function('dashboard_employer_list_employers');
select has_function('dashboard_employer_list_foundations');
select has_function('dashboard_employer_list_team_members');
select has_function('dashboard_employer_list_user_invitations');
select has_function('dashboard_employer_publish_job');
select has_function('dashboard_employer_search_applications');
select has_function('dashboard_employer_update_employer');
select has_function('dashboard_employer_update_job');
select has_function('dashboard_job_seeker_cancel_application');
select has_function('dashboard_job_seeker_get_profile');
select has_function('dashboard_job_seeker_list_applications');
select has_function('dashboard_job_seeker_upsert_profile');
select has_function('dashboard_moderator_approve_job');
select has_function('dashboard_moderator_list_jobs_for_moderation');
select has_function('dashboard_moderator_reject_job');
select has_function('i_array_to_string');
select has_function('img_get_image_version');
select has_function('jobboard_apply_to_job');
select has_function('jobboard_get_job_jobboard');
select has_function('jobboard_get_jobs_filters_options');
select has_function('jobboard_get_stats');
select has_function('jobboard_search_jobs');
select has_function('jobboard_update_jobs_views');
select has_function('jobboard_update_search_appearances');
select has_function('misc_search_locations');
select has_function('misc_search_locations_json');
select has_function('misc_search_members');
select has_function('misc_search_projects');
select has_function('notifications_enqueue_notification');
select has_function('notifications_get_pending_notification');
select has_function('notifications_update_notification');
select has_function('syncer_add_member');
select has_function('syncer_add_project');
select has_function('syncer_list_foundations');
select has_function('syncer_list_members');
select has_function('syncer_list_projects');
select has_function('syncer_remove_member');
select has_function('syncer_remove_project');
select has_function('syncer_update_member');
select has_function('syncer_update_project');
select has_function('workers_archive_expired_jobs');

-- Test: check expected primary keys
select has_pk('application');
select has_pk('certification');
select has_pk('email_verification_code');
select has_pk('employer');
select has_pk('employer_member');
select has_pk('employer_team');
select has_pk('faq');
select has_pk('foundation');
select has_pk('image');
select has_pk('image_version');
select has_pk('job');
select has_pk('job_certification');
select has_pk('job_kind');
select has_pk('job_project');
select has_pk('job_seeker_profile');
select has_pk('job_status');
select hasnt_pk('job_views');
select has_pk('location');
select has_pk('member');
select has_pk('notification');
select has_pk('notification_kind');
select has_pk('project');
select hasnt_pk('search_appearances');
select has_pk('seniority');
select has_pk('session');
select has_pk('user');
select has_pk('workplace');

-- ============================================================================
-- CLEANUP
-- ============================================================================

select * from finish();
rollback;
