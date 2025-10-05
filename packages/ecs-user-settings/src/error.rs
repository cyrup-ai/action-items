//! Error types for user settings operations
//!
//! All errors are designed to be safely cloned and sent across ECS boundaries
//! via Bevy events. Errors include detailed context for debugging while avoiding
//! sensitive data exposure.

use thiserror::Error;

/// Errors that can occur during settings operations
#[derive(Debug, Clone, Error)]
pub enum SettingsError {
    #[error("Database connection failed: {0}")]
    ConnectionFailed(String),

    #[error("Database query failed: {0}")]
    QueryFailed(String),

    #[error("Setting not found: {table}/{key}")]
    NotFound { table: String, key: String },

    #[error("Invalid setting value: {0}")]
    InvalidValue(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Schema migration failed: {0}")]
    MigrationFailed(String),

    #[error("Database error: {0}")]
    DatabaseError(String),
}
