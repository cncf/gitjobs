//! Shared sample data builders for handlers tests.
#![allow(dead_code)]

use std::{collections::HashMap, sync::Arc};

use axum::Router;
use axum_login::tower_sessions::session;
use chrono::{TimeZone, Utc};
use serde_json::json;
use serde_qs::Config;
use time::{Duration as TimeDuration, OffsetDateTime};
use uuid::Uuid;

use crate::{
    auth::User as AuthUser,
    config::{CookieConfig, HttpServerConfig, LoginOptions},
    db::{DynDB, dashboard::employer::ApplicationsSearchOutput, jobboard::JobsSearchOutput, mock::MockDB},
    event_tracker::{Event, MockEventTracker},
    handlers::auth::{AUTH_PROVIDER_KEY, SELECTED_EMPLOYER_ID_KEY},
    img::{ImageFormat, MockImageStore},
    notifications::{MockNotificationsManager, NotificationKind},
    router,
    templates::{
        dashboard::{
            employer::{
                applications::FiltersOptions as ApplicationsFiltersOptions,
                employers::{Employer, EmployerSummary},
                jobs::{Job, JobKind, JobStats, JobStatus, JobSummary, Workplace},
                team::{TeamInvitation, TeamMember},
            },
            job_seeker::{applications::Application as JobSeekerApplication, profile::JobSeekerProfile},
            moderator::jobs::{Employer as ModeratorEmployer, JobSummary as ModeratorJobSummary},
        },
        jobboard::{
            jobs::{
                Employer as JobboardEmployer, FiltersOptions, Job as JobboardJob,
                JobSummary as JobboardJobSummary,
            },
            stats::{JobsStats, Stats},
        },
        misc::{Certification, Foundation, Location, Member, Project},
    },
};

/// Builder for test router configuration.
pub(crate) struct TestRouterBuilder {
    db: MockDB,

    cfg: Option<HttpServerConfig>,
    event_tracker: Option<MockEventTracker>,
    image_store: Option<MockImageStore>,
    notifications_manager: Option<MockNotificationsManager>,
}

impl TestRouterBuilder {
    /// Builds the application router with the configured options.
    pub(crate) async fn build(self) -> Router {
        let cfg = self.cfg.unwrap_or_else(test_http_server_cfg);
        let db: DynDB = Arc::new(self.db);
        let event_tracker = Arc::new(self.event_tracker.unwrap_or_default());
        let image_store = Arc::new(self.image_store.unwrap_or_default());
        let notifications_manager = Arc::new(self.notifications_manager.unwrap_or_default());

        router::setup(cfg, db, image_store, notifications_manager, event_tracker)
            .await
            .expect("router setup should succeed")
    }

    /// Creates a new test router builder with required dependencies.
    pub(crate) fn new(db: MockDB, notifications_manager: MockNotificationsManager) -> Self {
        Self {
            db,
            cfg: None,
            event_tracker: None,
            image_store: None,
            notifications_manager: Some(notifications_manager),
        }
    }

    /// Sets a custom HTTP server configuration.
    pub(crate) fn with_cfg(mut self, cfg: HttpServerConfig) -> Self {
        self.cfg = Some(cfg);
        self
    }

    /// Sets a custom event tracker.
    pub(crate) fn with_event_tracker(mut self, event_tracker: MockEventTracker) -> Self {
        self.event_tracker = Some(event_tracker);
        self
    }

    /// Sets a custom image store.
    pub(crate) fn with_image_store(mut self, image_store: MockImageStore) -> Self {
        self.image_store = Some(image_store);
        self
    }

    /// Sets a custom notifications manager.
    pub(crate) fn with_notifications_manager(
        mut self,
        notifications_manager: MockNotificationsManager,
    ) -> Self {
        self.notifications_manager = Some(notifications_manager);
        self
    }
}

/// Returns true when a session record has the expected flash message.
pub(crate) fn message_matches(record: &session::Record, expected_message: &str) -> bool {
    record
        .data
        .get("axum-messages.data")
        .and_then(|value| value.get("pending_messages"))
        .and_then(|messages| messages.as_array())
        .and_then(|messages| messages.first())
        .and_then(|message| message.get("m"))
        .and_then(|message| message.as_str())
        == Some(expected_message)
}

/// Sample authenticated user used across handler tests.
pub(crate) fn sample_auth_user(user_id: Uuid, auth_hash: &str) -> AuthUser {
    AuthUser {
        auth_hash: auth_hash.as_bytes().to_vec(),
        email: "user@example.test".to_string(),
        email_verified: true,
        has_profile: true,
        moderator: false,
        name: "Test User".to_string(),
        user_id,
        username: "test-user".to_string(),

        has_password: Some(true),
        password: Some(password_auth::generate_hash("password")),
    }
}

/// Sample certifications list used by dashboard jobs pages.
pub(crate) fn sample_certifications() -> Vec<Certification> {
    vec![Certification {
        certification_id: Uuid::new_v4(),
        name: "CKA".to_string(),
        provider: "CNCF".to_string(),
        short_name: "CKA".to_string(),

        description: Some("Kubernetes administrator".to_string()),
        logo_url: Some("https://example.test/cka.svg".to_string()),
        url: Some("https://example.test/cka".to_string()),
    }]
}

/// Sample employer details used by dashboard pages.
pub(crate) fn sample_employer(_employer_id: Uuid) -> Employer {
    Employer {
        company: "Example Corp".to_string(),
        description: "Employer description".to_string(),
        public: true,

        location: Some(sample_location()),
        logo_id: Some(Uuid::new_v4()),
        members: Some(vec![sample_member()]),
        website_url: Some("https://example.test".to_string()),
    }
}

/// Sample employer summary used for auth and dashboard tests.
pub(crate) fn sample_employer_summary(employer_id: Uuid) -> EmployerSummary {
    EmployerSummary {
        company: "Example Corp".to_string(),
        employer_id,

        logo_id: Some(Uuid::new_v4()),
    }
}

/// Sample employer application list output.
pub(crate) fn sample_employer_applications_output() -> ApplicationsSearchOutput {
    ApplicationsSearchOutput {
        applications: Vec::new(),
        total: 0,
    }
}

/// Sample employer applications filters options.
pub(crate) fn sample_employer_applications_filters_options(job_id: Uuid) -> ApplicationsFiltersOptions {
    ApplicationsFiltersOptions {
        jobs: vec![sample_employer_job_summary(job_id)],
    }
}

/// Sample employer job used by dashboard preview/update handlers.
pub(crate) fn sample_employer_job(job_id: Uuid) -> Job {
    Job {
        description: "Work on open-source tooling".to_string(),
        kind: JobKind::FullTime,
        status: JobStatus::Draft,
        title: "Rust Engineer".to_string(),
        workplace: Workplace::Remote,

        apply_instructions: Some("Apply with resume".to_string()),
        apply_url: Some("https://example.test/jobs/apply".to_string()),
        benefits: Some(vec!["Health".to_string()]),
        certifications: Some(sample_certifications()),
        job_id: Some(job_id),
        location: Some(sample_location()),
        open_source: Some(5),
        projects: Some(vec![sample_project()]),
        published_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap()),
        qualifications: Some("3+ years Rust".to_string()),
        responsibilities: Some("Build backend services".to_string()),
        review_notes: None,
        salary: Some(100_000),
        salary_currency: Some("USD".to_string()),
        salary_max: Some(130_000),
        salary_max_usd_year: Some(130_000),
        salary_min: Some(100_000),
        salary_min_usd_year: Some(100_000),
        salary_period: Some("year".to_string()),
        salary_usd_year: Some(120_000),
        seniority: Some(crate::templates::jobboard::jobs::Seniority::Senior),
        skills: Some(vec!["rust".to_string(), "sql".to_string()]),
        tz_end: Some("UTC+02:00".to_string()),
        tz_start: Some("UTC-08:00".to_string()),
        updated_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap()),
        upstream_commitment: Some(4),
    }
}

/// Sample employer job summary used across dashboard and filters tests.
pub(crate) fn sample_employer_job_summary(job_id: Uuid) -> JobSummary {
    JobSummary {
        created_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        job_id,
        status: JobStatus::Draft,
        title: "Rust Engineer".to_string(),
        workplace: Workplace::Remote,

        archived_at: None,
        city: Some("San Francisco".to_string()),
        country: Some("United States".to_string()),
        published_at: None,
        review_notes: None,
    }
}

/// Sample foundations list used by forms.
pub(crate) fn sample_foundations() -> Vec<Foundation> {
    vec![Foundation {
        name: "CNCF".to_string(),
    }]
}

/// Sample image payload used by image handlers tests.
pub(crate) fn sample_image() -> (Vec<u8>, ImageFormat) {
    (vec![0x89, 0x50, 0x4E, 0x47], ImageFormat::Png)
}

/// Sample job seeker application used in dashboard tests.
pub(crate) fn sample_job_seeker_application(application_id: Uuid, job_id: Uuid) -> JobSeekerApplication {
    JobSeekerApplication {
        application_id,
        applied_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        job_id,
        job_status: JobStatus::Published,
        job_title: "Rust Engineer".to_string(),
        job_workplace: Workplace::Remote,

        job_location: Some(sample_location()),
    }
}

/// Sample job seeker profile used by profile handlers tests.
pub(crate) fn sample_job_seeker_profile() -> JobSeekerProfile {
    JobSeekerProfile {
        email: "jane@example.test".to_string(),
        name: "Jane Doe".to_string(),
        public: true,
        summary: "Software engineer".to_string(),

        github_url: Some("https://github.com/jane".to_string()),
        location: Some(sample_location()),
        skills: Some(vec!["Rust".to_string(), "SQL".to_string()]),
        ..Default::default()
    }
}

/// Sample job board employer object.
pub(crate) fn sample_jobboard_employer(employer_id: Uuid) -> JobboardEmployer {
    JobboardEmployer {
        company: "Example Corp".to_string(),
        employer_id,
        description: Some("Employer description".to_string()),
        logo_id: Some(Uuid::new_v4()),
        members: Some(vec![sample_member()]),
        website_url: Some("https://example.test".to_string()),
    }
}

/// Sample job board filters options.
pub(crate) fn sample_jobboard_filters_options() -> FiltersOptions {
    FiltersOptions {
        foundations: sample_foundations(),
    }
}

/// Sample job board job used by detail and embed handlers.
pub(crate) fn sample_jobboard_job(job_id: Uuid, employer_id: Uuid) -> JobboardJob {
    JobboardJob {
        description: "Work on open-source tooling".to_string(),
        employer: sample_jobboard_employer(employer_id),
        job_id,
        kind: JobKind::FullTime,
        title: "Rust Engineer".to_string(),
        workplace: Workplace::Remote,

        apply_instructions: Some("Apply with resume".to_string()),
        apply_url: Some("https://example.test/jobs/apply".to_string()),
        benefits: Some(vec!["Health".to_string()]),
        certifications: Some(sample_certifications()),
        location: Some(sample_location()),
        open_source: Some(5),
        projects: Some(vec![sample_project()]),
        published_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap()),
        qualifications: Some("3+ years Rust".to_string()),
        responsibilities: Some("Build backend services".to_string()),
        salary: Some(100_000),
        salary_currency: Some("USD".to_string()),
        salary_max: Some(130_000),
        salary_min: Some(100_000),
        salary_period: Some("year".to_string()),
        seniority: Some(crate::templates::jobboard::jobs::Seniority::Senior),
        skills: Some(vec!["rust".to_string(), "sql".to_string()]),
        tz_end: Some("UTC+02:00".to_string()),
        tz_start: Some("UTC-08:00".to_string()),
        updated_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap()),
        upstream_commitment: Some(4),
    }
}

/// Sample job board jobs search output.
pub(crate) fn sample_jobboard_jobs_output(job_id: Uuid, employer_id: Uuid) -> JobsSearchOutput {
    JobsSearchOutput {
        jobs: vec![sample_jobboard_job_summary(job_id, employer_id)],
        total: 1,
    }
}

/// Sample job board job summary used in list pages.
pub(crate) fn sample_jobboard_job_summary(job_id: Uuid, employer_id: Uuid) -> JobboardJobSummary {
    JobboardJobSummary {
        employer: sample_jobboard_employer(employer_id),
        job_id,
        kind: JobKind::FullTime,
        published_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        title: "Rust Engineer".to_string(),
        workplace: Workplace::Remote,

        location: Some(sample_location()),
        open_source: Some(5),
        projects: Some(vec![sample_project()]),
        salary: Some(100_000),
        salary_currency: Some("USD".to_string()),
        salary_max: Some(130_000),
        salary_min: Some(100_000),
        salary_period: Some("year".to_string()),
        seniority: Some(crate::templates::jobboard::jobs::Seniority::Senior),
        skills: Some(vec!["rust".to_string(), "sql".to_string()]),
        updated_at: Some(Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap()),
        upstream_commitment: Some(4),
    }
}

/// Sample job board stats used by the stats page handler.
pub(crate) fn sample_jobboard_stats() -> Stats {
    Stats {
        jobs: JobsStats {
            published_per_foundation: Some(vec![("CNCF".to_string(), 1)]),
            published_per_month: Some(vec![("2024".to_string(), "01".to_string(), 1)]),
            published_running_total: Some(vec![(1_704_067_200_000, 1)]),
            views_daily: Some(vec![(1_704_067_200_000, 5)]),
            views_monthly: Some(vec![(1_704_067_200_000, 20)]),
        },
        ts_now: 1_706_745_600_000,
        ts_one_month_ago: 1_704_067_200_000,
        ts_two_years_ago: 1_643_673_600_000,
    }
}

/// Sample job stats used by employer dashboard job stats handler.
pub(crate) fn sample_job_stats() -> JobStats {
    JobStats {
        search_appearances_daily: Some(vec![(1_704_067_200_000, 10)]),
        search_appearances_total_last_month: 10,
        views_daily: Some(vec![(1_704_067_200_000, 8)]),
        views_total_last_month: 8,
    }
}

/// Sample location used across tests.
pub(crate) fn sample_location() -> Location {
    Location {
        city: "San Francisco".to_string(),
        country: "United States".to_string(),
        location_id: Uuid::new_v4(),

        state: Some("CA".to_string()),
    }
}

/// Sample member used in employer objects.
pub(crate) fn sample_member() -> Member {
    Member {
        foundation: "CNCF".to_string(),
        level: "Platinum".to_string(),
        logo_url: "https://example.test/member.svg".to_string(),
        member_id: Uuid::new_v4(),
        name: "Example Member".to_string(),
    }
}

/// Sample moderator job summary used by moderator handlers.
pub(crate) fn sample_moderator_job_summary(job_id: Uuid, employer_id: Uuid) -> ModeratorJobSummary {
    ModeratorJobSummary {
        created_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        employer: ModeratorEmployer {
            company: "Example Corp".to_string(),
            employer_id,
            logo_id: Some(Uuid::new_v4()),
            members: Some(vec![sample_member()]),
            website_url: Some("https://example.test".to_string()),
        },
        job_id,
        title: "Rust Engineer".to_string(),
    }
}

/// Sample project used by job objects.
pub(crate) fn sample_project() -> Project {
    Project {
        foundation: "CNCF".to_string(),
        logo_url: "https://example.test/project.svg".to_string(),
        maturity: "graduated".to_string(),
        name: "Kubernetes".to_string(),
        project_id: Uuid::new_v4(),
    }
}

/// Sample session record used across handler tests.
pub(crate) fn sample_session_record(
    session_id: session::Id,
    user_id: Uuid,
    auth_hash: &str,
    selected_employer_id: Option<Uuid>,
) -> session::Record {
    let mut data = HashMap::new();
    data.insert(
        "axum-login.data".to_string(),
        json!({
            "auth_hash": auth_hash.as_bytes(),
            "user_id": user_id,
        }),
    );
    if let Some(selected_employer_id) = selected_employer_id {
        data.insert(SELECTED_EMPLOYER_ID_KEY.to_string(), json!(selected_employer_id));
    }

    session::Record {
        data,
        expiry_date: OffsetDateTime::now_utc().saturating_add(TimeDuration::days(1)),
        id: session_id,
    }
}

/// Sample team invitation used by employer dashboard tests.
pub(crate) fn sample_team_invitation(employer_id: Uuid) -> TeamInvitation {
    TeamInvitation {
        company: "Example Corp".to_string(),
        created_at: Utc.with_ymd_and_hms(2024, 1, 2, 12, 0, 0).unwrap(),
        employer_id,
    }
}

/// Sample team member used by employer dashboard tests.
pub(crate) fn sample_team_member(user_id: Uuid) -> TeamMember {
    TeamMember {
        approved: true,
        email: "member@example.test".to_string(),
        name: "Team Member".to_string(),
        user_id,
        username: "team-member".to_string(),
    }
}

/// Creates a deterministic test HTTP server configuration.
pub(crate) fn test_http_server_cfg() -> HttpServerConfig {
    HttpServerConfig {
        addr: "127.0.0.1:9000".to_string(),
        base_url: "http://localhost:9000".to_string(),
        login: LoginOptions {
            email: true,
            github: false,
            linuxfoundation: false,
        },
        oauth2: HashMap::new(),
        oidc: HashMap::new(),

        analytics: None,
        basic_auth: None,
        cookie: Some(CookieConfig { secure: Some(false) }),
        slack_webhook_url: None,
    }
}

/// Configures mocks for tracking search appearances.
pub(crate) fn expect_track_search_appearances(event_tracker: &mut MockEventTracker, job_ids: Vec<Uuid>) {
    event_tracker
        .expect_track()
        .withf(move |event| {
            *event
                == Event::SearchAppearances {
                    job_ids: job_ids.clone(),
                }
        })
        .times(1)
        .returning(|_| Box::pin(async { Ok(()) }));
}

/// Configures mocks for tracking a single job view.
pub(crate) fn expect_track_view(event_tracker: &mut MockEventTracker, job_id: Uuid) {
    event_tracker
        .expect_track()
        .withf(move |event| *event == Event::JobView { job_id })
        .times(1)
        .returning(|_| Box::pin(async { Ok(()) }));
}

/// Verifies an enqueued notification payload.
pub(crate) fn notification_matches_kind(
    notification: &crate::notifications::NewNotification,
    kind: &NotificationKind,
) -> bool {
    notification.kind.to_string() == kind.to_string()
}

/// Creates a deterministic `serde_qs` configuration for handlers tests.
pub(crate) fn qs_config() -> Config {
    Config::new().max_depth(3).use_form_encoding(true)
}

/// Returns a sample auth provider string and stores it in session data.
pub(crate) fn with_auth_provider(record: &mut session::Record, provider: &str) {
    record
        .data
        .insert(AUTH_PROVIDER_KEY.to_string(), json!(provider.to_string()));
}
