//! Shared debts server functions
//!
//! This module contains all server-side handlers for shared debt operations.

mod create;
mod delete;
mod query;
mod update;

// Re-export all server functions
pub use create::*;
pub use delete::*;
pub use query::*;
pub use update::*;
