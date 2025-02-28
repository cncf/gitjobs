//! This module defines a `HandlerError` type to make error propagation easier
//! in handlers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Possible errors that can occur in a handler.
#[derive(thiserror::Error, Debug)]
pub(crate) enum HandlerError {
    #[error("auth error: {0}")]
    Auth(String),

    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::Error),

    #[error("template error: {0}")]
    Template(#[from] rinja::Error),

    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Allows to convert a `HandlerError` into a `Response`.
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
