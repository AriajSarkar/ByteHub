use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("database error: {0}")]
    Database(String),
    #[error("invalid signature")]
    InvalidSignature,
    #[error("invalid payload: {0}")]
    InvalidPayload(String),
    #[error("project already exists: {0}")]
    ProjectAlreadyExists(String),
    #[error("not found: {0}")]
    NotFound(String),
    #[error("unauthorized")]
    Unauthorized,
    #[error("discord api error: {0}")]
    Discord(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Error::InvalidSignature | Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::InvalidPayload(_) => StatusCode::BAD_REQUEST,
            Error::ProjectAlreadyExists(_) => StatusCode::CONFLICT,
            Error::NotFound(_) => StatusCode::NOT_FOUND,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.to_string()).into_response()
    }
}

pub type Result<T> = std::result::Result<T, Error>;
