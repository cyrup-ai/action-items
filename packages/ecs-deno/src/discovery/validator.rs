//! Security validation and constraint enforcement
//!
//! This module provides comprehensive security validation for discovery operations,
//! including path traversal prevention, file size limits, and string field validation.

use std::path::Path;

use super::types::*;

/// Security validator for discovery operations
pub struct SecurityValidator {
    pub max_file_size: u64,
    pub max_string_length: usize,
    pub max_directory_depth: usize,
}

impl SecurityValidator {
    /// Create a new security validator with default limits
    pub fn new() -> Self {
        Self {
            max_file_size: MAX_PACKAGE_JSON_SIZE,
            max_string_length: MAX_STRING_LENGTH,
            max_directory_depth: MAX_DIRECTORY_DEPTH,
        }
    }

    /// Validate extension structure and security constraints
    pub fn validate_extension(
        &self,
        extension: &IsolatedRaycastExtension,
    ) -> Result<(), RaycastValidationError> {
        // Validate required fields
        if extension.id.is_empty() {
            return Err(RaycastValidationError::MissingRequired {
                field: "id".to_string(),
                context: "extension".to_string(),
            });
        }

        if extension.name.is_empty() {
            return Err(RaycastValidationError::MissingRequired {
                field: "name".to_string(),
                context: "extension".to_string(),
            });
        }

        // Validate string field lengths
        self.validate_string_length(&extension.name, "name")?;
        self.validate_string_length(&extension.title, "title")?;
        self.validate_string_length(&extension.description, "description")?;
        self.validate_string_length(&extension.author, "author")?;

        Ok(())
    }

    /// Validate string field length
    fn validate_string_length(
        &self,
        value: &str,
        field_name: &str,
    ) -> Result<(), RaycastValidationError> {
        if value.len() > self.max_string_length {
            return Err(RaycastValidationError::ConstraintViolation {
                field: field_name.to_string(),
                constraint: format!("max_length_{}", self.max_string_length),
                value: format!("{}...", &value[..50.min(value.len())]),
            });
        }
        Ok(())
    }
}

impl Default for SecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Validate directory path for security issues
#[inline]
pub fn validate_directory_path(path: &str) -> Result<(), RaycastDiscoveryError> {
    // Check for path traversal attempts
    if path.contains("..") || path.contains("~") {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "path_traversal".to_string(),
            message: "Path contains potentially dangerous components".to_string(),
        });
    }

    // Check for null bytes
    if path.contains('\0') {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "null_byte".to_string(),
            message: "Path contains null bytes".to_string(),
        });
    }

    // Check for extremely long paths
    if path.len() > 4096 {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string(),
            violation: "path_too_long".to_string(),
            message: "Path exceeds maximum length".to_string(),
        });
    }

    Ok(())
}

/// Validate file size to prevent DoS attacks
#[inline]
pub fn validate_file_size(size: u64, file_path: &Path) -> Result<(), RaycastDiscoveryError> {
    if size > MAX_PACKAGE_JSON_SIZE {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: file_path.to_string_lossy().into_owned(),
            violation: "file_too_large".to_string(),
            message: format!(
                "File size {} bytes exceeds maximum {}",
                size, MAX_PACKAGE_JSON_SIZE
            ),
        });
    }
    Ok(())
}

/// Validate directory depth to prevent DoS attacks
#[inline]
pub fn validate_directory_depth(path: &Path, base_dir: &str) -> Result<(), RaycastDiscoveryError> {
    let relative_path = match path.strip_prefix(base_dir) {
        Ok(rel) => rel,
        Err(_) => return Ok(()), // Not under base directory, allow
    };

    let depth = relative_path.components().count();
    if depth > MAX_DIRECTORY_DEPTH {
        return Err(RaycastDiscoveryError::SecurityViolation {
            path: path.to_string_lossy().into_owned(),
            violation: "directory_too_deep".to_string(),
            message: format!(
                "Directory depth {} exceeds maximum {}",
                depth, MAX_DIRECTORY_DEPTH
            ),
        });
    }

    Ok(())
}

/// Validate and truncate string fields to prevent DoS attacks
#[inline]
pub fn validate_string_field(input: &str) -> String {
    if input.len() <= MAX_STRING_LENGTH {
        input.to_string()
    } else {
        // Truncate oversized strings and add indicator
        let mut truncated = input
            .chars()
            .take(MAX_STRING_LENGTH - 3)
            .collect::<String>();
        truncated.push_str("...");
        truncated
    }
}
