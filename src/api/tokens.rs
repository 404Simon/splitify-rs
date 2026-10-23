use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};
use sqlx::SqlitePool;
use uuid::Uuid;

use super::error::ApiError;

pub const TOKEN_TTL_DAYS: i64 = 30;

/// Generate an opaque bearer token: `splitify_` plus two random UUIDs.
pub fn generate_token() -> String {
    format!(
        "splitify_{}{}",
        Uuid::new_v4().simple(),
        Uuid::new_v4().simple()
    )
}

/// Hash a token with SHA-256 and return lowercase hex.
pub fn hash_token(token: &str) -> String {
    let digest = Sha256::digest(token.as_bytes());
    format!("{digest:x}")
}

/// Create and persist a new token, returning the plaintext token once.
pub async fn create_token(
    pool: &SqlitePool,
    user_id: i64,
    name: Option<&str>,
) -> Result<String, ApiError> {
    let token = generate_token();
    let token_hash = hash_token(&token);
    let expires_at = unix_timestamp() + TOKEN_TTL_DAYS * 24 * 60 * 60;

    sqlx::query!(
        r#"
        INSERT INTO api_tokens (user_id, token_hash, name, expires_at)
        VALUES (?, ?, ?, ?)
        "#,
        user_id,
        token_hash,
        name,
        expires_at
    )
    .execute(pool)
    .await?;

    Ok(token)
}

/// Delete a token by its hash (logout).
pub async fn revoke_token(pool: &SqlitePool, token_hash: &str) -> Result<(), ApiError> {
    sqlx::query!("DELETE FROM api_tokens WHERE token_hash = ?", token_hash)
        .execute(pool)
        .await?;

    Ok(())
}

pub(crate) fn unix_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| i64::try_from(duration.as_secs()).unwrap_or(i64::MAX))
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn is_lowercase_hex(value: &str) -> bool {
        !value.is_empty()
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    }

    #[test]
    fn generated_token_has_expected_shape() {
        let token = generate_token();
        let suffix = token
            .strip_prefix("splitify_")
            .expect("token should have the splitify_ prefix");

        assert_eq!(suffix.len(), 64);
        assert!(is_lowercase_hex(suffix));
    }

    #[test]
    fn generated_tokens_differ() {
        assert_ne!(generate_token(), generate_token());
    }

    #[test]
    fn token_hash_is_deterministic_and_lowercase_hex() {
        let token = "splitify_example";
        let first = hash_token(token);
        let second = hash_token(token);

        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert!(is_lowercase_hex(&first));
    }

    #[test]
    fn different_tokens_hash_differently() {
        assert_ne!(hash_token("splitify_a"), hash_token("splitify_b"));
    }
}
