//! Database service re-exports from ecs-surrealdb
//!
//! This module provides access to the database services implemented in ecs-surrealdb,
//! making them available through the standard services module hierarchy.

// Re-export database types from ecs-surrealdb - temporarily disabled due to compilation issues
// pub use action_items_ecs_surrealdb::{
//     config::{DatabaseConfig, DatabaseError, DatabaseEngine},
//     service::{DatabaseService, DatabaseServiceError, DatabaseShutdown},
// };

// Placeholder types for when database service is disabled
pub struct DatabaseConfig;
pub struct DatabaseError;
pub struct DatabaseService;
