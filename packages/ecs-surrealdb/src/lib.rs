//! SurrealDB ECS service for Bevy applications
//!
//! Provides a complete SurrealDB integration with Bevy ECS patterns,
//! including transaction support, health monitoring, and graceful shutdown.
//!
//! Uses SurrealDB v3.0's recommended LazyLock singleton pattern

pub mod config;
pub mod service;
pub mod transactions;
pub mod plugin;

// Re-export public API
pub use config::{DatabaseConfig, DatabaseEngine, DatabaseError};
pub use service::{DatabaseService, DatabaseServiceError, DatabaseShutdown};
pub use transactions::TransactionContext;
pub use plugin::DatabasePlugin;