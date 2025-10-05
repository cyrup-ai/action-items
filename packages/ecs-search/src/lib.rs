//! ECS Search Service
//!
//! Provides search functionality with scoring, filtering, and caching.
//! Can be used headless (core only) or with UI.
//!
//! ## Features
//!
//! - Event-driven search requests and responses
//! - Async search task management
//! - Result scoring and filtering
//! - Search result caching with TTL
//! - Configurable search parameters
//!
//! ## Usage
//!
//! ```no_run
//! use bevy::prelude::*;
//! use action_items_ecs_search::*;
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(SearchPlugin)
//!         .add_systems(Update, handle_search_results)
//!         .run();
//! }
//!
//! fn handle_search_results(
//!     mut search_events: EventReader<SearchCompleted>,
//! ) {
//!     for event in search_events.read() {
//!         println!("Found {} results for '{}'", event.results.len(), event.query);
//!     }
//! }
//! ```

pub mod components;
pub mod events;
pub mod plugin;
pub mod resources;
pub mod scoring;
pub mod systems;
pub mod ui;

// Re-export main plugin
pub use plugin::{SearchPlugin, SearchPluginWithConfig};

// Re-export UI plugin
pub use ui::SearchUIPlugin;

// Re-export key types
pub use events::*;
pub use resources::*;
pub use components::*;
pub use scoring::{SearchScore, ScoreTier, ConfidenceLevel};
pub use systems::filtering::{FilterCategory, FilterState};
