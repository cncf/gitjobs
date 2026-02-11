//! Custom extractors for handlers.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    Form,
    extract::{FromRequest, FromRequestParts, Path, Request},
    http::{StatusCode, request::Parts},
};
use garde::Validate;
use serde::de::DeserializeOwned;
use tower_sessions::Session;
use tracing::instrument;
use uuid::Uuid;

use crate::{
    auth::{AuthSession, OAuth2ProviderDetails, OidcProviderDetails},
    config::{OAuth2Provider, OidcProvider},
    handlers::auth::SELECTED_EMPLOYER_ID_KEY,
    router,
};

/// Extractor for `OAuth2` provider details from the authenticated session.
pub(crate) struct OAuth2(pub Arc<OAuth2ProviderDetails>);

impl FromRequestParts<router::State> for OAuth2 {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        let Ok(provider) = Path::<OAuth2Provider>::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "missing oauth2 provider"));
        };
        let Ok(auth_session) = AuthSession::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "missing auth session"));
        };
        let Some(provider_details) = auth_session.backend.oauth2_providers.get(&provider) else {
            return Err((StatusCode::BAD_REQUEST, "oauth2 provider not supported"));
        };
        Ok(OAuth2(provider_details.clone()))
    }
}

/// Extractor for `Oidc` provider details from the authenticated session.
pub(crate) struct Oidc(pub Arc<OidcProviderDetails>);

impl FromRequestParts<router::State> for Oidc {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        let Ok(provider) = Path::<OidcProvider>::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "missing oidc provider"));
        };
        let Ok(auth_session) = AuthSession::from_request_parts(parts, state).await else {
            return Err((StatusCode::BAD_REQUEST, "missing auth session"));
        };
        let Some(provider_details) = auth_session.backend.oidc_providers.get(&provider) else {
            return Err((StatusCode::BAD_REQUEST, "oidc provider not supported"));
        };
        Ok(Oidc(provider_details.clone()))
    }
}

/// Extractor for the selected employer id from the session, as an Option.
/// Returns Some(Uuid) if present, or None if not set in the session.
pub(crate) struct SelectedEmployerIdOptional(pub Option<Uuid>);

impl FromRequestParts<router::State> for SelectedEmployerIdOptional {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        let Ok(session) = Session::from_request_parts(parts, state).await else {
            return Err((StatusCode::UNAUTHORIZED, "user not logged in"));
        };
        let Ok(employer_id) = session.get(SELECTED_EMPLOYER_ID_KEY).await else {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "error getting selected employer from session",
            ));
        };
        Ok(SelectedEmployerIdOptional(employer_id))
    }
}

/// Extractor for the selected employer id from the session, required variant.
/// Returns the Uuid if present, or an error if not found in the session.
pub(crate) struct SelectedEmployerIdRequired(pub Uuid);

impl FromRequestParts<router::State> for SelectedEmployerIdRequired {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        match SelectedEmployerIdOptional::from_request_parts(parts, state).await {
            Ok(SelectedEmployerIdOptional(Some(employer_id))) => Ok(SelectedEmployerIdRequired(employer_id)),
            Ok(SelectedEmployerIdOptional(None)) => Err((StatusCode::BAD_REQUEST, "missing employer id")),
            Err(err) => Err(err),
        }
    }
}

/// Extractor that deserializes and validates form data using Axum's `Form`.
pub(crate) struct ValidatedForm<T>(pub T);

impl<T> FromRequest<router::State> for ValidatedForm<T>
where
    T: DeserializeOwned + Validate,
    T::Context: Default,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &router::State) -> Result<Self, Self::Rejection> {
        // Deserialize form data
        let Form(value) = Form::<T>::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        // Validate the deserialized value
        value
            .validate()
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        Ok(Self(value))
    }
}

/// Extractor that deserializes and validates form data using `serde_qs`.
pub(crate) struct ValidatedFormQs<T>(pub T);

impl<T> FromRequest<router::State> for ValidatedFormQs<T>
where
    T: DeserializeOwned + Validate,
    T::Context: Default,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: Request, state: &router::State) -> Result<Self, Self::Rejection> {
        // Read body as string
        let body = String::from_request(req, state)
            .await
            .map_err(|e| (StatusCode::BAD_REQUEST, e.to_string()))?;

        // Deserialize using serde_qs
        let value: T = state
            .serde_qs_de
            .deserialize_str(&body)
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        // Validate the deserialized value
        value
            .validate()
            .map_err(|e| (StatusCode::UNPROCESSABLE_ENTITY, e.to_string()))?;

        Ok(Self(value))
    }
}

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::{Body, to_bytes},
        extract::Path,
        http::{Request, StatusCode, header::SET_COOKIE},
        response::IntoResponse,
        routing::{get, post},
    };
    use garde::Validate;
    use serde::Deserialize;
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        event_tracker::{DynEventTracker, MockEventTracker},
        handlers::auth::SELECTED_EMPLOYER_ID_KEY,
        handlers::tests::{TestRouterBuilder, qs_config, test_http_server_cfg},
        img::{DynImageStore, MockImageStore},
        notifications::{DynNotificationsManager, MockNotificationsManager},
        validation::{MAX_LEN_S, trimmed_non_empty, trimmed_non_empty_vec},
    };

    use super::*;

    #[tokio::test]
    async fn test_oauth2_extractor_returns_bad_request_when_provider_is_missing() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route("/oauth2", post(|_provider: OAuth2| async { StatusCode::OK }))
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/oauth2")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_oauth2_extractor_returns_bad_request_when_provider_is_unsupported() {
        // Setup config and database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.github = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oauth2/github")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_oidc_extractor_returns_bad_request_when_provider_is_missing() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route("/oidc", post(|_provider: Oidc| async { StatusCode::OK }))
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/oidc")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_oidc_extractor_returns_bad_request_when_provider_is_unsupported() {
        // Setup config and database mock
        let mut cfg = test_http_server_cfg();
        cfg.login.linuxfoundation = true;

        let mut db = MockDB::new();
        allow_session_store_updates(&mut db);

        // Setup router and send request
        let router = TestRouterBuilder::new(db, MockNotificationsManager::new())
            .with_cfg(cfg)
            .build()
            .await;
        let request = Request::builder()
            .method("GET")
            .uri("/log-in/oidc/linuxfoundation")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_selected_employer_id_optional_returns_id_when_set() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/optional",
                get(
                    |SelectedEmployerIdOptional(id): SelectedEmployerIdOptional| async move {
                        id.map_or_else(|| "none".to_string(), |value| value.to_string())
                    },
                ),
            )
            .route(
                "/set/{id}",
                post(|session: Session, Path(id): Path<Uuid>| async move {
                    session.insert(SELECTED_EMPLOYER_ID_KEY, id).await.unwrap();
                    StatusCode::NO_CONTENT.into_response()
                }),
            )
            .with_state(state)
            .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false));
        let employer_id = Uuid::new_v4();

        // Set selected employer in session
        let set_request = Request::builder()
            .method("POST")
            .uri(format!("/set/{employer_id}"))
            .body(Body::empty())
            .unwrap();
        let set_response = router.clone().oneshot(set_request).await.unwrap();
        let cookie = session_cookie(&set_response);

        // Send request and check response
        let request = Request::builder()
            .method("GET")
            .uri("/optional")
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(body.as_ref(), employer_id.to_string().as_bytes());
    }

    #[tokio::test]
    async fn test_selected_employer_id_optional_returns_none_when_not_set() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/optional",
                get(
                    |SelectedEmployerIdOptional(id): SelectedEmployerIdOptional| async move {
                        id.map_or_else(|| "none".to_string(), |value| value.to_string())
                    },
                ),
            )
            .with_state(state)
            .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false));

        // Send request and check response
        let request = Request::builder()
            .method("GET")
            .uri("/optional")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(body.as_ref(), b"none");
    }

    #[tokio::test]
    async fn test_selected_employer_id_required_returns_bad_request_when_not_set() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/required",
                get(|SelectedEmployerIdRequired(id): SelectedEmployerIdRequired| async move {
                    id.to_string()
                }),
            )
            .with_state(state)
            .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false));

        // Send request and check response
        let request = Request::builder()
            .method("GET")
            .uri("/required")
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    }

    #[tokio::test]
    async fn test_selected_employer_id_required_returns_id_when_set() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/required",
                get(
                    |SelectedEmployerIdRequired(id): SelectedEmployerIdRequired| async move {
                        id.to_string()
                    },
                ),
            )
            .route(
                "/set/{id}",
                post(|session: Session, Path(id): Path<Uuid>| async move {
                    session.insert(SELECTED_EMPLOYER_ID_KEY, id).await.unwrap();
                    StatusCode::NO_CONTENT.into_response()
                }),
            )
            .with_state(state)
            .layer(SessionManagerLayer::new(MemoryStore::default()).with_secure(false));
        let employer_id = Uuid::new_v4();

        // Set selected employer in session
        let set_request = Request::builder()
            .method("POST")
            .uri(format!("/set/{employer_id}"))
            .body(Body::empty())
            .unwrap();
        let set_response = router.clone().oneshot(set_request).await.unwrap();
        let cookie = session_cookie(&set_response);

        // Send request and check response
        let request = Request::builder()
            .method("GET")
            .uri("/required")
            .header("cookie", cookie)
            .body(Body::empty())
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX).await.unwrap();

        assert_eq!(body.as_ref(), employer_id.to_string().as_bytes());
    }

    #[tokio::test]
    async fn test_validated_form_qs_returns_unprocessable_entity_for_invalid_body() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form-qs",
                post(|ValidatedFormQs(_form): ValidatedFormQs<TestFormQs>| async move {
                    StatusCode::OK.into_response()
                }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form-qs")
            .body(Body::from("name=test&tags[abc]=rust"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_validated_form_qs_returns_unprocessable_entity_for_invalid_form() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form-qs",
                post(|ValidatedFormQs(_form): ValidatedFormQs<TestFormQs>| async move {
                    StatusCode::OK.into_response()
                }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form-qs")
            .body(Body::from("name=test&tags[0]=+"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_validated_form_qs_succeeds_for_valid_form() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form-qs",
                post(|ValidatedFormQs(form): ValidatedFormQs<TestFormQs>| async move {
                    form.name.into_response()
                }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form-qs")
            .body(Body::from("name=test&tags[0]=rust&tags[1]=oss"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(bytes.as_ref(), b"test");
    }

    #[tokio::test]
    async fn test_validated_form_returns_unprocessable_entity_for_invalid_body() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form",
                post(|ValidatedForm(_form): ValidatedForm<TestForm>| async move {
                    StatusCode::OK.into_response()
                }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from(""))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_validated_form_returns_unprocessable_entity_for_invalid_form() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form",
                post(|ValidatedForm(_form): ValidatedForm<TestForm>| async move {
                    StatusCode::OK.into_response()
                }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("name=+"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
    }

    #[tokio::test]
    async fn test_validated_form_succeeds_for_valid_form() {
        // Setup state and router
        let db: DynDB = Arc::new(MockDB::new());
        let image_store: DynImageStore = Arc::new(MockImageStore::new());
        let notifications_manager: DynNotificationsManager = Arc::new(MockNotificationsManager::new());
        let state = build_state(db, image_store, notifications_manager);
        let router = Router::new()
            .route(
                "/form",
                post(|ValidatedForm(form): ValidatedForm<TestForm>| async move { form.name.into_response() }),
            )
            .with_state(state);

        // Send request and check response
        let request = Request::builder()
            .method("POST")
            .uri("/form")
            .header("content-type", "application/x-www-form-urlencoded")
            .body(Body::from("name=test"))
            .unwrap();
        let response = router.oneshot(request).await.unwrap();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        assert_eq!(parts.status, StatusCode::OK);
        assert_eq!(bytes.as_ref(), b"test");
    }

    // Helpers.

    #[derive(Debug, Deserialize, Validate)]
    struct TestForm {
        #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_S))]
        name: String,
    }

    #[derive(Debug, Deserialize, Validate)]
    struct TestFormQs {
        #[garde(custom(trimmed_non_empty), length(max = MAX_LEN_S))]
        name: String,
        #[garde(custom(trimmed_non_empty_vec))]
        tags: Option<Vec<String>>,
    }

    fn allow_session_store_updates(db: &mut MockDB) {
        db.expect_create_session().times(0..).returning(|_| Ok(()));
        db.expect_delete_session().times(0..).returning(|_| Ok(()));
        db.expect_update_session().times(0..).returning(|_| Ok(()));
    }

    fn build_state(
        db: DynDB,
        image_store: DynImageStore,
        notifications_manager: DynNotificationsManager,
    ) -> router::State {
        let event_tracker: DynEventTracker = Arc::new(MockEventTracker::new());

        router::State {
            cfg: test_http_server_cfg(),
            db,
            event_tracker,
            http_client: reqwest::Client::new(),
            image_store,
            notifications_manager,
            serde_qs_de: qs_config(),
        }
    }

    fn session_cookie(response: &axum::response::Response) -> String {
        response
            .headers()
            .get(SET_COOKIE)
            .and_then(|value| value.to_str().ok())
            .and_then(|set_cookie| set_cookie.split(';').next())
            .expect("set-cookie header to include session cookie")
            .to_string()
    }
}
