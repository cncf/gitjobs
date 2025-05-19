//! This module defines a `HandlerError` type to make error propagation easier
//! in handlers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

/// Represents all possible errors that can occur in a handler.
#[derive(thiserror::Error, Debug)]
pub(crate) enum HandlerError {
    /// Error related to authentication, contains a message.
    #[error("auth error: {0}")]
    Auth(String),

    /// Error during JSON serialization or deserialization.
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Error related to session management.
    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::Error),

    /// Error during template rendering.
    #[error("template error: {0}")]
    Template(#[from] askama::Error),

    /// Any other error, wrapped in `anyhow::Error` for flexibility.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

/// Enables conversion of `HandlerError` into an HTTP response for Axum handlers.
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}
