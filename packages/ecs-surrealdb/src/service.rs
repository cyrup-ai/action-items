//! Core database service implementation with connection management

use std::time::Duration;
use bevy::prelude::*;
use serde::Serialize;
use surrealdb::engine::local::{Db, SurrealKv};
use surrealdb::opt::Config;
use surrealdb::opt::capabilities::Capabilities;
use surrealdb::opt::auth::Root;
use surrealdb::{RecordId, Response, Surreal, Value};
use tracing::{debug, info, warn};

use crate::config::{DatabaseConfig, DatabaseEngine, DatabaseError, validate_table_name};

/// Resource indicating database service is unavailable
#[derive(Resource)]
pub struct DatabaseServiceError(pub String);

/// Database shutdown resource for tracking shutdown state
#[derive(Resource)]
pub struct DatabaseShutdown;

/// Main database service - uses proper SurrealDB v3.0 instance pattern
#[derive(Resource, Clone)]
pub struct DatabaseService {
    config: DatabaseConfig,
    db: Surreal<Db>,
}

impl DatabaseService {
    /// Check if database service is available
    pub fn is_available(world: &World) -> bool {
        world.get_resource::<DatabaseService>().is_some()
            && world.get_resource::<DatabaseServiceError>().is_none()
    }
}

impl DatabaseService {
    /// Create a new database service
    pub async fn new(config: DatabaseConfig) -> Result<Self, DatabaseError> {
        Self::new_with_retries(config, 3).await
    }

    /// Create a new database service with connection retries
    pub async fn new_with_retries(
        config: DatabaseConfig,
        max_retries: u32,
    ) -> Result<Self, DatabaseError> {
        config.validate()?;

        if max_retries == 0 {
            return Err(DatabaseError::InvalidConfiguration(
                "max_retries must be at least 1".into(),
            ));
        }

        let mut last_error = DatabaseError::ConnectionFailed("No connection attempts made".into());
        for attempt in 1..=max_retries {
            match Self::try_connect(&config).await {
                Ok(service) => return Ok(service),
                Err(e) => {
                    last_error = e;
                    if attempt < max_retries {
                        warn!(
                            "Database connection attempt {} failed, retrying...",
                            attempt
                        );
                        async_std::task::sleep(Duration::from_millis(1000 * attempt as u64)).await;
                    }
                },
            }
        }
        Err(last_error)
    }

    async fn try_connect(config: &DatabaseConfig) -> Result<Self, DatabaseError> {
        config.validate()?;

        // Ensure storage directory exists for SurrealKv - create the full storage path
        let DatabaseEngine::SurrealKv(storage_path) = &config.engine;
        debug!("Creating storage directory: {:?}", storage_path);
        std::fs::create_dir_all(storage_path).map_err(|e| {
            DatabaseError::InvalidConfiguration(
                format!("Failed to create storage directory {:?}: {}", storage_path, e)
            )
        })?;

        // Initialize SurrealDB v3.0 using proper SDK authentication pattern from tests
        let db = match &config.engine {
            DatabaseEngine::SurrealKv(storage_path) => {
                debug!("Creating SurrealKV connection to: {:?}", storage_path);
                
                // Create root credentials using pattern from SDK tests (lines 470-481)
                // Use static strings to avoid lifetime issues - matching SDK test pattern
                let root = Root {
                    username: "root",
                    password: "root",
                };
                
                // Use proper Config pattern with root user and all capabilities
                let sdb_config = Config::new()
                    .user(root)
                    .capabilities(if config.enable_all_capabilities {
                        Capabilities::all()
                    } else {
                        Capabilities::none()
                    });
                
                debug!("Initializing SurrealKV with root credentials and proper config");
                // Use the correct SurrealDB v3.0 initialization pattern
                let result = Surreal::new::<SurrealKv>((storage_path.clone(), sdb_config)).await;
                debug!("SurrealKV initialization result: {:?}", result.is_ok());
                let db = result.map_err(|e| {
                    warn!("SurrealKV initialization failed: {}", e);
                    DatabaseError::ConnectionFailed(e.to_string())
                })?;
                
                // Sign in using root credentials (required for embedded databases)
                debug!("Signing in with root credentials");
                db.signin(root).await.map_err(|e| {
                    warn!("SurrealKV signin failed: {}", e);
                    DatabaseError::ConnectionFailed(format!("Authentication failed: {}", e))
                })?;
                debug!("SurrealKV signin successful");
                
                db
            },
        };

        // Use namespace and database
        db.use_ns(&config.namespace)
            .use_db(&config.database)
            .await
            .map_err(|e| DatabaseError::ConnectionFailed(e.to_string()))?;

        info!(
            "Database connected: {}/{}",
            config.namespace, config.database
        );
        Ok(Self {
            config: config.clone(),
            db,
        })
    }

    /// Execute a raw SurrealQL query
    pub async fn query(&self, sql: &str) -> Result<Response, DatabaseError> {
        if self.config.enable_query_logging {
            debug!("Executing query: {}", sql);
        }

        self.db.query(sql)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Execute a query with parameters
    pub async fn query_with_params(
        &self,
        sql: &str,
        params: std::collections::HashMap<String, Value>,
    ) -> Result<Response, DatabaseError> {
        if self.config.enable_query_logging {
            debug!(
                "Executing parameterized query: {} with {} parameters",
                sql,
                params.len()
            );
        }

        self.db.query(sql).bind(params)
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Create a record in a table
    pub async fn create<T>(&self, table: &str, data: T) -> Result<RecordId, DatabaseError>
    where
        T: Serialize + Send + 'static,
    {
        validate_table_name(table)?;

        let result: Result<Option<RecordId>, _> = self.db.create(table).content(data).await;

        match result {
            Ok(Some(record)) => Ok(record),
            Ok(None) => Err(DatabaseError::QueryFailed("No record created".to_string())),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Select records from a table
    pub async fn select<T>(&self, table: &str) -> Result<Vec<T>, DatabaseError>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        validate_table_name(table)?;

        let result: Result<Vec<T>, _> = self.db.select(table).await;

        result.map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Update a record by ID
    pub async fn update<T>(&self, thing: &RecordId, data: T) -> Result<Option<T>, DatabaseError>
    where
        T: Serialize + for<'de> serde::Deserialize<'de> + Send + 'static,
    {
        let result: Result<Vec<T>, _> = self.db.update(thing.to_string()).content(data).await;

        match result {
            Ok(mut records) if !records.is_empty() => Ok(Some(records.remove(0))),
            Ok(_) => Ok(None),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Delete a record by ID
    pub async fn delete(&self, thing: &RecordId) -> Result<Option<Value>, DatabaseError> {
        let result: Result<Vec<Value>, _> = self.db.delete(thing.to_string()).await;

        match result {
            Ok(mut records) if !records.is_empty() => Ok(Some(records.remove(0))),
            Ok(_) => Ok(None),
            Err(e) => Err(DatabaseError::QueryFailed(e.to_string())),
        }
    }

    /// Health check using SurrealDB's internal health management
    pub async fn health_check(&self) -> Result<(), DatabaseError> {
        self.db.health()
            .await
            .map_err(|e| DatabaseError::QueryFailed(e.to_string()))
    }

    /// Gracefully shutdown the database connection
    pub async fn shutdown(&self) -> Result<(), DatabaseError> {
        debug!("Shutting down database service");

        // Attempt graceful shutdown operations
        if let Err(e) = self.health_check().await {
            warn!("Database health check failed during shutdown: {}", e);
        }

        // Connection cleanup handled by SurrealDB SDK on drop
        info!("Database service shutdown complete");
        Ok(())
    }

    /// Get reference to database instance for transactions
    pub(crate) fn db(&self) -> &Surreal<Db> {
        &self.db
    }
}