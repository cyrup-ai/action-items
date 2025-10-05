#![recursion_limit = "256"]
//! # Bevy Progress Tracking Plugin
//!
//! A comprehensive progress tracking system for Bevy applications using ECS
//! patterns, events, and hooks for better integration and extensibility.
//!
//! ## Features
//!
//! - **Event-driven architecture**: Uses Bevy events for progress updates and
//!   state transitions
//! - **Resource-based state management**: Eliminates mutex locks for better
//!   performance
//! - **Hook system**: Extensible callbacks for progress milestones
//! - **Builder pattern**: Ergonomic plugin configuration
//! - **Asset tracking**: Built-in asset loading progress (optional)
//! - **Async support**: Background thread progress updates (optional)
//! - **Debug logging**: Progress monitoring and debugging (optional)
//!
//! ## Basic Usage
//!
//! ```rust
//! use bevy::prelude::*;
//! use action_items_ecs_progress::prelude::*;
//!
//! #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! enum GameState {
//!     #[default]
//!     Loading,
//!     Playing,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .init_state::<GameState>()
//!         .add_plugins(
//!             ProgressPlugin::<GameState>::new()
//!                 .with_transition(GameState::Loading, GameState::Playing)
//!                 .with_assets()  // Enable asset tracking
//!         )
//!         .add_systems(Update, loading_system.track_progress::<GameState>())
//!         .run();
//! }
//!
//! fn loading_system(mut handle: ProgressHandle<GameState>) -> Progress {
//!     // Your loading logic here
//!     handle.set_visible(1, 1); // Mark as complete
//!     Progress { done: 1, total: 1 }
//! }
//! ```
//!
//! ## Progress Reporting Methods
//!
//! ### System Parameters
//! Use `ProgressHandle` in your systems:
//! ```rust
//! # use bevy::prelude::*;
//! # use action_items_ecs_progress::prelude::*;
//! # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! # enum GameState { #[default] Loading }
//! fn my_loading_system(mut progress: ProgressHandle<GameState>) {
//!     progress.set_visible(50, 100);  // 50% complete
//!     progress.set_hidden(1, 1);      // Hidden requirement complete
//! }
//! ```
//!
//! ### Return Values
//! Return progress from systems:
//! ```rust
//! # use bevy::prelude::*;
//! # use action_items_ecs_progress::prelude::*;
//! # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! # enum GameState { #[default] Loading }
//! fn loading_system() -> Progress {
//!     // Your loading logic
//!     Progress { done: 3, total: 10 }
//! }
//!
//! // Add with: .add_systems(Update, loading_system.track_progress::<GameState>())
//! ```
//!
//! ### Entity Components
//! Use components on entities:
//! ```rust
//! # use bevy::prelude::*;
//! # use action_items_ecs_progress::prelude::*;
//! # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! # enum GameState { #[default] Loading }
//! fn spawn_loading_entities(mut commands: Commands) {
//!     commands.spawn(ProgressEntity::<GameState>::new()
//!         .with_progress(0, 100)  // Start at 0/100
//!         .with_hidden_progress(0, 5));  // Hidden: 0/5
//! }
//! ```
//!
//! ## Hooks and Events
//!
//! Register hooks to react to progress events:
//! ```rust
//! # use bevy::prelude::*;
//! # use action_items_ecs_progress::prelude::*;
//! # #[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
//! # enum GameState { #[default] Loading }
//! struct LoadingHook;
//!
//! impl CompletionHook<GameState> for LoadingHook {
//!     fn on_complete(&self, state: &GameState) {
//!         println!("Loading complete for state: {:?}", state);
//!     }
//! }
//!
//! // Add to plugin:
//! // .with_completion_hook(LoadingHook)
//! ```

#![warn(missing_docs)]

// Bevy imports
use bevy::prelude::*;

/// All public API items
pub mod prelude {
    #[cfg(feature = "assets")]
    pub use crate::assets::AssetProgressSet;
    #[cfg(feature = "debug")]
    pub use crate::debug::*;
    pub use crate::entity::*;
    pub use crate::monitor::{ProgressMonitor, EntryId};
    pub use crate::plugin::ProgressPlugin;
    pub use crate::progress::*;
    #[cfg(feature = "async")]
    pub use crate::send::{ProgressSender, AsyncProgressMessage as SendAsyncProgressMessage, process_async_progress};
    pub use crate::state::*;
    pub use crate::system::{IntoProgress, hide_progress, show_progress};
    pub use crate::utils::*;
}

// Re-export everything from prelude at crate root for convenience
pub use prelude::*;

// Module declarations
mod entity;
mod monitor;
mod plugin;
mod progress;
mod state;
mod system;
mod utils;

#[cfg(feature = "assets")]
mod assets;
#[cfg(feature = "debug")]
mod debug;
#[cfg(feature = "async")]
mod send;
