use axum::extract::FromRef;
use leptos::prelude::LeptosOptions;
use sqlx::SqlitePool;

use crate::features::shopping_lists::EventBroadcaster;

/// Shared Axum state for Leptos routes, REST API routes and SSE.
#[derive(Clone)]
pub struct AppState {
    pub leptos_options: LeptosOptions,
    pub pool: SqlitePool,
    pub broadcaster: EventBroadcaster,
}

impl FromRef<AppState> for LeptosOptions {
    fn from_ref(state: &AppState) -> Self {
        state.leptos_options.clone()
    }
}

impl FromRef<AppState> for SqlitePool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}

impl FromRef<AppState> for EventBroadcaster {
    fn from_ref(state: &AppState) -> Self {
        state.broadcaster.clone()
    }
}
