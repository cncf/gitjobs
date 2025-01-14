//! Custom extractors for handlers.

use anyhow::Result;
use axum::{
    extract::FromRequestParts,
    http::{header::HOST, request::Parts, StatusCode},
};
use cached::proc_macro::cached;
use tracing::{error, instrument};
use uuid::Uuid;

use crate::{db::DynDB, router};

/// Custom extractor to get the board id from the request's host header.
pub(crate) struct BoardId(pub Uuid);

impl FromRequestParts<router::State> for BoardId {
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

        // Lookup the board id in the database
        let Some(board_id) = lookup_board_id(state.db.clone(), host).await.map_err(|err| {
            error!(?err, "error looking up board id");
            (StatusCode::INTERNAL_SERVER_ERROR, "")
        })?
        else {
            return Err((StatusCode::BAD_REQUEST, "board host not found"));
        };

        Ok(BoardId(board_id))
    }
}

/// Lookup the board id in the database using the host provided.
#[cached(
    time = 86400,
    key = "String",
    convert = r#"{ String::from(host) }"#,
    sync_writes = true,
    result = true
)]
#[instrument(skip(db), err)]
async fn lookup_board_id(db: DynDB, host: &str) -> Result<Option<Uuid>> {
    if host.is_empty() {
        return Ok(None);
    }
    db.get_board_id(host).await
}
