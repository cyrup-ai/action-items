//! Bevy systems for user settings management

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use surrealdb::{RecordId, Value};
use tracing::{debug, error, info};
use action_items_ecs_surrealdb::DatabaseService;
use action_items_common::AppDirectories;

use crate::components::*;
use crate::error::SettingsError;
use crate::events::*;
use crate::types::parse_record_id;
use crate::migration;
use crate::schema::USER_SETTINGS_SCHEMA;

// ============================================================================
// Schema Initialization
// ============================================================================

/// Initialize user settings schema in database
pub fn initialize_user_settings_schema(
    mut commands: Commands,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot initialize user settings schema");
        return;
    };

    info!("Initializing user settings schema");

    // Clone for async task
    let db = (*db).clone();

    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();

        debug!("Applying user settings schema to database");

        // Execute schema using DatabaseService.query()
        match db.query(USER_SETTINGS_SCHEMA).await {
            Ok(_) => {
                info!("User settings schema applied successfully");
                command_queue.push(|world: &mut World| {
                    world.insert_resource(SchemaInitialized);
                });
            },
            Err(e) => {
                error!("Failed to apply user settings schema: {}", e);
                command_queue.push(move |world: &mut World| {
                    world.insert_resource(SchemaInitError(e.to_string()));
                });
            },
        }

        command_queue
    });

    commands.spawn(SchemaInitTask(task));
}

/// Handle schema initialization task completion
pub fn handle_schema_init_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SchemaInitTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Read Operations
// ============================================================================

/// Process settings read requests
pub fn process_settings_read_requests(
    mut commands: Commands,
    mut events: EventReader<SettingsReadRequested>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot process read requests");
        for request in events.read() {
            let operation_id = request.operation_id;
            let table = request.table.clone();
            let key = request.key.clone();
            let requester = request.requester;
            
            commands.queue(move |world: &mut World| {
                world.send_event(SettingsReadCompleted {
                    operation_id,
                    table,
                    key,
                    result: Err(SettingsError::DatabaseError(
                        "Database service not available".into()
                    )),
                    requester,
                });
            });
        }
        return;
    };

    for request in events.read() {
        // Validate table and key before spawning async task
        let record_id = match parse_record_id(&request.table, &request.key) {
            Ok(rid) => rid,
            Err(e) => {
                error!("Invalid table/key for read: {}", e);
                let operation_id = request.operation_id;
                let table = request.table.clone();
                let key = request.key.clone();
                let requester = request.requester;
                
                commands.queue(move |world: &mut World| {
                    world.send_event(SettingsReadCompleted {
                        operation_id,
                        table,
                        key,
                        result: Err(e),
                        requester,
                    });
                });
                continue;
            }
        };

        let db = db.as_ref().clone();
        let table = request.table.clone();
        let key = request.key.clone();
        let operation_id = request.operation_id;
        let requester = request.requester;

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();

            // Build SurrealQL query using validated RecordId (SQL injection safe)
            let query = format!("SELECT * FROM {}", record_id);

            // Execute query
            let result = match db.query(&query).await {
                Ok(mut response) => {
                    // Extract first result
                    match response.take::<Option<Value>>(0) {
                        Ok(value) => Ok(value),
                        Err(e) => Err(SettingsError::QueryFailed(e.to_string())),
                    }
                },
                Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
            };

            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(SettingsReadCompleted {
                    operation_id,
                    table,
                    key,
                    result,
                    requester,
                });
            });

            command_queue
        });

        commands.spawn(SettingsReadTask {
            operation_id,
            table: request.table.clone(),
            key: request.key.clone(),
            requester,
            task,
        });
    }
}

/// Handle settings read task completion
pub fn handle_settings_read_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SettingsReadTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Write Operations
// ============================================================================

/// Process settings write requests
pub fn process_settings_write_requests(
    mut commands: Commands,
    mut events: EventReader<SettingsWriteRequested>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot process write requests");
        for request in events.read() {
            let operation_id = request.operation_id;
            let table = request.table.clone();
            let key = request.key.clone();
            let requester = request.requester;
            
            commands.queue(move |world: &mut World| {
                world.send_event(SettingsWriteCompleted {
                    operation_id,
                    table,
                    key,
                    result: Err(SettingsError::DatabaseError(
                        "Database service not available".into()
                    )),
                    requester,
                });
            });
        }
        return;
    };

    for request in events.read() {
        // Validate table and key before spawning async task
        let record_id = match parse_record_id(&request.table, &request.key) {
            Ok(rid) => rid,
            Err(e) => {
                error!("Invalid table/key for write: {}", e);
                let operation_id = request.operation_id;
                let table = request.table.clone();
                let key = request.key.clone();
                let requester = request.requester;
                
                commands.queue(move |world: &mut World| {
                    world.send_event(SettingsWriteCompleted {
                        operation_id,
                        table,
                        key,
                        result: Err(e),
                        requester,
                    });
                });
                continue;
            }
        };

        let db = db.as_ref().clone();
        let table = request.table.clone();
        let key = request.key.clone();
        let value = request.value.clone();
        let operation_id = request.operation_id;
        let requester = request.requester;

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();

            // Read-before-write for audit trail (task 2.1)
            // First, read existing value for old_value in SettingChanged event
            let read_query = format!("SELECT * FROM {}", record_id);
            let old_value = match db.query(&read_query).await {
                Ok(mut response) => {
                    match response.take::<Option<Value>>(0) {
                        Ok(val) => val,
                        Err(e) => {
                            error!("Failed to read old value for audit: {}", e);
                            None
                        }
                    }
                },
                Err(e) => {
                    error!("Failed to query old value for audit: {}", e);
                    None
                }
            };

            // Build UPSERT query using validated RecordId (SQL injection safe)
            let write_query = format!("UPDATE {} CONTENT {}", record_id, value);

            // Clone for use in multiple closures
            let table_for_change = table.clone();
            let key_for_change = key.clone();
            let value_for_change = value.clone();

            // Execute write query
            let result = match db.query(&write_query).await {
                Ok(_) => {
                    // Emit change notification with proper old_value for audit trail
                    command_queue.push(move |world: &mut World| {
                        world.send_event(SettingChanged {
                            table: table_for_change,
                            key: key_for_change,
                            old_value,
                            new_value: value_for_change,
                            changed_at: chrono::Utc::now(),
                        });
                    });
                    Ok(())
                },
                Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
            };

            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(SettingsWriteCompleted {
                    operation_id,
                    table,
                    key,
                    result,
                    requester,
                });
            });

            command_queue
        });

        commands.spawn(SettingsWriteTask {
            operation_id,
            table: request.table.clone(),
            key: request.key.clone(),
            requester,
            task,
        });
    }
}

/// Handle settings write task completion
pub fn handle_settings_write_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SettingsWriteTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Update Operations
// ============================================================================

/// Process settings update requests (partial update)
pub fn process_settings_update_requests(
    mut commands: Commands,
    mut events: EventReader<SettingsUpdateRequested>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot process update requests");
        for request in events.read() {
            let operation_id = request.operation_id;
            let table = request.table.clone();
            let key = request.key.clone();
            let requester = request.requester;
            
            commands.queue(move |world: &mut World| {
                world.send_event(SettingsUpdateCompleted {
                    operation_id,
                    table,
                    key,
                    result: Err(SettingsError::DatabaseError(
                        "Database service not available".into()
                    )),
                    requester,
                });
            });
        }
        return;
    };

    for request in events.read() {
        // Validate table and key before spawning async task
        let record_id = match parse_record_id(&request.table, &request.key) {
            Ok(rid) => rid,
            Err(e) => {
                error!("Invalid table/key for update: {}", e);
                let operation_id = request.operation_id;
                let table = request.table.clone();
                let key = request.key.clone();
                let requester = request.requester;
                
                commands.queue(move |world: &mut World| {
                    world.send_event(SettingsUpdateCompleted {
                        operation_id,
                        table,
                        key,
                        result: Err(e),
                        requester,
                    });
                });
                continue;
            }
        };

        let db = db.as_ref().clone();
        let table = request.table.clone();
        let key = request.key.clone();
        let fields = request.fields.clone();
        let operation_id = request.operation_id;
        let requester = request.requester;

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();

            // Serialize fields without unwrap
            let fields_json = match serde_json::to_string(&fields) {
                Ok(json) => json,
                Err(e) => {
                    let error = SettingsError::SerializationError(e.to_string());
                    command_queue.push(move |world: &mut World| {
                        world.send_event(SettingsUpdateCompleted {
                            operation_id,
                            table,
                            key,
                            result: Err(error),
                            requester,
                        });
                    });
                    return command_queue;
                }
            };

            // Use MERGE with RETURN BEFORE to atomically get old value (task 2.2)
            let merge_query = format!(
                "UPDATE {} MERGE {} RETURN BEFORE",
                record_id, fields_json
            );

            // Clone for use in multiple closures
            let table_for_change = table.clone();
            let key_for_change = key.clone();

            // Execute MERGE query to get old value
            let result = match db.query(&merge_query).await {
                Ok(mut response) => {
                    // RETURN BEFORE gives us the old value atomically
                    let old_value = match response.take::<Option<Value>>(0) {
                        Ok(val) => val,
                        Err(e) => {
                            error!("Failed to extract old value from MERGE RETURN BEFORE: {}", e);
                            None
                        }
                    };

                    // After merge, select to get new value for change event
                    let select_query = format!("SELECT * FROM {}", record_id);
                    let new_value = match db.query(&select_query).await {
                        Ok(mut resp) => {
                            match resp.take::<Option<Value>>(0) {
                                Ok(Some(val)) => val,
                                Ok(None) => {
                                    error!("Record disappeared after merge");
                                    Value::default()  // None value
                                },
                                Err(e) => {
                                    error!("Failed to read new value after merge: {}", e);
                                    Value::default()  // None value
                                }
                            }
                        },
                        Err(e) => {
                            error!("Failed to query new value after merge: {}", e);
                            Value::default()  // None value
                        }
                    };

                    // Emit change notification with proper old_value from RETURN BEFORE
                    command_queue.push(move |world: &mut World| {
                        world.send_event(SettingChanged {
                            table: table_for_change,
                            key: key_for_change,
                            old_value,
                            new_value,
                            changed_at: chrono::Utc::now(),
                        });
                    });
                    Ok(())
                },
                Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
            };

            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(SettingsUpdateCompleted {
                    operation_id,
                    table,
                    key,
                    result,
                    requester,
                });
            });

            command_queue
        });

        commands.spawn(SettingsUpdateTask {
            operation_id,
            table: request.table.clone(),
            key: request.key.clone(),
            requester,
            task,
        });
    }
}

/// Handle settings update task completion
pub fn handle_settings_update_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SettingsUpdateTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Delete Operations
// ============================================================================

/// Process settings delete requests
pub fn process_settings_delete_requests(
    mut commands: Commands,
    mut events: EventReader<SettingsDeleteRequested>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot process delete requests");
        for request in events.read() {
            let operation_id = request.operation_id;
            let table = request.table.clone();
            let key = request.key.clone();
            let requester = request.requester;
            
            commands.queue(move |world: &mut World| {
                world.send_event(SettingsDeleteCompleted {
                    operation_id,
                    table,
                    key,
                    result: Err(SettingsError::DatabaseError(
                        "Database service not available".into()
                    )),
                    requester,
                });
            });
        }
        return;
    };

    for request in events.read() {
        // Validate table and key before spawning async task
        let record_id = match parse_record_id(&request.table, &request.key) {
            Ok(rid) => rid,
            Err(e) => {
                error!("Invalid table/key for delete: {}", e);
                let operation_id = request.operation_id;
                let table = request.table.clone();
                let key = request.key.clone();
                let requester = request.requester;
                
                commands.queue(move |world: &mut World| {
                    world.send_event(SettingsDeleteCompleted {
                        operation_id,
                        table,
                        key,
                        result: Err(e),
                        requester,
                    });
                });
                continue;
            }
        };

        let db = db.as_ref().clone();
        let table = request.table.clone();
        let key = request.key.clone();
        let operation_id = request.operation_id;
        let requester = request.requester;

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();

            // Build DELETE query using validated RecordId (SQL injection safe)
            let query = format!("DELETE {}", record_id);

            // Execute query
            let result = match db.query(&query).await {
                Ok(mut response) => {
                    // Check if record existed (returns deleted record or None)
                    match response.take::<Option<Value>>(0) {
                        Ok(Some(deleted_value)) => {
                            // Emit SettingChanged for deletion (task 2.3)
                            let table_for_change = table.clone();
                            let key_for_change = key.clone();
                            
                            command_queue.push(move |world: &mut World| {
                                world.send_event(SettingChanged {
                                    table: table_for_change,
                                    key: key_for_change,
                                    old_value: Some(deleted_value),
                                    new_value: Value::default(),  // None value for deletion
                                    changed_at: chrono::Utc::now(),
                                });
                            });
                            Ok(true)  // Existed and deleted
                        },
                        Ok(None) => Ok(false),  // Didn't exist
                        Err(e) => Err(SettingsError::QueryFailed(e.to_string())),
                    }
                },
                Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
            };

            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(SettingsDeleteCompleted {
                    operation_id,
                    table,
                    key,
                    result,
                    requester,
                });
            });

            command_queue
        });

        commands.spawn(SettingsDeleteTask {
            operation_id,
            table: request.table.clone(),
            key: request.key.clone(),
            requester,
            task,
        });
    }
}

/// Handle settings delete task completion
pub fn handle_settings_delete_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SettingsDeleteTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Query Operations
// ============================================================================

/// Process settings query requests (raw SurrealQL)
pub fn process_settings_query_requests(
    mut commands: Commands,
    mut events: EventReader<SettingsQueryRequested>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot process query requests");
        
        for request in events.read() {
            let operation_id = request.operation_id;
            let requester = request.requester;
            
            commands.queue(move |world: &mut World| {
                world.send_event(SettingsQueryCompleted {
                    operation_id,
                    result: Err(SettingsError::DatabaseError(
                        "Database service not available".into()
                    )),
                    requester,
                });
            });
        }
        return;
    };

    for request in events.read() {
        let db = db.as_ref().clone();
        let query = request.query.clone();
        let params = request.params.clone();
        let operation_id = request.operation_id;
        let requester = request.requester;

        let task = AsyncComputeTaskPool::get().spawn(async move {
            let mut command_queue = CommandQueue::default();

            // Execute query with or without params
            let result = if let Some(params) = params {
                match db.query_with_params(&query, params).await {
                    Ok(mut response) => {
                        match response.take::<Vec<Value>>(0) {
                            Ok(values) => Ok(values),
                            Err(e) => Err(SettingsError::QueryFailed(e.to_string())),
                        }
                    },
                    Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
                }
            } else {
                match db.query(&query).await {
                    Ok(mut response) => {
                        match response.take::<Vec<Value>>(0) {
                            Ok(values) => Ok(values),
                            Err(e) => Err(SettingsError::QueryFailed(e.to_string())),
                        }
                    },
                    Err(e) => Err(SettingsError::DatabaseError(e.to_string())),
                }
            };

            // Emit completion event
            command_queue.push(move |world: &mut World| {
                world.send_event(SettingsQueryCompleted {
                    operation_id,
                    result,
                    requester,
                });
            });

            command_queue
        });

        commands.spawn(SettingsQueryTask {
            operation_id,
            requester,
            task,
        });
    }
}

/// Handle settings query task completion
pub fn handle_settings_query_tasks(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut SettingsQueryTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.task)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// Audit Trail
// ============================================================================

/// Write audit trail entries to settings_history table
/// 
/// Listens to SettingChanged events and writes complete audit records
/// including old_value, new_value, and timestamp for compliance and debugging.
pub fn write_audit_trail(
    mut events: EventReader<SettingChanged>,
    db_service: Option<Res<DatabaseService>>,
) {
    let Some(db) = db_service else {
        if !events.is_empty() {
            error!("DatabaseService not available - cannot write audit trail");
        }
        return;
    };

    for change_event in events.read() {
        let db = db.as_ref().clone();
        let table = change_event.table.clone();
        let key = change_event.key.clone();
        let old_value = change_event.old_value.clone();
        let new_value = change_event.new_value.clone();
        let changed_at = change_event.changed_at;

        // Spawn async task to write audit entry
        AsyncComputeTaskPool::get().spawn(async move {
            // Generate unique audit ID
            let audit_id = uuid::Uuid::new_v4().to_string();
            
            // Build audit record
            let audit_record = serde_json::json!({
                "table": table,
                "key": key,
                "old_value": old_value,
                "new_value": new_value,
                "changed_at": changed_at,
            });

            // Insert into settings_history table using RecordId for safety
            let record_id = RecordId::from(("settings_history", audit_id.as_str()));
            let query = format!("CREATE {} CONTENT {}", record_id, audit_record);

            match db.query(&query).await {
                Ok(_) => {
                    debug!("Audit entry written: {}:{} changed", table, key);
                },
                Err(e) => {
                    error!("Failed to write audit entry for {}:{}: {}", table, key, e);
                }
            }
        }).detach();
    }
}

// ============================================================================
// Migration
// ============================================================================

/// Run migration from JSON files on first startup
pub fn run_migration_on_startup(
    mut commands: Commands,
    db_service: Option<Res<DatabaseService>>,
    schema_init: Option<Res<SchemaInitialized>>,
    app_dirs: Option<Res<AppDirectories>>,
) {
    // Only run if schema is initialized
    if schema_init.is_none() {
        return;
    }

    let Some(db) = db_service else {
        error!("DatabaseService not available - cannot run migration");
        return;
    };

    let Some(dirs) = app_dirs else {
        error!("AppDirectories not available - cannot determine config path");
        return;
    };

    // Check if migration already run (marker file)
    let config_dir = dirs.config_dir();
    let migration_marker = config_dir.join(".settings-migrated");

    if migration_marker.exists() {
        debug!("Migration already completed, skipping");
        return;
    }

    info!("Starting first-time settings migration");

    let db = db.clone();
    let config_dir_clone = config_dir.clone();

    let task = AsyncComputeTaskPool::get().spawn(async move {
        let mut command_queue = CommandQueue::default();

        match migration::run_migrations(&db, &config_dir_clone).await {
            Ok(_) => {
                // Create marker file to prevent re-running migration
                if let Err(e) = std::fs::write(&migration_marker, "") {
                    error!(
                        "Migration succeeded but failed to write marker file at {:?}: {}. \
                        Migration may run again on next startup.",
                        migration_marker, e
                    );
                }

                command_queue.push(|world: &mut World| {
                    world.insert_resource(MigrationCompleted);
                    info!("Settings migration completed successfully");
                });
            },
            Err(e) => {
                let error_msg = e.to_string();
                error!("Settings migration failed: {}", error_msg);
                command_queue.push(move |world: &mut World| {
                    world.insert_resource(MigrationFailed(error_msg));
                });
            },
        }

        command_queue
    });

    commands.spawn(MigrationTask(task));
}

/// Handle migration task completion
pub fn handle_migration_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut MigrationTask)>,
) {
    for (entity, mut task) in &mut tasks {
        if let Some(mut command_queue) = block_on(future::poll_once(&mut task.0)) {
            commands.append(&mut command_queue);
            commands.entity(entity).despawn();
        }
    }
}
