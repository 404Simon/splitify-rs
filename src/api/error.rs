use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use leptos::prelude::ServerFnError;
use serde::Serialize;

/// Error type returned by REST API handlers. Rendered as `{ "error": "..." }`.
#[derive(Debug)]
pub struct ApiError {
    status: StatusCode,
    message: String,
}

#[derive(Serialize)]
struct ErrorBody {
    error: String,
}

impl ApiError {
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            message: message.into(),
        }
    }

    pub fn unauthorized(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            message: message.into(),
        }
    }

    pub fn forbidden(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            message: message.into(),
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            message: message.into(),
        }
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            message: message.into(),
        }
    }

    pub(crate) fn server_message(err: &ServerFnError) -> String {
        server_fn_message(err)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ErrorBody {
                error: self.message,
            }),
        )
            .into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!(error = %err, "API database error");
        Self::internal("Internal server error")
    }
}

impl From<ServerFnError> for ApiError {
    fn from(err: ServerFnError) -> Self {
        let message = server_fn_message(&err);
        let lower = message.to_ascii_lowercase();

        let status = if lower.contains("not authenticated") {
            StatusCode::UNAUTHORIZED
        } else if message.starts_with("Unauthorized")
            || lower.contains("permission")
            || lower.contains("not a member")
            || lower.contains("only the creator")
        {
            StatusCode::FORBIDDEN
        } else if lower.contains("not found") {
            StatusCode::NOT_FOUND
        } else {
            StatusCode::BAD_REQUEST
        };

        Self { status, message }
    }
}

fn server_fn_message(err: &ServerFnError) -> String {
    match err {
        ServerFnError::Registration(message)
        | ServerFnError::Request(message)
        | ServerFnError::Response(message)
        | ServerFnError::ServerError(message)
        | ServerFnError::MiddlewareError(message)
        | ServerFnError::Deserialization(message)
        | ServerFnError::Serialization(message)
        | ServerFnError::Args(message)
        | ServerFnError::MissingArg(message) => message.clone(),
        _ => err.to_string(),
    }
}
