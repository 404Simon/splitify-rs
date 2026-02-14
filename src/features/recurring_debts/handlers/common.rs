// Common imports and utilities for recurring debt handlers

pub use super::models::{GeneratedInstance, RecurringDebtMember, RecurringDebtWithDetails};
pub use leptos::prelude::*;

#[cfg(feature = "ssr")]
pub use super::models::Frequency;

#[cfg(feature = "ssr")]
pub use rust_decimal::Decimal;

#[cfg(feature = "ssr")]
pub use time::Date;

#[cfg(feature = "ssr")]
pub use super::utils::{calculate_next_occurrence, should_generate};

#[cfg(feature = "ssr")]
pub use crate::features::auth::utils::get_user_from_session;

#[cfg(feature = "ssr")]
pub use crate::features::shared_debts::utils::calculate_shares;

#[cfg(feature = "ssr")]
pub use leptos_axum::extract;

#[cfg(feature = "ssr")]
pub use tower_sessions::Session;

#[cfg(feature = "ssr")]
pub use sqlx::SqlitePool;
