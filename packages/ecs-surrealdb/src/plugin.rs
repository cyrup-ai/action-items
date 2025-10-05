//! Bevy plugin integration for DatabaseService

use bevy::ecs::world::CommandQueue;
use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on};
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
use tracing::{debug, error, info, warn};

use crate::config::DatabaseConfig;
use crate::service::{DatabaseService, DatabaseServiceError, DatabaseShutdown};

/// Database service plugin for Bevy
pub struct DatabasePlugin {
    config: DatabaseConfig,
}

impl DatabasePlugin {
    pub fn new(config: DatabaseConfig) -> Self {
        Self { config }
    }
}

#[derive(Component)]
struct DatabaseInitTask(Task<CommandQueue>);

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        app.add_systems(PostStartup, move |mut commands: Commands| {
            let config = config.clone();
            debug!("Starting database initialization task with panic protection");
            
            let task = AsyncComputeTaskPool::get().spawn(async move {
                let mut command_queue = CommandQueue::default();
                
                debug!("Database async task started with timeout and panic catching");
                
                // Wrap entire database initialization in panic catching (remove timeout for Bevy compatibility)
                let initialization_result = AssertUnwindSafe(async move {
                    debug!("Attempting database service creation");
                    DatabaseService::new(config).await
                }).catch_unwind().await;
                
                // Handle all possible outcomes: success, error, panic  
                match initialization_result {
                    Ok(Ok(service)) => {
                        debug!("Database service creation succeeded");
                        command_queue.push(move |world: &mut World| {
                            // Safely check if resource already exists to prevent conflicts
                            if world.get_resource::<DatabaseService>().is_none() {
                                world.insert_resource(service);
                                info!("Database service initialized successfully");
                            } else {
                                warn!("Database service resource already exists, skipping initialization");
                            }
                        });
                    },
                    Ok(Err(database_error)) => {
                        error!("Database initialization failed with error: {}", database_error);
                        let error_msg = database_error.to_string();
                        command_queue.push(move |world: &mut World| {
                            world.insert_resource(DatabaseServiceError(error_msg));
                            warn!("Database service unavailable - operations will fail gracefully");
                        });
                    },
                    Err(_panic_payload) => {
                        error!("Database initialization panicked - this indicates a serious bug");
                        command_queue.push(move |world: &mut World| {
                            world.insert_resource(DatabaseServiceError(
                                "Database initialization panicked - check logs for details".to_string()
                            ));
                            error!("Database service unavailable due to panic");
                        });
                    },
                }
                
                debug!("Database async task completed with all error cases handled");
                command_queue
            });

            commands.spawn(DatabaseInitTask(task));
        })
        .add_systems(Update, handle_database_init_task)
        .add_systems(Last, handle_database_shutdown);
    }
}

fn handle_database_init_task(
    mut commands: Commands,
    mut tasks: Query<(Entity, &mut DatabaseInitTask)>,
) {
    for (entity, mut task) in &mut tasks {
        debug!("Polling database initialization task");
        
        // Use panic catching for task polling to prevent system-level panics
        let poll_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            block_on(future::poll_once(&mut task.0))
        }));
        
        match poll_result {
            Ok(Some(mut command_queue)) => {
                debug!("Database initialization task completed successfully, applying commands");
                
                // Use panic catching for command queue application
                let apply_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
                    commands.append(&mut command_queue);
                }));
                
                match apply_result {
                    Ok(()) => {
                        debug!("Commands applied successfully");
                        commands.entity(entity).despawn();
                        debug!("Database initialization task cleaned up");
                    },
                    Err(_panic) => {
                        error!("Command queue application panicked - database service may be in inconsistent state");
                        // Insert error resource to indicate failure
                        commands.insert_resource(DatabaseServiceError(
                            "Database initialization completed but command application failed".to_string()
                        ));
                        commands.entity(entity).despawn();
                        warn!("Database initialization task cleaned up after command panic");
                    },
                }
            },
            Ok(None) => {
                // Task still running, continue polling next frame
                debug!("Database initialization task still running");
            },
            Err(_panic) => {
                error!("Database initialization task polling panicked - this is a serious bug");
                commands.insert_resource(DatabaseServiceError(
                    "Database initialization task polling failed".to_string()
                ));
                commands.entity(entity).despawn();
                error!("Database initialization task cleaned up after polling panic");
            },
        }
    }
}

fn handle_database_shutdown(
    mut commands: Commands,
    database: Option<Res<DatabaseService>>,
    shutdown: Option<Res<DatabaseShutdown>>,
) {
    if let (Some(_shutdown), Some(db_service)) = (shutdown.as_ref(), database.as_ref()) {
        debug!("Database shutdown requested");
        
        // Use panic catching for database shutdown
        let shutdown_result = std::panic::catch_unwind(AssertUnwindSafe(|| {
            block_on(db_service.shutdown())
        }));
        
        match shutdown_result {
            Ok(Ok(())) => {
                debug!("Database shutdown completed successfully");
            },
            Ok(Err(e)) => {
                warn!("Database shutdown failed with error: {}", e);
            },
            Err(_panic) => {
                error!("Database shutdown panicked - resources will be cleaned up anyway");
            },
        }
        
        // Always clean up resources regardless of shutdown success/failure
        commands.remove_resource::<DatabaseService>();
        commands.remove_resource::<DatabaseShutdown>();
        debug!("Database resources cleaned up");
    }
}