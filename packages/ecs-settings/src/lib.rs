//! ECS Settings Service
//!
//! Provides settings management with event-driven architecture.
//! Separates core settings logic from UI presentation.
//!
//! ## Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use action_items_ecs_settings::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(SettingsPlugin)
//!         .add_systems(Update, handle_tab_changes)
//!         .run();
//! }
//!
//! fn handle_tab_changes(mut events: EventReader<TabChanged>) {
//!     for event in events.read() {
//!         println!("Tab changed to: {:?}", event.tab);
//!     }
//! }
//! ```

pub mod events;
pub mod navigation;
pub mod persistence;
pub mod plugin;
pub mod resources;
pub mod systems;
pub mod ui;

// Re-export main plugin
pub use plugin::SettingsPlugin;

// Re-export UI plugin
pub use ui::SettingsUIPlugin;

// Re-export key types
pub use events::*;
pub use resources::*;
pub use navigation::{SettingsTab, ExtensionFilter};
