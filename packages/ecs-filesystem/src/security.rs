use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};

use crate::types::{FileOperationId, FileSystemError};

/// Security configuration for filesystem operations
#[derive(Debug, Clone)]
pub struct SecurityConfig {
    /// Allowed root directories - paths must be within these
    pub allowed_directories: HashSet<PathBuf>,
    /// Maximum file size for read operations (in bytes)
    pub max_file_size: u64,
    /// Allowed file extensions (empty = all allowed)
    pub allowed_extensions: HashSet<String>,
    /// Blocked file extensions (takes precedence over allowed)
    pub blocked_extensions: HashSet<String>,
    /// Whether to follow symlinks (security risk if enabled)
    pub follow_symlinks: bool,
    /// Maximum path depth to prevent excessive recursion
    pub max_path_depth: usize,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            allowed_directories: HashSet::new(),
            max_file_size: 100 * 1024 * 1024, // 100MB default limit
            allowed_extensions: HashSet::new(),
            blocked_extensions: {
                let mut blocked = HashSet::new();
                // Block potentially dangerous executable extensions
                blocked.insert("exe".to_string());
                blocked.insert("bat".to_string());
                blocked.insert("cmd".to_string());
                blocked.insert("com".to_string());
                blocked.insert("scr".to_string());
                blocked.insert("msi".to_string());
                blocked
            },
            follow_symlinks: false,
            max_path_depth: 10,
        }
    }
}

/// Path validator with comprehensive security checks
pub struct PathValidator {
    config: SecurityConfig,
}

impl PathValidator {
    pub fn new(config: SecurityConfig) -> Self {
        Self { config }
    }

    /// Validate path for security and policy compliance
    pub fn validate_path(&self, path: &Path, _operation: &str) -> Result<PathBuf, FileSystemError> {
        // 1. Normalize path to prevent traversal attacks
        let normalized = self.normalize_path(path)?;

        // 2. Check for path traversal attempts
        self.check_path_traversal(&normalized)?;

        // 3. Validate against allowed directories
        self.check_allowed_directories(&normalized)?;

        // 4. Check path depth
        self.check_path_depth(&normalized)?;

        // 5. Validate file extension if applicable
        if normalized.is_file() || path.extension().is_some() {
            self.check_file_extension(&normalized)?;
        }

        Ok(normalized)
    }

    /// Normalize path to prevent traversal attacks and canonicalize it
    fn normalize_path(&self, path: &Path) -> Result<PathBuf, FileSystemError> {
        // Convert to absolute path and canonicalize to resolve .. and . components
        let absolute = if path.is_absolute() {
            path.to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| FileSystemError::Io { source: e })?
                .join(path)
        };

        // Resolve the canonical path to handle symlinks and normalize
        absolute.canonicalize().map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => FileSystemError::NotFound {
                path: path.to_path_buf(),
            },
            _ => FileSystemError::Io { source: e },
        })
    }

    /// Check for path traversal attempts (../, ..\, etc.)
    fn check_path_traversal(&self, path: &Path) -> Result<(), FileSystemError> {
        for component in path.components() {
            match component {
                Component::ParentDir => {
                    return Err(FileSystemError::PathTraversalBlocked {
                        path: path.to_path_buf(),
                    });
                },
                Component::CurDir => {
                    // Current dir is generally safe but can indicate path manipulation
                    continue;
                },
                _ => continue,
            }
        }
        Ok(())
    }

    /// Validate that path is within allowed directories
    fn check_allowed_directories(&self, path: &Path) -> Result<(), FileSystemError> {
        if self.config.allowed_directories.is_empty() {
            // If no restrictions are configured, allow all paths
            return Ok(());
        }

        for allowed_dir in &self.config.allowed_directories {
            if path.starts_with(allowed_dir) {
                return Ok(());
            }
        }

        Err(FileSystemError::PathNotAllowed {
            path: path.to_path_buf(),
        })
    }
    /// Check path depth to prevent excessive recursion
    fn check_path_depth(&self, path: &Path) -> Result<(), FileSystemError> {
        let depth = path.components().count();
        if depth > self.config.max_path_depth {
            return Err(FileSystemError::AccessDenied {
                path: path.to_path_buf(),
            });
        }
        Ok(())
    }

    /// Validate file extension against security policy
    fn check_file_extension(&self, path: &Path) -> Result<(), FileSystemError> {
        if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
            let ext_lower = extension.to_lowercase();

            // Check blocked extensions first (takes precedence)
            if self.config.blocked_extensions.contains(&ext_lower) {
                return Err(FileSystemError::UnsupportedFileType {
                    extension: ext_lower,
                });
            }

            // If allowed extensions are specified, file must be in that list
            if !self.config.allowed_extensions.is_empty()
                && !self.config.allowed_extensions.contains(&ext_lower)
            {
                return Err(FileSystemError::UnsupportedFileType {
                    extension: ext_lower,
                });
            }
        }
        Ok(())
    }

    /// Validate file size for read operations
    pub fn validate_file_size(&self, size: u64) -> Result<(), FileSystemError> {
        if size > self.config.max_file_size {
            return Err(FileSystemError::FileSizeExceeded {
                size,
                limit: self.config.max_file_size,
            });
        }
        Ok(())
    }

    /// Generate audit log entry for filesystem operation
    pub fn audit_log(
        &self,
        operation_id: FileOperationId,
        operation: &str,
        path: &Path,
        success: bool,
    ) {
        tracing::info!(
            operation_id = %operation_id.0,
            operation = operation,
            path = %path.display(),
            success = success,
            "Filesystem operation audit"
        );
    }
}
