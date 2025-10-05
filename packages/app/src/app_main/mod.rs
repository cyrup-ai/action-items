//! Main application module
//!
//! Provides the main application entry point and configuration,
//! decomposed into focused modules for better maintainability.

pub mod app_config;
pub mod events;
pub mod hotkey_setup;
pub mod systems;
pub mod window_config;
pub mod window_resize;

pub use app_config::{AppState, configure_app};
pub use systems::{add_post_startup_systems, add_startup_systems, add_update_systems};
