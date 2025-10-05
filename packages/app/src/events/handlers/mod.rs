//! Event handler modules
//!
//! Modular organization of event handling systems for the Action Items launcher.

pub mod global_hotkeys;
pub mod key_capture;
pub mod launcher_events;
pub mod preferences;
pub mod ui_interactions;
pub mod utils;

// Re-export all handler functions for easy access
pub use global_hotkeys::handle_global_hotkeys;
pub use key_capture::{detect_preferences_command, real_hotkey_capture_system};
pub use launcher_events::{
    handle_execute_commands, handle_launcher_events, update_current_query_from_events,
};
pub use preferences::handle_preferences_events;
pub use ui_interactions::handle_preferences_ui_interactions;
