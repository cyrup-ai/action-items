//! User settings service with database-backed persistence
//!
//! # Overview
//!
//! Provides centralized user settings storage using SurrealDB backend with:
//! - **SQL Injection Prevention**: Table name validation and RecordId type safety
//! - **Complete Audit Trail**: Full change history with old/new values in settings_history
//! - **Event-Driven Architecture**: Request/response pattern for async database operations
//! - **Automatic Migration**: JSON to database migration on first startup
//!
//! # Security
//!
//! All database operations use:
//! - Whitelisted table names (12 valid tables) - see [`types::VALID_TABLES`]
//! - SurrealDB's RecordId type for safe record addressing
//! - Parameterized queries where applicable
//! - No string interpolation of user input into queries
//!
//! # Usage
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use action_items_ecs_user_settings::*;
//!
//! fn setup(mut app: App) {
//!     app.add_plugins(UserSettingsPlugin);
//! }
//!
//! fn request_setting(mut events: EventWriter<SettingsReadRequested>) {
//!     events.send(SettingsReadRequested {
//!         operation_id: Uuid::new_v4(),
//!         table: "user_preferences".to_string(),
//!         key: "theme".to_string(),
//!         requester: entity,
//!     });
//! }
//!
//! fn handle_response(mut events: EventReader<SettingsReadCompleted>) {
//!     for event in events.read() {
//!         match &event.result {
//!             Ok(Some(value)) => println!("Setting: {:?}", value),
//!             Ok(None) => println!("Not found"),
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//! }
//! ```
//!
//! # Architecture
//!
//! - [`plugin::UserSettingsPlugin`] - Main Bevy plugin
//! - [`events`] - Request and response events for all operations
//! - [`systems`] - Request processors and task handlers
//! - [`types`] - Table validation and RecordId construction
//! - [`schema`] - SurrealDB schema definition
//! - [`migration`] - JSON to database migration logic

mod components;
mod error;
mod events;
mod migration;
mod plugin;
mod schema;
mod systems;
pub mod table_names;
mod types;

#[cfg(test)]
mod tests;

pub use components::*;
pub use error::*;
pub use events::*;
pub use plugin::UserSettingsPlugin;
pub use schema::USER_SETTINGS_SCHEMA;
