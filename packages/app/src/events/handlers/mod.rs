//! Event handler modules
//!
//! Modular organization of event handling systems for the Action Items launcher.

pub mod key_capture;
pub mod launcher_events;
pub mod preferences;
pub mod ui_interactions;
pub mod utils;
pub mod wizard_bridge;

// Re-export all handler functions for easy access
pub use key_capture::{detect_preferences_command, real_hotkey_capture_system};
pub use launcher_events::{
    handle_execute_commands, handle_launcher_events, update_current_query_from_events,
};
pub use preferences::handle_preferences_events;
pub use ui_interactions::handle_preferences_ui_interactions;
pub use wizard_bridge::bridge_wizard_visibility_to_window;
