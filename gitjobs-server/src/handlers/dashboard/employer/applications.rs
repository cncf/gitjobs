//! This module defines the HTTP handlers for the applications page.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde_qs::axum::QsQuery;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    db::{DynDB, dashboard::employer::ApplicationsSearchOutput},
    handlers::{error::HandlerError, extractors::SelectedEmployerIdRequired},
    templates::{
        dashboard::{
            employer::applications::{ApplicationsPage, Filters},
            job_seeker,
        },
        pagination::NavigationLinks,
    },
};

// Pages handlers.

/// Renders the applications list page for the selected employer.
#[instrument(skip_all, err)]
pub(crate) async fn list_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    QsQuery(filters): QsQuery<Filters>,
) -> Result<impl IntoResponse, HandlerError> {
    // Get filter options and applications that match the query
    let (filters_options, ApplicationsSearchOutput { applications, total }) = tokio::try_join!(
        db.get_applications_filters_options(&employer_id),
        db.search_applications(&employer_id, &filters)
    )?;

    // Prepare template
    let navigation_links = NavigationLinks::from_filters(&filters, total)?;
    let template = ApplicationsPage {
        applications,
        filters,
        filters_options,
        navigation_links,
    };

    Ok(Html(template.render()?))
}

/// Renders the page to preview a job seeker's profile for employers.
#[instrument(skip_all, err)]
pub(crate) async fn profile_preview_page(
    State(db): State<DynDB>,
    Path(profile_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    let Some(user_id) = db.get_job_seeker_user_id(&profile_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let Some(profile) = db.get_job_seeker_profile(&user_id).await? else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let template = job_seeker::profile::PreviewPage { profile };

    Ok(Html(template.render()?).into_response())
}

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::{Path, State},
        http::{Request, header::COOKIE},
        response::IntoResponse,
    };
    use axum_login::tower_sessions::session;
    use reqwest::StatusCode;
    use serde_qs::axum::QsQuery;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_employer_applications_filters_options,
            sample_employer_applications_output, sample_job_seeker_profile, sample_session_record,
        },
        notifications::MockNotificationsManager,
        templates::dashboard::employer::applications::Filters,
    };

    use super::*;

    #[tokio::test]
    async fn test_list_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();
        let job_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_applications_filters_options()
            .times(1)
            .withf(move |id| *id == employer_id)
            .returning(move |_| Ok(sample_employer_applications_filters_options(job_id)));
        db.expect_search_applications()
            .times(1)
            .withf(move |id, _| *id == employer_id)
            .returning(|_, _| Ok(sample_employer_applications_output()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = list_page(
            State(db),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
            QsQuery(Filters::default()),
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_profile_preview_page_returns_not_found_when_profile_is_missing() {
        // Setup identifiers and data structures
        let profile_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_job_seeker_user_id()
            .times(1)
            .withf(move |id| *id == profile_id)
            .returning(|_| Ok(None));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = profile_preview_page(State(db), Path(profile_id))
            .await
            .unwrap()
            .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_profile_preview_page_renders_successfully() {
        // Setup identifiers and data structures
        let profile_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_job_seeker_user_id()
            .times(1)
            .withf(move |id| *id == profile_id)
            .returning(move |_| Ok(Some(user_id)));
        db.expect_get_job_seeker_profile()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(|_| Ok(Some(sample_job_seeker_profile())));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = profile_preview_page(State(db), Path(profile_id))
            .await
            .unwrap()
            .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_profile_preview_route_returns_forbidden_when_access_is_denied() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let selected_employer_id = Uuid::new_v4();
        let profile_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record =
            sample_session_record(session_id, user_id, auth_hash, Some(selected_employer_id));

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_session()
            .times(1)
            .withf(move |id| *id == session_id)
            .returning(move |_| Ok(Some(session_record.clone())));
        db.expect_get_user_by_id()
            .times(1)
            .withf(move |id| *id == user_id)
            .returning(move |_| Ok(Some(sample_auth_user(user_id, auth_hash))));
        db.expect_user_owns_employer()
            .times(1)
            .withf(move |id, employer| *id == user_id && *employer == selected_employer_id)
            .returning(|_, _| Ok(true));
        db.expect_user_has_profile_access()
            .times(1)
            .withf(move |id, profile| *id == user_id && *profile == profile_id)
            .returning(|_, _| Ok(false));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri(format!(
                "/dashboard/employer/applications/profile/{profile_id}/preview"
            ))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
    }
}
