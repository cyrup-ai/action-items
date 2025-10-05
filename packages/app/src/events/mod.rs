//! Event system for the Action Items launcher
//!
//! This module contains all event definitions and handlers for the launcher application.
//! Events are used for communication between different systems and components.

pub mod global_hotkey;
pub mod handlers;
pub mod preferences;

// Re-export all events for easy access
pub use global_hotkey::GlobalHotkeyEvent;
// Re-export handlers for system registration
pub use handlers::*;
pub use preferences::PreferencesEvent;
