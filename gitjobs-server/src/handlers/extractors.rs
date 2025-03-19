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

/// Extractor to get the oauth2 provider from the auth session.
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

/// Extractor to get the oidc provider from the auth session.
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

/// Extractor to get the selected employer id from the session. It returns the
/// employer id as an option.
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

/// Extractor to get the selected employer id from the session. An error is
/// returned if the employer id is not found in the session.
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
