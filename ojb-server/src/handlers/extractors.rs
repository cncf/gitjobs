//! Custom extractors for handlers.

use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{header::HOST, request::Parts, StatusCode},
};
use cached::proc_macro::cached;
use tower_sessions::Session;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{db::DynDB, router};

/// Custom extractor to get the job board id from the request's host header.
pub(crate) struct JobBoardId(pub Uuid);

impl FromRequestParts<router::State> for JobBoardId {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        // Extract host from the request headers
        let Some(host_header) = parts.headers.get(HOST) else {
            return Err((StatusCode::BAD_REQUEST, "missing host header"));
        };
        let host = host_header
            .to_str()
            .unwrap_or_default()
            .split(':')
            .next()
            .unwrap_or_default();

        // Lookup the job board id in the database
        let Some(job_board_id) = lookup_job_board_id(state.db.clone(), host).await.map_err(|err| {
            error!(?err, "error looking up job board id");
            (StatusCode::INTERNAL_SERVER_ERROR, "")
        })?
        else {
            return Err((StatusCode::BAD_REQUEST, "job board host not found"));
        };

        Ok(JobBoardId(job_board_id))
    }
}

/// Lookup the job board id in the database using the host provided.
#[cached(
    time = 86400,
    key = "String",
    convert = r#"{ String::from(host) }"#,
    sync_writes = true,
    result = true
)]
#[instrument(skip(db), err)]
async fn lookup_job_board_id(db: DynDB, host: &str) -> Result<Option<Uuid>> {
    if host.is_empty() {
        return Ok(None);
    }
    db.get_job_board_id(host).await
}

/// Key to store the selected employer id in the session.
pub(crate) const SELECTED_EMPLOYER_ID_KEY: &str = "selected_employer_id";

/// Custom extractor to get the employer id from the request's query string.
pub(crate) struct EmployerId(pub Uuid);

impl FromRequestParts<router::State> for EmployerId {
    type Rejection = (StatusCode, &'static str);

    #[instrument(skip_all, err(Debug))]
    async fn from_request_parts(parts: &mut Parts, state: &router::State) -> Result<Self, Self::Rejection> {
        let Ok(session) = Session::from_request_parts(parts, state).await else {
            return Err((StatusCode::UNAUTHORIZED, "user not logged in"));
        };
        let Ok(Some(employer_id)) = session.get(SELECTED_EMPLOYER_ID_KEY).await else {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                "selected employer_id not found in session",
            ));
        };
        Ok(EmployerId(employer_id))
    }
}
