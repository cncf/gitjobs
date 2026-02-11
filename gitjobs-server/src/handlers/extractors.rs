//! Custom extractors for handlers.

use std::sync::Arc;

use anyhow::Result;
use axum::{
    extract::{FromRequestParts, Path},
    http::{StatusCode, request::Parts},
};
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

// Tests.

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{
        Router,
        body::Body,
        extract::Path,
        http::{Request, StatusCode, header::SET_COOKIE},
        response::IntoResponse,
        routing::{get, post},
    };
    use tower::ServiceExt;
    use tower_sessions::{MemoryStore, Session, SessionManagerLayer};
    use uuid::Uuid;

    use crate::{
        db::{DynDB, mock::MockDB},
        event_tracker::{DynEventTracker, MockEventTracker},
        handlers::auth::SELECTED_EMPLOYER_ID_KEY,
        handlers::tests::{qs_config, test_http_server_cfg},
        img::{DynImageStore, MockImageStore},
        notifications::{DynNotificationsManager, MockNotificationsManager},
    };

    use super::*;

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

    // Helpers.

    fn build_state(
        db: DynDB,
        image_store: DynImageStore,
        notifications_manager: DynNotificationsManager,
    ) -> router::State {
        let event_tracker: DynEventTracker = Arc::new(MockEventTracker::new());

        router::State {
            cfg: test_http_server_cfg(),
            db,
            image_store,
            serde_qs_de: qs_config(),
            notifications_manager,
            event_tracker,
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
