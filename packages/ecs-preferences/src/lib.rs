//! ECS Preferences Service
//!
//! Provides preferences management with both core functionality and optional UI.
//! Integrates with ecs-hotkey for conflict detection.
//!
//! ## Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use action_items_ecs_preferences::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(PreferencesPlugin)  // Core (headless)
//!         .add_plugins(PreferencesUIPlugin)  // UI (optional)
//!         .run();
//! }
//! ```

pub mod events;
pub mod plugin;
pub mod resources;
pub mod systems;
pub mod ui;

// Re-export main plugin
pub use plugin::PreferencesPlugin;

// Re-export UI plugin
pub use ui::PreferencesUIPlugin;

// Re-export key types
pub use events::*;
pub use resources::*;
