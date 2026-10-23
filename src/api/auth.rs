use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;

use super::{
    error::ApiError,
    extractors::ApiUser,
    tokens::{create_token, revoke_token},
};
use crate::features::auth::models::UserSession;

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub email: Option<String>,
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct UserResponse {
    pub id: i64,
    pub username: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: UserResponse,
}

impl From<UserSession> for UserResponse {
    fn from(user: UserSession) -> Self {
        Self {
            id: user.id,
            username: user.username,
        }
    }
}

/// Credential endpoints, mounted behind the per-IP rate limiter.
pub fn public_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

/// Token-authenticated endpoints that are safe to call frequently.
pub fn protected_router() -> Router<crate::state::AppState> {
    Router::new()
        .route("/logout", post(logout))
        .route("/me", get(me))
}

async fn register(
    State(pool): State<SqlitePool>,
    Json(payload): Json<RegisterRequest>,
) -> Result<(StatusCode, Json<AuthResponse>), ApiError> {
    let user = crate::features::auth::service::register_user(
        &pool,
        payload.username,
        payload.password,
        payload.email,
    )
    .await?;

    let token = create_token(&pool, user.id, Some("api")).await?;

    Ok((
        StatusCode::CREATED,
        Json(AuthResponse {
            token,
            user: UserResponse::from(user),
        }),
    ))
}

async fn login(
    State(pool): State<SqlitePool>,
    Json(payload): Json<LoginRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    let user = crate::features::auth::service::authenticate_user(
        &pool,
        &payload.username,
        &payload.password,
    )
    .await
    .map_err(|err| ApiError::unauthorized(ApiError::server_message(&err)))?;

    let token = create_token(&pool, user.id, Some("api")).await?;

    Ok(Json(AuthResponse {
        token,
        user: UserResponse::from(user),
    }))
}

async fn logout(State(pool): State<SqlitePool>, user: ApiUser) -> Result<StatusCode, ApiError> {
    revoke_token(&pool, &user.token_hash).await?;
    Ok(StatusCode::NO_CONTENT)
}

async fn me(user: ApiUser) -> Json<UserResponse> {
    Json(UserResponse {
        id: user.id,
        username: user.username,
    })
}
