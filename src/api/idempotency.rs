//! Idempotent mutation support for the REST API.
//!
//! Clients may send an `Idempotency-Key` header on any write request. The first
//! successful response for a `(token, key, method, path)` tuple is stored and
//! replayed for retries within seven days, which makes outbox retries safe after
//! a lost response.

use axum::{
    body::{Body, to_bytes},
    extract::{Request, State},
    http::{HeaderValue, Method, StatusCode, header},
    middleware::Next,
    response::{IntoResponse, Response},
};
use sqlx::SqlitePool;

use super::{
    error::ApiError,
    tokens::{hash_token, unix_timestamp},
};

const IDEMPOTENCY_HEADER: &str = "Idempotency-Key";
const REPLAYED_HEADER: &str = "Idempotent-Replayed";
const MAX_KEY_CHARS: usize = 200;
const MAX_REQUEST_BODY: usize = 1024 * 1024;
const MAX_RESPONSE_BODY: usize = 2 * 1024 * 1024;
const RETENTION_SECONDS: i64 = 7 * 24 * 60 * 60;

/// Axum middleware replaying stored responses for repeated idempotency keys.
pub async fn middleware(State(pool): State<SqlitePool>, request: Request, next: Next) -> Response {
    if !is_idempotent_request(request.method(), request.uri().path()) {
        return next.run(request).await;
    }

    let Some(key) = idempotency_key(&request) else {
        return next.run(request).await;
    };

    if key.chars().count() > MAX_KEY_CHARS {
        return ApiError::bad_request("Idempotency-Key must be 200 characters or fewer")
            .into_response();
    }

    // Without a bearer token there is nothing to scope the key to. Let the
    // handler produce the canonical `401` response instead of caching it.
    let Some(token_hash) = bearer_token(&request).map(hash_token) else {
        return next.run(request).await;
    };

    let method = request.method().as_str().to_string();
    let path = request.uri().path().to_string();

    match load_response(&pool, &token_hash, &key, &method, &path).await {
        Ok(Some(stored)) => return stored,
        Ok(None) => {}
        Err(err) => {
            // Failing open keeps mutations working when the cache is unreadable.
            tracing::error!(error = %err, "Failed to read idempotency key");
        }
    }

    let (parts, body) = request.into_parts();
    let Ok(body) = to_bytes(body, MAX_REQUEST_BODY).await else {
        return ApiError::bad_request("Request body exceeds the 1 MiB limit").into_response();
    };
    let response = next.run(Request::from_parts(parts, Body::from(body))).await;

    if !response.status().is_success() {
        return response;
    }

    let (parts, body) = response.into_parts();
    let Ok(body) = to_bytes(body, MAX_RESPONSE_BODY).await else {
        return ApiError::internal("Response body exceeds the 2 MiB idempotency limit")
            .into_response();
    };

    if let Err(err) = store_response(
        &pool,
        &token_hash,
        &key,
        &method,
        &path,
        parts.status,
        &body,
        parts
            .headers
            .get(header::CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
    )
    .await
    {
        tracing::error!(error = %err, "Failed to store idempotency key");
    }

    Response::from_parts(parts, Body::from(body))
}

/// Delete keys older than the seven-day retention window.
pub(crate) async fn purge_expired(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    let cutoff = unix_timestamp() - RETENTION_SECONDS;

    let result = sqlx::query!(
        "DELETE FROM api_idempotency_keys WHERE created_at < ?",
        cutoff
    )
    .execute(pool)
    .await?;

    Ok(result.rows_affected())
}

fn is_idempotent_request(method: &Method, path: &str) -> bool {
    path.starts_with("/api/v1/") && matches!(*method, Method::POST | Method::PATCH | Method::DELETE)
}

fn idempotency_key(request: &Request) -> Option<String> {
    request
        .headers()
        .get(IDEMPOTENCY_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|key| !key.is_empty())
        .map(str::to_owned)
}

fn bearer_token(request: &Request) -> Option<&str> {
    request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .map(str::trim)
        .filter(|token| !token.is_empty())
}

async fn load_response(
    pool: &SqlitePool,
    token_hash: &str,
    key: &str,
    method: &str,
    path: &str,
) -> Result<Option<Response>, sqlx::Error> {
    let min_created_at = unix_timestamp() - RETENTION_SECONDS;

    let stored = sqlx::query!(
        r#"
        SELECT status as "status!", body as "body!", content_type
        FROM api_idempotency_keys
        WHERE token_hash = ? AND key = ? AND method = ? AND path = ?
          AND created_at >= ?
        "#,
        token_hash,
        key,
        method,
        path,
        min_created_at
    )
    .fetch_optional(pool)
    .await?;

    let Some(stored) = stored else {
        return Ok(None);
    };

    let status = u16::try_from(stored.status)
        .ok()
        .and_then(|status| StatusCode::from_u16(status).ok())
        .unwrap_or(StatusCode::OK);

    let mut builder = Response::builder()
        .status(status)
        .header(REPLAYED_HEADER, HeaderValue::from_static("true"));

    if let Some(content_type) = stored.content_type {
        builder = builder.header(header::CONTENT_TYPE, content_type);
    }

    Ok(Some(builder.body(Body::from(stored.body)).unwrap_or_else(
        |_| ApiError::internal("Invalid stored response").into_response(),
    )))
}

#[allow(clippy::too_many_arguments)]
async fn store_response(
    pool: &SqlitePool,
    token_hash: &str,
    key: &str,
    method: &str,
    path: &str,
    status: StatusCode,
    body: &[u8],
    content_type: Option<&str>,
) -> Result<(), sqlx::Error> {
    // Only textual (JSON) responses are replayable; anything else would be
    // corrupted by the TEXT column.
    let Ok(body) = std::str::from_utf8(body) else {
        return Ok(());
    };

    let status = i64::from(status.as_u16());
    let created_at = unix_timestamp();

    sqlx::query!(
        r#"
        INSERT INTO api_idempotency_keys
            (token_hash, key, method, path, status, body, content_type, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(token_hash, key, method, path) DO NOTHING
        "#,
        token_hash,
        key,
        method,
        path,
        status,
        body,
        content_type,
        created_at
    )
    .execute(pool)
    .await?;

    purge_expired(pool).await?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_writes_below_the_api_prefix_are_idempotent() {
        assert!(is_idempotent_request(&Method::POST, "/api/v1/groups"));
        assert!(is_idempotent_request(&Method::PATCH, "/api/v1/debts/1"));
        assert!(is_idempotent_request(&Method::DELETE, "/api/v1/debts/1"));
        assert!(!is_idempotent_request(&Method::GET, "/api/v1/groups"));
        assert!(!is_idempotent_request(&Method::POST, "/groups"));
    }
}
