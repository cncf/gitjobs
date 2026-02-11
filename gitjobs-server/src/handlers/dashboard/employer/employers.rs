//! This module defines the HTTP handlers to manage employers.

use anyhow::Result;
use askama::Template;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse},
};
use axum_messages::Messages;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::AuthSession,
    db::DynDB,
    handlers::{auth::SELECTED_EMPLOYER_ID_KEY, error::HandlerError, extractors::SelectedEmployerIdRequired},
    templates::dashboard::employer::employers::{self, Employer},
};

// Pages handlers.

/// Displays the page to add a new employer.
#[instrument(skip_all, err)]
pub(crate) async fn add_page(State(db): State<DynDB>) -> Result<impl IntoResponse, HandlerError> {
    let foundations = db.list_foundations().await?;
    let template = employers::AddPage { foundations };

    Ok(Html(template.render()?))
}

/// Displays the page to update an employer.
#[instrument(skip_all, err)]
pub(crate) async fn update_page(
    State(db): State<DynDB>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
) -> Result<impl IntoResponse, HandlerError> {
    let employer = db.get_employer(&employer_id).await?;
    let foundations = db.list_foundations().await?;
    let template = employers::UpdatePage {
        employer,
        foundations,
    };

    Ok(Html(template.render()?))
}

// Actions handlers.

/// Adds a new employer to the database and sets it as selected in the session.
#[instrument(skip_all, err)]
pub(crate) async fn add(
    auth_session: AuthSession,
    messages: Messages,
    session: Session,
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get user from session
    let Some(user) = auth_session.user else {
        return Ok((StatusCode::FORBIDDEN).into_response());
    };

    // Get employer information from body
    let employer: Employer = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };

    // Add employer to database
    let employer_id = db.add_employer(&user.user_id, &employer).await?;
    messages.success("Employer added successfully.");

    // Use new employer as the selected employer for the session
    session.insert(SELECTED_EMPLOYER_ID_KEY, employer_id).await?;

    Ok((
        StatusCode::CREATED,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/employer", "target":"body"}"#,
        )],
    )
        .into_response())
}

/// Sets the selected employer in the session for the current user.
#[instrument(skip_all, err)]
pub(crate) async fn select(
    session: Session,
    Path(employer_id): Path<Uuid>,
) -> Result<impl IntoResponse, HandlerError> {
    // Update the selected employer in the session
    session.insert(SELECTED_EMPLOYER_ID_KEY, employer_id).await?;

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/employer", "target":"body"}"#,
        )],
    )
        .into_response())
}

/// Updates an existing employer's information in the database.
#[instrument(skip_all, err)]
pub(crate) async fn update(
    messages: Messages,
    State(db): State<DynDB>,
    State(serde_qs_de): State<serde_qs::Config>,
    SelectedEmployerIdRequired(employer_id): SelectedEmployerIdRequired,
    body: String,
) -> Result<impl IntoResponse, HandlerError> {
    // Get employer information from body
    let employer: Employer = match serde_qs_de.deserialize_str(&body).map_err(anyhow::Error::new) {
        Ok(profile) => profile,
        Err(e) => return Ok((StatusCode::UNPROCESSABLE_ENTITY, e.to_string()).into_response()),
    };

    // Update employer in database
    db.update_employer(&employer_id, &employer).await?;
    messages.success("Employer updated successfully.");

    Ok((
        StatusCode::NO_CONTENT,
        [(
            "HX-Location",
            r#"{"path":"/dashboard/employer?tab=profile", "target":"body"}"#,
        )],
    )
        .into_response())
}

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        body::Body,
        extract::State,
        http::{Request, StatusCode, header::COOKIE},
        response::IntoResponse,
    };
    use axum_login::tower_sessions::session;
    use tower::ServiceExt;
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        handlers::tests::{
            TestRouterBuilder, sample_auth_user, sample_employer, sample_foundations, sample_session_record,
        },
        notifications::MockNotificationsManager,
    };

    use super::*;

    #[tokio::test]
    async fn test_add_page_renders_successfully() {
        // Setup database mock
        let mut db = MockDB::new();
        db.expect_list_foundations()
            .times(1)
            .returning(|| Ok(sample_foundations()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = add_page(State(db)).await.unwrap().into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_update_page_renders_successfully() {
        // Setup identifiers and data structures
        let employer_id = Uuid::new_v4();

        // Setup database mock
        let mut db = MockDB::new();
        db.expect_get_employer()
            .times(1)
            .withf(move |id| *id == employer_id)
            .returning(move |_| Ok(sample_employer(employer_id)));
        db.expect_list_foundations()
            .times(1)
            .returning(|| Ok(sample_foundations()));
        let db: DynDB = Arc::new(db);

        // Execute handler
        let response = update_page(
            State(db),
            crate::handlers::extractors::SelectedEmployerIdRequired(employer_id),
        )
        .await
        .unwrap()
        .into_response();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_add_creates_employer_and_returns_created() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let created_employer_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);
        let mut employer = sample_employer(Uuid::new_v4());
        employer.members = None;
        let body = serde_qs::to_string(&employer).unwrap();

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
        db.expect_add_employer()
            .times(1)
            .returning(move |_, _| Ok(created_employer_id));
        db.expect_update_session().times(1..).returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("POST")
            .uri("/dashboard/employer/employers/add")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::CREATED);
    }

    #[tokio::test]
    async fn test_select_updates_session_and_returns_no_content() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, None);

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
            .withf(move |id, employer| *id == user_id && *employer == employer_id)
            .returning(|_, _| Ok(true));
        db.expect_update_session().times(1..).returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri(format!("/dashboard/employer/employers/{employer_id}/select"))
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }

    #[tokio::test]
    async fn test_update_updates_employer_and_returns_no_content() {
        // Setup identifiers and data structures
        let auth_hash = "hash";
        let employer_id = Uuid::new_v4();
        let session_id = session::Id::default();
        let user_id = Uuid::new_v4();
        let session_record = sample_session_record(session_id, user_id, auth_hash, Some(employer_id));
        let mut employer = sample_employer(employer_id);
        employer.members = None;
        let body = serde_qs::to_string(&employer).unwrap();

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
        db.expect_update_employer()
            .times(1)
            .withf(move |id, _| *id == employer_id)
            .returning(|_, _| Ok(()));
        db.expect_update_session().times(1..).returning(|_| Ok(()));

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .build()
            .await;
        let request = Request::builder()
            .method("PUT")
            .uri("/dashboard/employer/employers/update")
            .header(COOKIE, format!("id={session_id}"))
            .body(Body::from(body))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        // Check response matches expectations
        assert_eq!(response.status(), StatusCode::NO_CONTENT);
    }
}
