mod auth;
mod debts;
mod error;
mod extractors;
mod groups;
mod idempotency;
mod invites;
mod maps;
mod recurring_debts;
mod shopping_lists;
mod sync;
mod tokens;
mod transactions;

use std::time::Duration;

use axum::{
    Json, Router,
    extract::{Request, State},
    http::{HeaderValue, header},
    middleware::{self, Next},
    response::Response,
    routing::get,
};
use serde::Serialize;
use sqlx::SqlitePool;
use tower_governor::{
    GovernorLayer, governor::GovernorConfigBuilder, key_extractor::SmartIpKeyExtractor,
};

pub use error::ApiError;
pub use extractors::ApiUser;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
}

pub fn router(pool: SqlitePool) -> Router<AppState> {
    // Throttle credential endpoints per client IP to slow down brute-force and
    // credential-stuffing attempts. Applied only to the public auth routes so
    // that authenticated polling (e.g. `/auth/me`) is never throttled.
    let governor_conf = GovernorConfigBuilder::default()
        .per_second(1)
        .burst_size(10)
        .key_extractor(SmartIpKeyExtractor)
        .finish()
        .expect("valid API rate limit configuration");

    // Periodically drop stale per-IP buckets so memory stays bounded.
    let limiter = governor_conf.limiter().clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            limiter.retain_recent();
        }
    });

    let auth_public = auth::public_router().layer(GovernorLayer::new(governor_conf));

    Router::new()
        .route("/api/v1/health", get(health))
        .nest("/api/v1/auth", auth_public.merge(auth::protected_router()))
        .nest(
            "/api/v1/groups",
            groups::router()
                .merge(debts::group_router())
                .merge(transactions::group_router())
                .merge(invites::group_router())
                .merge(shopping_lists::group_router())
                .merge(recurring_debts::group_router())
                .merge(maps::group_router()),
        )
        .nest("/api/v1/debts", debts::resource_router())
        .nest("/api/v1/transactions", transactions::resource_router())
        .nest("/api/v1/invites", invites::public_router())
        .nest("/api/v1/shopping-lists", shopping_lists::resource_router())
        .nest(
            "/api/v1/recurring-debts",
            recurring_debts::resource_router(),
        )
        .nest("/api/v1/markers", maps::marker_router())
        .nest("/api/v1/map", maps::public_router())
        .route("/api/v1/users", get(list_users))
        // Idempotency wraps the routes but stays inside the security-header
        // layer so replayed responses also carry the standard headers.
        .layer(middleware::from_fn_with_state(
            pool,
            idempotency::middleware,
        ))
        .layer(middleware::from_fn(api_security_headers))
}

/// Delete idempotency keys that fell out of the seven-day retention window.
///
/// Called once on startup; the middleware also purges opportunistically after
/// storing a response.
pub async fn purge_expired_idempotency_keys(pool: &SqlitePool) -> Result<u64, sqlx::Error> {
    idempotency::purge_expired(pool).await
}

/// Default security headers for every JSON API response.
///
/// `no-store` keeps tokens and financial data out of any shared/browser cache,
/// `nosniff` stops MIME-type sniffing, and `no-referrer` avoids leaking API
/// paths to third parties via the `Referer` header.
async fn api_security_headers(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::X_CONTENT_TYPE_OPTIONS,
        HeaderValue::from_static("nosniff"),
    );
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
    })
}

async fn list_users(
    State(pool): State<SqlitePool>,
    user: ApiUser,
) -> Result<Json<Vec<auth::UserResponse>>, ApiError> {
    let users = crate::features::groups::service::list_other_users(&pool, user.id).await?;

    Ok(Json(
        users.into_iter().map(auth::UserResponse::from).collect(),
    ))
}
