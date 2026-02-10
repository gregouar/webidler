use std::error::Error;
use std::fmt;

use axum::Json;
use axum::response::{IntoResponse, Response};
use http::StatusCode;

use shared::http::server::ErrorResponse;

#[derive(Debug)]
pub enum AppError {
    Database(sqlx::Error),
    Anyhow(anyhow::Error),
    UserError(String),
    Unauthorized(String),
    Forbidden,
    NotFound,
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Database(err) => write!(f, "Database error: {err}"), // TODO: remove details?
            AppError::Anyhow(err) => write!(f, "Unexpected error: {err}"), // TODO: remove details?
            AppError::UserError(err) => write!(f, "{err}"),
            AppError::Unauthorized(err) => write!(f, "{err}"),
            AppError::Forbidden => write!(f, "Forbidden"),
            AppError::NotFound => write!(f, "Not found"),
        }
    }
}

impl Error for AppError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            AppError::Database(err) => Some(err),
            AppError::Anyhow(err) => Some(err.root_cause()),
            AppError::NotFound
            | AppError::Unauthorized(_)
            | AppError::Forbidden
            | AppError::UserError(_) => None,
        }
    }
}

impl From<sqlx::Error> for AppError {
    fn from(e: sqlx::Error) -> Self {
        AppError::Database(e)
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        AppError::Anyhow(e)
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let code = match self {
            AppError::NotFound => StatusCode::NOT_FOUND,
            AppError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            AppError::Forbidden => StatusCode::FORBIDDEN,
            AppError::UserError(_) => StatusCode::CONFLICT,
            AppError::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(ErrorResponse {
            error: self.to_string(),
        });

        if code == StatusCode::INTERNAL_SERVER_ERROR {
            tracing::error!(?self, "responding with error");
        }

        (code, body).into_response()
    }
}
