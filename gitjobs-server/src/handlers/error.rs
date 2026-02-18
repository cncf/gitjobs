//! This module defines a `HandlerError` type to make error propagation easier
//! in handlers.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use tokio_postgres::error::SqlState;

/// Represents all possible errors that can occur in a handler.
#[derive(thiserror::Error, Debug)]
pub(crate) enum HandlerError {
    /// Error related to authentication, contains a message.
    #[error("auth error: {0}")]
    Auth(String),

    /// Database error with user-facing message.
    #[error("database error: {0}")]
    Database(String),

    /// Any other error, wrapped in `anyhow::Error` for flexibility.
    #[error(transparent)]
    Other(anyhow::Error),

    /// Error during JSON serialization or deserialization.
    #[error("serde json error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Error related to session management.
    #[error("session error: {0}")]
    Session(#[from] tower_sessions::session::Error),

    /// Error during template rendering.
    #[error("template error: {0}")]
    Template(#[from] askama::Error),
}

/// Enables conversion of `HandlerError` into an HTTP response for Axum handlers.
impl IntoResponse for HandlerError {
    fn into_response(self) -> Response {
        match self {
            HandlerError::Auth(_) => StatusCode::UNAUTHORIZED.into_response(),
            HandlerError::Database(msg) => (StatusCode::UNPROCESSABLE_ENTITY, msg).into_response(),
            HandlerError::Other(_)
            | HandlerError::Serde(_)
            | HandlerError::Session(_)
            | HandlerError::Template(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}

impl From<anyhow::Error> for HandlerError {
    fn from(err: anyhow::Error) -> Self {
        if let Some(msg) = extract_db_error_message(&err) {
            return HandlerError::Database(msg);
        }

        HandlerError::Other(err)
    }
}

/// Extracts user-facing message from P0001 (RAISE EXCEPTION) database errors.
fn extract_db_error_message(err: &anyhow::Error) -> Option<String> {
    let pg_err = err.downcast_ref::<tokio_postgres::Error>()?;
    let db_err = pg_err.as_db_error()?;

    if db_err.code() == &SqlState::RAISE_EXCEPTION {
        return Some(db_err.message().to_string());
    }

    None
}

#[cfg(test)]
mod tests {
    use axum::body::to_bytes;

    use super::*;

    #[tokio::test]
    async fn test_auth_error_returns_401() {
        let error = HandlerError::Auth("authentication failed".to_string());
        let response = error.into_response();
        let parts = response.into_parts().0;

        assert_eq!(parts.status, StatusCode::UNAUTHORIZED);
    }

    #[tokio::test]
    async fn test_database_error_returns_422_with_message() {
        let message = "team member not found";
        let error = HandlerError::Database(message.to_string());
        let response = error.into_response();
        let (parts, body) = response.into_parts();
        let bytes = to_bytes(body, usize::MAX).await.unwrap();

        assert_eq!(parts.status, StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(bytes.as_ref(), message.as_bytes());
    }

    #[tokio::test]
    async fn test_non_db_anyhow_error_returns_500() {
        let error: HandlerError = anyhow::anyhow!("some internal error").into();
        let response = error.into_response();
        let parts = response.into_parts().0;

        assert_eq!(parts.status, StatusCode::INTERNAL_SERVER_ERROR);
    }
}
