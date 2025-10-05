//! Icon extraction systems for async platform-specific icon loading
//!
//! Provides background task infrastructure to extract icons from OS without
//! blocking the main render thread. Uses Bevy's AsyncComputeTaskPool for
//! parallel I/O operations.
//!
//! # Architecture
//! - `platform` - Platform-specific extraction (macOS/Windows/Linux)
//! - `systems` - ECS systems for task management and polling
//!
//! # Systems
//! - [`process_icon_extraction_requests`] - Spawn extraction tasks
//! - [`poll_icon_extraction_tasks`] - Check task completion
//! - [`process_icon_extraction_results`] - Handle result events
//!
//! # Example
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_ui::icons::extraction::*;
//!
//! fn setup_extraction(app: &mut App) {
//!     app
//!         .add_systems(Update, (
//!             process_icon_extraction_requests,
//!             poll_icon_extraction_tasks,
//!             process_icon_extraction_results,
//!         ));
//! }
//! ```

pub mod platform;
pub mod systems;

// Re-export systems and component
pub use systems::{
    IconExtractionInProgress,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};

// Re-export platform function for testing/advanced usage
pub use platform::extract_icon_from_file;
