use axum::{
    extract::{FromRef, FromRequestParts},
    http::{header::AUTHORIZATION, request::Parts},
};
use sqlx::SqlitePool;

use super::{error::ApiError, tokens::hash_token};
use crate::api::tokens::unix_timestamp;

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
                u.username as "username!"
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

        let _ = sqlx::query!(
            "UPDATE api_tokens SET last_used_at = ? WHERE token_hash = ?",
            now,
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
