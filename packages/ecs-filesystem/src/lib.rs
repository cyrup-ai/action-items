//! # ECS Filesystem
//!
//! High-performance, secure Bevy ECS-based filesystem operations with:
//! - **Security**: Path traversal prevention, whitelist-based access control, audit logging
//! - **Performance**: Memory-mapped I/O, intelligent caching, batch operations, zero-copy where
//!   possible
//! - **Cross-platform**: macOS FSEvents, Linux inotify, Windows ReadDirectoryChangesW
//! - **Enterprise-grade**: Comprehensive error handling, rate limiting, resource management
//!
//! ## Quick Start
//!
//! ```rust
//! use bevy::prelude::*;
//! use ecs_filesystem::{FileSystemPlugin, FileSystemRequest, FileSystemResponse};
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(FileSystemPlugin)
//!         .add_systems(Startup, setup_filesystem)
//!         .run();
//! }
//!
//! fn setup_filesystem(mut commands: Commands) {
//!     // Request to read a file
//!     commands
//!         .spawn(())
//!         .observe(|mut events: EventReader<FileSystemResponse>| {
//!             for event in events.read() {
//!                 match event {
//!                     FileSystemResponse::ReadFileResult { result, .. } => match result {
//!                         Ok(content) => println!("Read {} bytes", content.data.len()),
//!                         Err(e) => eprintln!("Failed to read file: {}", e),
//!                     },
//!                     _ => {},
//!                 }
//!             }
//!         });
//! }
//! ```

pub mod events;
pub mod manager;
pub mod plugin;
pub mod security;
pub mod types;
pub mod watcher;

// Re-export main types and plugin for easy use
// Re-export Bevy types that users will need
pub use bevy::prelude::{Component, Entity, Event, Resource};
pub use events::{
    FileSystemChanged, FileSystemOperationFailed, FileSystemRequest, FileSystemResponse,
};
pub use plugin::{FileSystemConfig, FileSystemPlugin, FileSystemResource};
pub use security::SecurityConfig;
pub use types::{
    DirectoryListing, FileContent, FileMetadata, FileOperationId, FileSystemChange,
    FileSystemError, Priority, WatchConfig,
};
