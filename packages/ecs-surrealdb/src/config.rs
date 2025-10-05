//! Database configuration, error types, and validation

use std::path::Path;
use action_items_common::AppDirectories;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Database service errors
#[derive(Error, Debug)]
pub enum DatabaseError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Query execution failed: {0}")]
    QueryFailed(String),
    #[error("Transaction failed: {0}")]
    TransactionFailed(String),
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    #[error("Timeout: operation took longer than {timeout_ms}ms")]
    Timeout { timeout_ms: u64 },
}

/// Database engine type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DatabaseEngine {
    /// SurrealKV persistent storage with path
    SurrealKv(std::path::PathBuf),
}

/// Database root credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseCredentials {
    /// Root username for embedded database
    pub username: String,
    /// Root password for embedded database
    pub password: String,
}

impl Default for DatabaseCredentials {
    fn default() -> Self {
        Self {
            username: "root".to_string(),
            password: "root".to_string(),
        }
    }
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database namespace
    pub namespace: String,
    /// Database name
    pub database: String,
    /// Database engine type
    pub engine: DatabaseEngine,
    /// Query timeout in milliseconds
    pub query_timeout_ms: u64,
    /// Enable query logging
    pub enable_query_logging: bool,
    /// Root credentials for embedded database
    pub credentials: DatabaseCredentials,
    /// Enable all capabilities (scripting, live queries, etc.)
    pub enable_all_capabilities: bool,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        let app_dirs = AppDirectories::new();
        let db_path = app_dirs.config_dir().join("database");
        Self {
            namespace: "action_items".to_string(),
            database: "main".to_string(),
            engine: DatabaseEngine::SurrealKv(db_path),
            query_timeout_ms: 10000,
            enable_query_logging: true,
            credentials: DatabaseCredentials::default(),
            enable_all_capabilities: true,
        }
    }
}

impl DatabaseConfig {
    /// Create configuration for persistent SurrealKV storage
    pub fn surreal_kv<P: Into<std::path::PathBuf>>(storage_path: P) -> Self {
        Self {
            engine: DatabaseEngine::SurrealKv(storage_path.into()),
            ..Default::default()
        }
    }

    /// Validate configuration values
    pub fn validate(&self) -> Result<(), DatabaseError> {
        if self.query_timeout_ms == 0 || self.query_timeout_ms > 300_000 {
            return Err(DatabaseError::InvalidConfiguration(
                "Query timeout must be between 1ms and 300,000ms".into(),
            ));
        }

        if !Self::is_valid_identifier(&self.namespace) {
            return Err(DatabaseError::InvalidConfiguration(
                "Invalid namespace: must contain only alphanumeric characters, hyphens, and \
                 underscores"
                    .into(),
            ));
        }

        if !Self::is_valid_identifier(&self.database) {
            return Err(DatabaseError::InvalidConfiguration(
                "Invalid database name: must contain only alphanumeric characters, hyphens, and \
                 underscores"
                    .into(),
            ));
        }

        // Validate storage path security
        let DatabaseEngine::SurrealKv(path) = &self.engine;
        Self::validate_storage_path(path)?;

        Ok(())
    }

    fn is_valid_identifier(name: &str) -> bool {
        !name.is_empty()
            && name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
            && name.chars().next().is_some_and(|c| c.is_alphabetic())
    }

    fn validate_storage_path(path: &Path) -> Result<(), DatabaseError> {
        // Prevent path traversal attacks
        if path.to_string_lossy().contains("..") {
            return Err(DatabaseError::InvalidConfiguration(
                "Storage path cannot contain path traversal sequences".into(),
            ));
        }

        // Ensure path is not root or system directory
        if path == Path::new("/")
            || path.starts_with("/sys")
            || path.starts_with("/proc")
            || path.starts_with("/dev")
        {
            return Err(DatabaseError::InvalidConfiguration(
                "Storage path cannot be system directory".into(),
            ));
        }

        Ok(())
    }
}

/// Validate table name
pub fn validate_table_name(table: &str) -> Result<(), DatabaseError> {
    if table.is_empty() || table.len() > 64 {
        return Err(DatabaseError::InvalidConfiguration(
            "Table name must be 1-64 characters".into(),
        ));
    }
    if !table
        .chars()
        .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
    {
        return Err(DatabaseError::InvalidConfiguration(
            "Table name must contain only alphanumeric characters, hyphens, and underscores".into(),
        ));
    }
    Ok(())
}