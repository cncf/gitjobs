//! DB trait mock implementation for testing.

use anyhow::Result;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use mockall::mock;
use uuid::Uuid;

mock! {
    /// Mock `DB` struct for testing purposes, implementing DB traits.
    pub(crate) DB {}

    #[async_trait]
    impl crate::db::DB for DB {
        async fn tx_begin(&self) -> Result<Uuid>;
        async fn tx_commit(&self, client_id: Uuid) -> Result<()>;
        async fn tx_rollback(&self, client_id: Uuid) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::auth::DBAuth for DB {
        async fn create_session(
            &self,
            record: &axum_login::tower_sessions::session::Record,
        ) -> Result<()>;
        async fn delete_session(
            &self,
            session_id: &axum_login::tower_sessions::session::Id,
        ) -> Result<()>;
        async fn get_session(
            &self,
            session_id: &axum_login::tower_sessions::session::Id,
        ) -> Result<Option<axum_login::tower_sessions::session::Record>>;
        async fn get_user_by_email(
            &self,
            email: &str,
        ) -> Result<Option<crate::auth::User>>;
        async fn get_user_by_id(
            &self,
            user_id: &Uuid,
        ) -> Result<Option<crate::auth::User>>;
        async fn get_user_by_username(
            &self,
            username: &str,
        ) -> Result<Option<crate::auth::User>>;
        async fn get_user_password(&self, user_id: &Uuid) -> Result<Option<String>>;
        async fn is_image_public(&self, image_id: &Uuid) -> Result<bool>;
        async fn sign_up_user(
            &self,
            user_summary: &crate::auth::UserSummary,
            email_verified: bool,
        ) -> Result<(crate::auth::User, Option<crate::db::auth::VerificationCode>)>;
        async fn update_session(
            &self,
            record: &axum_login::tower_sessions::session::Record,
        ) -> Result<()>;
        async fn update_user_details(
            &self,
            user_id: &Uuid,
            user_summary: &crate::auth::UserSummary,
        ) -> Result<()>;
        async fn update_user_password(
            &self,
            user_id: &Uuid,
            new_password: &str,
        ) -> Result<()>;
        async fn user_has_image_access(
            &self,
            user_id: &Uuid,
            image_id: &Uuid,
        ) -> Result<bool>;
        async fn user_has_profile_access(
            &self,
            user_id: &Uuid,
            job_seeker_profile_id: &Uuid,
        ) -> Result<bool>;
        async fn user_owns_employer(
            &self,
            user_id: &Uuid,
            employer_id: &Uuid,
        ) -> Result<bool>;
        async fn user_owns_job(&self, user_id: &Uuid, job_id: &Uuid) -> Result<bool>;
        async fn verify_email(&self, code: &Uuid) -> Result<()>;
    }

    impl crate::db::dashboard::DBDashBoard for DB {}

    #[async_trait]
    impl crate::db::dashboard::employer::DBDashBoardEmployer for DB {
        async fn accept_team_member_invitation(
            &self,
            employer_id: &Uuid,
            user_id: &Uuid,
        ) -> Result<()>;
        async fn add_employer(
            &self,
            user_id: &Uuid,
            employer: &crate::templates::dashboard::employer::employers::Employer,
        ) -> Result<Uuid>;
        async fn add_job(
            &self,
            employer_id: &Uuid,
            job: &crate::templates::dashboard::employer::jobs::Job,
        ) -> Result<()>;
        async fn add_team_member(
            &self,
            employer_id: &Uuid,
            email: &str,
        ) -> Result<Option<Uuid>>;
        async fn archive_job(&self, job_id: &Uuid) -> Result<()>;
        async fn delete_job(&self, job_id: &Uuid) -> Result<()>;
        async fn delete_team_member(
            &self,
            employer_id: &Uuid,
            user_id: &Uuid,
        ) -> Result<()>;
        async fn get_applications_filters_options(
            &self,
            employer_id: &Uuid,
        ) -> Result<crate::templates::dashboard::employer::applications::FiltersOptions>;
        async fn get_employer(
            &self,
            employer_id: &Uuid,
        ) -> Result<crate::templates::dashboard::employer::employers::Employer>;
        async fn get_job_dashboard(
            &self,
            job_id: &Uuid,
        ) -> Result<crate::templates::dashboard::employer::jobs::Job>;
        async fn get_job_seeker_user_id(
            &self,
            job_seeker_profile_id: &Uuid,
        ) -> Result<Option<Uuid>>;
        async fn get_job_stats(
            &self,
            job_id: &Uuid,
        ) -> Result<crate::templates::dashboard::employer::jobs::JobStats>;
        async fn get_user_invitations_count(&self, user_id: &Uuid) -> Result<usize>;
        async fn list_certifications(
            &self,
        ) -> Result<Vec<crate::templates::misc::Certification>>;
        async fn list_employer_jobs(
            &self,
            employer_id: &Uuid,
        ) -> Result<Vec<crate::templates::dashboard::employer::jobs::JobSummary>>;
        async fn list_employers(
            &self,
            user_id: &Uuid,
        ) -> Result<Vec<crate::templates::dashboard::employer::employers::EmployerSummary>>;
        async fn list_foundations(
            &self,
        ) -> Result<Vec<crate::templates::misc::Foundation>>;
        async fn list_team_members(
            &self,
            employer_id: &Uuid,
        ) -> Result<Vec<crate::templates::dashboard::employer::team::TeamMember>>;
        async fn list_user_invitations(
            &self,
            user_id: &Uuid,
        ) -> Result<Vec<crate::templates::dashboard::employer::team::TeamInvitation>>;
        async fn publish_job(&self, job_id: &Uuid) -> Result<()>;
        async fn search_applications(
            &self,
            employer_id: &Uuid,
            filters: &crate::templates::dashboard::employer::applications::Filters,
        ) -> Result<crate::db::dashboard::employer::ApplicationsSearchOutput>;
        async fn update_employer(
            &self,
            employer_id: &Uuid,
            employer: &crate::templates::dashboard::employer::employers::Employer,
        ) -> Result<()>;
        async fn update_job(
            &self,
            job_id: &Uuid,
            job: &crate::templates::dashboard::employer::jobs::Job,
        ) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::dashboard::job_seeker::DBDashBoardJobSeeker for DB {
        async fn cancel_application(
            &self,
            application_id: &Uuid,
            user_id: &Uuid,
        ) -> Result<()>;
        async fn get_job_seeker_profile(
            &self,
            user_id: &Uuid,
        ) -> Result<Option<crate::templates::dashboard::job_seeker::profile::JobSeekerProfile>>;
        async fn list_job_seeker_applications(
            &self,
            user_id: &Uuid,
        ) -> Result<Vec<crate::templates::dashboard::job_seeker::applications::Application>>;
        async fn update_job_seeker_profile(
            &self,
            user_id: &Uuid,
            profile: &crate::templates::dashboard::job_seeker::profile::JobSeekerProfile,
        ) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::dashboard::moderator::DBDashBoardModerator for DB {
        async fn approve_job(
            &self,
            job_id: &Uuid,
            reviewer: &Uuid,
        ) -> Result<Option<DateTime<Utc>>>;
        async fn list_jobs_for_moderation(
            &self,
            status: crate::templates::dashboard::employer::jobs::JobStatus,
        ) -> Result<Vec<crate::templates::dashboard::moderator::jobs::JobSummary>>;
        async fn reject_job(
            &self,
            job_id: &Uuid,
            reviewer: &Uuid,
            review_notes: Option<String>,
        ) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::event_tracker::DBEventTracker for DB {
        async fn update_jobs_views(
            &self,
            data: Vec<(
                crate::event_tracker::JobId,
                crate::event_tracker::Day,
                crate::event_tracker::Total,
            )>,
        ) -> Result<()>;
        async fn update_search_appearances(
            &self,
            data: Vec<(
                crate::event_tracker::JobId,
                crate::event_tracker::Day,
                crate::event_tracker::Total,
            )>,
        ) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::img::DBImage for DB {
        async fn get_image_version(
            &self,
            image_id: Uuid,
            version: &str,
        ) -> Result<Option<(Vec<u8>, crate::img::ImageFormat)>>;
        async fn save_image_versions(
            &self,
            user_id: &Uuid,
            versions: Vec<crate::img::ImageVersion>,
        ) -> Result<Uuid>;
    }

    #[async_trait]
    impl crate::db::jobboard::DBJobBoard for DB {
        async fn apply_to_job(&self, job_id: &Uuid, user_id: &Uuid) -> Result<bool>;
        async fn get_job_jobboard(
            &self,
            job_id: &Uuid,
        ) -> Result<Option<crate::templates::jobboard::jobs::Job>>;
        async fn get_jobs_filters_options(
            &self,
        ) -> Result<crate::templates::jobboard::jobs::FiltersOptions>;
        async fn get_stats(&self) -> Result<crate::templates::jobboard::stats::Stats>;
        async fn search_jobs(
            &self,
            filters: &crate::templates::jobboard::jobs::Filters,
        ) -> Result<crate::db::jobboard::JobsSearchOutput>;
    }

    #[async_trait]
    impl crate::db::misc::DBMisc for DB {
        async fn search_locations(
            &self,
            ts_query: &str,
        ) -> Result<Vec<crate::templates::misc::Location>>;
        async fn search_members(
            &self,
            foundation: &str,
            member: &str,
        ) -> Result<Vec<crate::templates::misc::Member>>;
        async fn search_projects(
            &self,
            foundation: &str,
            project: &str,
        ) -> Result<Vec<crate::templates::misc::Project>>;
    }

    #[async_trait]
    impl crate::db::notifications::DBNotifications for DB {
        async fn enqueue_notification(
            &self,
            notification: &crate::notifications::NewNotification,
        ) -> Result<()>;
        async fn get_pending_notification(
            &self,
            client_id: Uuid,
        ) -> Result<Option<crate::notifications::Notification>>;
        async fn update_notification(
            &self,
            client_id: Uuid,
            notification: &crate::notifications::Notification,
            error: Option<String>,
        ) -> Result<()>;
    }

    #[async_trait]
    impl crate::db::workers::DBWorkers for DB {
        async fn archive_expired_jobs(&self) -> Result<()>;
    }
}
