//! File system scanning and directory traversal logic
//!
//! This module provides plugin scanning capabilities including directory traversal,
//! path validation, and security constraint checking for discovery operations.

use std::path::{Path, PathBuf};

use tokio::fs;

use super::types::*;
use super::validator::{validate_directory_depth, validate_directory_path};

/// Plugin scanner for file system operations
pub struct PluginScanner {
    pub max_depth: usize,
    pub max_extensions: usize,
}

impl PluginScanner {
    /// Create a new plugin scanner with default limits
    pub fn new() -> Self {
        Self {
            max_depth: MAX_DIRECTORY_DEPTH,
            max_extensions: MAX_EXTENSIONS_COUNT,
        }
    }

    /// Scan directory for plugin extensions
    pub async fn scan_directory(&self, path: &str) -> Result<Vec<PathBuf>, RaycastDiscoveryError> {
        // Validate input path for security
        validate_directory_path(path)?;

        // Collect all valid extension directory paths
        collect_extension_paths(path).await
    }

    /// Validate path security constraints
    pub fn validate_path(&self, path: &str) -> Result<(), RaycastDiscoveryError> {
        validate_directory_path(path)
    }

    /// Check security constraints for a path
    pub fn check_security_constraints(&self, _path: &Path) -> Result<(), RaycastDiscoveryError> {
        // This would be expanded with additional security checks as needed
        Ok(())
    }
}

impl Default for PluginScanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Collect all valid extension directory paths for parallel processing
#[inline]
pub async fn collect_extension_paths(
    extensions_dir: &str,
) -> Result<Vec<PathBuf>, RaycastDiscoveryError> {
    let mut extension_paths = Vec::new();
    let mut read_dir = fs::read_dir(extensions_dir)
        .await
        .map_err(|e| create_filesystem_error(extensions_dir, "read_directory", e))?;

    let mut extension_count = 0;
    while let Some(entry) = read_dir
        .next_entry()
        .await
        .map_err(|e| create_filesystem_error(extensions_dir, "read_directory_entry", e))?
    {
        // Security: limit number of extensions to prevent DoS
        if extension_count >= MAX_EXTENSIONS_COUNT {
            return Err(RaycastDiscoveryError::SecurityViolation {
                path: extensions_dir.to_string(),
                violation: "too_many_extensions".to_string(),
                message: format!(
                    "Directory contains more than {} extensions",
                    MAX_EXTENSIONS_COUNT
                ),
            });
        }

        let path = entry.path();

        // Skip non-directories
        if !entry
            .file_type()
            .await
            .map_err(|e| create_filesystem_error(&path.to_string_lossy(), "get_file_type", e))?
            .is_dir()
        {
            continue;
        }

        // Validate directory depth for security
        validate_directory_depth(&path, extensions_dir)?;

        // Check if directory contains package.json
        let package_json_path = path.join(PACKAGE_JSON);
        if package_json_path.exists() {
            extension_paths.push(path);
            extension_count += 1;
        }
    }

    extension_paths.shrink_to_fit();
    Ok(extension_paths)
}

/// Create filesystem error with context
#[cold]
#[inline(never)]
fn create_filesystem_error(
    path: &str,
    operation: &'static str,
    source: std::io::Error,
) -> RaycastDiscoveryError {
    RaycastDiscoveryError::Filesystem {
        path: path.to_string(),
        operation: operation.to_string(),
        source: source.to_string(),
    }
}

/// Plugin scanning trait for testability
pub trait PluginScanning {
    fn scan_directory(
        &self,
        path: &str,
    ) -> impl std::future::Future<Output = Result<Vec<PathBuf>, RaycastDiscoveryError>> + Send;
    fn validate_path(&self, path: &str) -> Result<(), RaycastDiscoveryError>;
    fn check_security_constraints(&self, path: &Path) -> Result<(), RaycastDiscoveryError>;
}

impl PluginScanning for PluginScanner {
    async fn scan_directory(&self, path: &str) -> Result<Vec<PathBuf>, RaycastDiscoveryError> {
        self.scan_directory(path).await
    }

    fn validate_path(&self, path: &str) -> Result<(), RaycastDiscoveryError> {
        self.validate_path(path)
    }

    fn check_security_constraints(&self, path: &Path) -> Result<(), RaycastDiscoveryError> {
        self.check_security_constraints(path)
    }
}
