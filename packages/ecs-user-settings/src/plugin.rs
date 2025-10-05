//! Bevy plugin for user settings service

use bevy::prelude::*;
use tracing::info;

use crate::events::*;
use crate::systems::*;

/// User settings plugin - provides centralized database-backed settings storage
pub struct UserSettingsPlugin;

impl Plugin for UserSettingsPlugin {
    fn build(&self, app: &mut App) {
        info!("Initializing User Settings Plugin");

        // Initialize schema at startup (chained systems)
        app.add_systems(PostStartup, (
            initialize_user_settings_schema,
            handle_schema_init_task,
        ).chain());

        // Run migration after schema initialization
        app.add_systems(PostStartup, run_migration_on_startup.after(handle_schema_init_task));
        app.add_systems(PostStartup, handle_migration_task.after(run_migration_on_startup));

        // Add request events
        app.add_event::<SettingsReadRequested>()
            .add_event::<SettingsWriteRequested>()
            .add_event::<SettingsUpdateRequested>()
            .add_event::<SettingsDeleteRequested>()
            .add_event::<SettingsQueryRequested>();

        // Add response events
        app.add_event::<SettingsReadCompleted>()
            .add_event::<SettingsWriteCompleted>()
            .add_event::<SettingsUpdateCompleted>()
            .add_event::<SettingsDeleteCompleted>()
            .add_event::<SettingsQueryCompleted>();

        // Add change notification event
        app.add_event::<SettingChanged>();

        // Add processing systems (Update schedule)
        app.add_systems(Update, (
            // Request processors
            process_settings_read_requests,
            process_settings_write_requests,
            process_settings_update_requests,
            process_settings_delete_requests,
            process_settings_query_requests,
            // Task handlers
            handle_settings_read_tasks,
            handle_settings_write_tasks,
            handle_settings_update_tasks,
            handle_settings_delete_tasks,
            handle_settings_query_tasks,
            // Audit trail (runs after task handlers to capture all SettingChanged events)
            write_audit_trail,
        ));

        info!("User Settings Plugin initialized successfully");
    }
}
