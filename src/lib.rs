#![recursion_limit = "1024"]

pub mod app;
pub mod components;
pub mod pages;

pub mod features {
    pub mod auth;
    pub mod groups;
    pub mod invites;
    pub mod shared_debts;
    pub mod transactions;
}

#[cfg(feature = "ssr")]
pub mod db;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    use crate::app::*;
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(App);
}
