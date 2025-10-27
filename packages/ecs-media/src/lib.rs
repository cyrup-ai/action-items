//! ECS Media Package
//!
//! Provides media upload, storage, and metadata management for the Action Items application.
//!
//! ## Features
//! - SurrealDB-based metadata storage
//! - Integration with ecs-filesystem for file storage
//! - Async ECS-based operations
//! - Type-safe media operations

pub mod events;
pub mod manager;
pub mod plugin;
pub mod schema;
pub mod types;

// Re-export public API
pub use events::{MediaRequest, MediaResponse};
pub use plugin::{MediaConfig, MediaPlugin, MediaService};
pub use types::{Media, MediaError, MediaId};
pub use schema::MEDIA_SCHEMA;
