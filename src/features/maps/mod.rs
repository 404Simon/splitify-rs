//! Group maps feature.
//!
//! Each group has a map that members can inspect and mark with locations they
//! want to visit. MapLibre GL JS runs on the client; this module provides the
//! server functions and models, plus a thin WASM bridge to the bundled glue.

pub mod handlers;
#[cfg(feature = "hydrate")]
pub mod maplibre;
pub mod models;
pub mod utils;

pub use handlers::*;
pub use models::*;
