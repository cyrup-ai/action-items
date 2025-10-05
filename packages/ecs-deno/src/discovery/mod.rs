//! Public API and module coordination for discovery operations
//!
//! This module provides the main orchestration and public API for the discovery system,
//! coordinating all the specialized modules to provide comprehensive plugin discovery.

use std::path::{Path, PathBuf};

use deno_core::op2;
use futures::future::join_all;

pub mod indexer;
pub mod metadata_parser;
pub mod plugin_scanner;
pub mod types;
pub mod validator;
pub mod watcher;

pub use indexer::{CachedExtension, DiscoveryIndexer, StringInterner};
pub use metadata_parser::{MetadataParser, MetadataParsing};
pub use plugin_scanner::{PluginScanner, PluginScanning, collect_extension_paths};
pub use types::*;
pub use validator::{
    SecurityValidator, validate_directory_depth, validate_directory_path, validate_file_size,
    validate_string_field,
};
pub use watcher::{DiscoveryEvent, PluginWatcher};

/// Main discovery orchestrator that coordinates all modules
pub struct DiscoveryOrchestrator {
    pub scanner: PluginScanner,
    pub validator: SecurityValidator,
    pub indexer: DiscoveryIndexer,
    pub watcher: Option<PluginWatcher>,
}

impl DiscoveryOrchestrator {
    /// Create a new discovery orchestrator
    pub fn new() -> Self {
        Self {
            scanner: PluginScanner::new(),
            validator: SecurityValidator::new(),
            indexer: DiscoveryIndexer::new(),
            watcher: None,
        }
    }

    /// Enable file system watching
    pub fn with_watcher(mut self) -> Result<Self, RaycastDiscoveryError> {
        self.watcher = Some(PluginWatcher::new()?);
        Ok(self)
    }

    /// Discover extensions using all coordinated modules
    pub async fn discover_extensions(
        &self,
        extensions_dir: &str,
    ) -> Result<Vec<IsolatedRaycastExtension>, RaycastDiscoveryError> {
        discover_extensions_internal(extensions_dir).await
    }
}

impl Default for DiscoveryOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Discovers Raycast extensions from the specified directory with zero-allocation optimizations
#[op2(async)]
#[string]
pub async fn op_discover_raycast_extensions(#[string] extensions_dir: String) -> String {
    let extensions = match discover_extensions_internal(&extensions_dir).await {
        Ok(exts) => exts,
        Err(e) => {
            return create_discovery_error_response(&extensions_dir, e);
        },
    };

    match serde_json::to_string(&extensions) {
        Ok(json) => json,
        Err(e) => create_serialization_error_response(e),
    }
}

/// Internal implementation with aggressive optimizations
#[inline]
pub async fn discover_extensions_internal(
    extensions_dir: &str,
) -> Result<Vec<IsolatedRaycastExtension>, RaycastDiscoveryError> {
    // Validate input path for security
    validate_directory_path(extensions_dir)?;

    // Initialize string interner for zero-allocation string handling
    let interner = StringInterner::new();

    // Collect all valid directory paths first for parallel processing
    let extension_paths = collect_extension_paths(extensions_dir).await?;

    // Process extensions in parallel batches for optimal performance
    let mut extensions = process_extensions_parallel(extension_paths, &interner).await?;

    // Shrink to fit to return minimum memory
    extensions.shrink_to_fit();
    Ok(extensions)
}

/// Process extensions in parallel batches for optimal performance
async fn process_extensions_parallel(
    extension_paths: Vec<PathBuf>,
    interner: &StringInterner,
) -> Result<Vec<IsolatedRaycastExtension>, RaycastDiscoveryError> {
    let mut extensions = Vec::with_capacity(extension_paths.len());

    // Process extensions in batches to control memory usage
    for batch in extension_paths.chunks(BATCH_SIZE) {
        let batch_futures: Vec<_> = batch
            .iter()
            .map(|path| process_single_extension(path, interner))
            .collect();

        // Limit concurrent processing to prevent resource exhaustion
        let batch_results = join_all(batch_futures).await;

        // Collect successful results and handle errors
        for result in batch_results {
            match result {
                Ok(extension) => extensions.push(extension),
                Err(_) => {
                    // Log error but continue processing other extensions
                    // In production, you might want to collect and report these errors
                    continue;
                },
            }
        }
    }

    Ok(extensions)
}

/// Process a single extension directory
async fn process_single_extension(
    extension_path: &Path,
    interner: &StringInterner,
) -> Result<IsolatedRaycastExtension, RaycastDiscoveryError> {
    let package_json_path = extension_path.join(PACKAGE_JSON);

    // Read and validate package.json
    let metadata = tokio::fs::metadata(&package_json_path)
        .await
        .map_err(|e| handle_file_read_error(e, &package_json_path, extension_path))?;

    validate_file_size(metadata.len(), &package_json_path)?;

    let content = tokio::fs::read_to_string(&package_json_path)
        .await
        .map_err(|e| handle_file_read_error(e, &package_json_path, extension_path))?;

    // Parse extension using metadata parser
    let parser = MetadataParser::new(interner);
    parser.parse_extension(&content, extension_path)
}

/// Create structured error response for discovery failures
#[cold]
#[inline(never)]
fn create_discovery_error_response(extensions_dir: &str, error: RaycastDiscoveryError) -> String {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: error_codes::DISCOVERY_FAILED.to_string(),
            message: format!("Failed to discover extensions: {}", error),
            category: error_categories::FILESYSTEM.to_string(),
            severity: error_severity::HIGH.to_string(),
        },
        context: ErrorContext {
            operation: "discover_extensions".to_string(),
            path: Some(extensions_dir.to_string()),
            line: None,
            column: None,
            file_size: None,
            directory_count: None,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    match serde_json::to_string(&error_response) {
        Ok(json) => json,
        Err(_) => r#"{"error": {"code": "SERIALIZATION_FAILED", "message": "Failed to serialize error response"}}"#.to_string(),
    }
}

/// Create structured error response for serialization failures
#[cold]
#[inline(never)]
fn create_serialization_error_response(error: serde_json::Error) -> String {
    let error_response = ErrorResponse {
        error: ErrorDetails {
            code: error_codes::SERIALIZATION_ERROR.to_string(),
            message: format!("Failed to serialize extensions: {}", error),
            category: error_categories::SERIALIZATION.to_string(),
            severity: error_severity::CRITICAL.to_string(),
        },
        context: ErrorContext {
            operation: "serialize_extensions".to_string(),
            path: None,
            line: Some(error.line() as u32),
            column: Some(error.column() as u32),
            file_size: None,
            directory_count: None,
        },
        timestamp: chrono::Utc::now().to_rfc3339(),
    };

    match serde_json::to_string(&error_response) {
        Ok(json) => json,
        Err(_) => r#"{"error": {"code": "CRITICAL_SERIALIZATION_FAILURE", "message": "Cannot serialize error response"}}"#.to_string(),
    }
}

#[cold]
#[inline(never)]
fn handle_file_read_error(
    error: std::io::Error,
    package_json_path: &Path,
    extension_path: &Path,
) -> RaycastDiscoveryError {
    match error.kind() {
        std::io::ErrorKind::NotFound => RaycastDiscoveryError::MissingFiles {
            extension_path: extension_path.to_string_lossy().into_owned(),
            missing_files: vec![PACKAGE_JSON.to_string()],
        },
        std::io::ErrorKind::PermissionDenied => RaycastDiscoveryError::PermissionDenied {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
        },
        _ => RaycastDiscoveryError::Filesystem {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
            source: error.to_string(),
        },
    }
}
