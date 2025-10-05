//! Accessibility system for Action Items launcher
//!
//! Generic accessibility (keyboard nav, focus, detection):
//!   Re-exported from action_items_ecs_ui::accessibility
//!
//! UI-specific accessibility (launcher setup, search announcements):
//!   Defined locally in this module

// Re-export generic accessibility from ecs-ui
pub use action_items_ecs_ui::accessibility::*;

// UI-specific accessibility modules
pub mod announcements;
pub mod setup;

// Re-export UI-specific functions
pub use announcements::update_accessibility_announcements;
pub use setup::setup_accessibility;
