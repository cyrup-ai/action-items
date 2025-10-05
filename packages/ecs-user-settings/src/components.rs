//! Component types for async task management
//!
//! Task components hold async operations spawned to `AsyncComputeTaskPool`.
//! Each task stores its operation ID, requester entity, and any relevant metadata
//! needed to send completion events back to the requester.
//!
//! Resource markers track initialization state (schema, migration) to ensure
//! proper startup sequencing.

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::Task;
use uuid::Uuid;

/// Component for schema initialization task
#[derive(Component)]
pub struct SchemaInitTask(pub Task<CommandQueue>);

/// Component for migration task
#[derive(Component)]
pub struct MigrationTask(pub Task<CommandQueue>);

/// Component for settings read task
#[derive(Component)]
pub struct SettingsReadTask {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub requester: Entity,
    pub task: Task<CommandQueue>,
}

/// Component for settings write task
#[derive(Component)]
pub struct SettingsWriteTask {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub requester: Entity,
    pub task: Task<CommandQueue>,
}

/// Component for settings update task
#[derive(Component)]
pub struct SettingsUpdateTask {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub requester: Entity,
    pub task: Task<CommandQueue>,
}

/// Component for settings delete task
#[derive(Component)]
pub struct SettingsDeleteTask {
    pub operation_id: Uuid,
    pub table: String,
    pub key: String,
    pub requester: Entity,
    pub task: Task<CommandQueue>,
}

/// Component for settings query task
#[derive(Component)]
pub struct SettingsQueryTask {
    pub operation_id: Uuid,
    pub requester: Entity,
    pub task: Task<CommandQueue>,
}

// ============================================================================
// Resource Markers
// ============================================================================

/// Marker resource indicating schema is initialized
#[derive(Resource)]
pub struct SchemaInitialized;

/// Resource indicating schema initialization error
#[derive(Resource)]
pub struct SchemaInitError(pub String);

/// Marker resource indicating migration completed
#[derive(Resource)]
pub struct MigrationCompleted;

/// Resource indicating migration failed
#[derive(Resource)]
pub struct MigrationFailed(pub String);
