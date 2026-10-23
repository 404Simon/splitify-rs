use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use sqlx::SqlitePool;

use super::{error::ApiError, tokens::hash_token};
use crate::api::tokens::{TOKEN_TTL_DAYS, unix_timestamp};

const SECONDS_PER_DAY: i64 = 24 * 60 * 60;

/// Authenticated API user resolved from a `Bearer` token.
pub struct ApiUser {
    pub id: i64,
    pub username: String,
    pub token_hash: String,
}

impl<S> FromRequestParts<S> for ApiUser
where
    S: Send + Sync,
    SqlitePool: FromRef<S>,
{
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = parts
            .headers
            .get(AUTHORIZATION)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .map(str::trim)
            .filter(|token| !token.is_empty())
            .ok_or_else(|| ApiError::unauthorized("Missing or invalid Authorization header"))?;

        let token_hash = hash_token(token);
        let token_hash_value = token_hash.as_str();
        let now = unix_timestamp();
        let pool = SqlitePool::from_ref(state);

        let user = sqlx::query!(
            r#"
            SELECT
                u.id as "id!",
                u.username as "username!",
                t.expires_at as "expires_at!"
            FROM api_tokens t
            INNER JOIN users u ON u.id = t.user_id
            WHERE t.token_hash = ? AND t.expires_at > ?
            "#,
            token_hash_value,
            now
        )
        .fetch_optional(&pool)
        .await?;

        let user = user.ok_or_else(|| ApiError::unauthorized("Invalid or expired token"))?;

        // Sliding expiration: keep active clients signed in indefinitely by
        // extending the token once it has less than `TOKEN_TTL_DAYS - 1` days
        // left. The CASE keeps this a single UPDATE.
        let extend_threshold = now + (TOKEN_TTL_DAYS - 1) * SECONDS_PER_DAY;
        let extended_expiry = now + TOKEN_TTL_DAYS * SECONDS_PER_DAY;

        if user.expires_at < extend_threshold {
            tracing::debug!(
                expires_at = user.expires_at,
                "Extending bearer token expiry"
            );
        }

        let _ = sqlx::query!(
            r#"
            UPDATE api_tokens
            SET last_used_at = ?,
                expires_at = CASE
                    WHEN expires_at < ? THEN ?
                    ELSE expires_at
                END
            WHERE token_hash = ?
            "#,
            now,
            extend_threshold,
            extended_expiry,
            token_hash_value
        )
        .execute(&pool)
        .await;

        Ok(Self {
            id: user.id,
            username: user.username,
            token_hash,
        })
    }
}
