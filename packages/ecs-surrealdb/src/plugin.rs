//! Bevy plugin integration for DatabaseService

use bevy::prelude::*;
use bevy::tasks::block_on;
use bevy_tokio_tasks::TokioTasksRuntime;
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

impl Plugin for DatabasePlugin {
    fn build(&self, app: &mut App) {
        let config = self.config.clone();

        app.add_systems(Startup, move |tokio_runtime: Res<TokioTasksRuntime>| {
            let config = config.clone();
            debug!("Starting database initialization using shared Tokio runtime");
            
            tokio_runtime.spawn_background_task(|mut ctx| async move {
                debug!("Database async task started in shared Tokio runtime");
                
                // Perform database initialization in Tokio context
                match DatabaseService::new(config).await {
                    Ok(service) => {
                        debug!("Database service creation succeeded");
                        
                        // Insert resource on main thread
                        ctx.run_on_main_thread(move |ctx| {
                            if ctx.world.get_resource::<DatabaseService>().is_none() {
                                ctx.world.insert_resource(service);
                                info!("Database service initialized successfully");
                            } else {
                                warn!("Database service resource already exists");
                            }
                        }).await;
                    }
                    Err(database_error) => {
                        error!("Database initialization failed: {}", database_error);
                        let error_msg = database_error.to_string();
                        
                        // Insert error resource on main thread
                        ctx.run_on_main_thread(move |ctx| {
                            ctx.world.insert_resource(DatabaseServiceError(error_msg));
                            warn!("Database service unavailable - operations will fail gracefully");
                        }).await;
                    }
                }
            });
        })
        .add_systems(Last, handle_database_shutdown);
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