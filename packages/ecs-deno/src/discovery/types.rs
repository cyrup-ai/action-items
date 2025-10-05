//! Common types and constants for discovery operations
//!
//! This module contains all shared types, constants, and error structures
//! used across the discovery system modules.

use serde::{Deserialize, Serialize};

pub use crate::raycast_errors::*;
// Re-export from raycast_types and raycast_errors
pub use crate::raycast_types::*;

// Static string constants to avoid allocations
pub const UNKNOWN_AUTHOR: &str = "Unknown";
pub const DEFAULT_MODE: &str = "view";
pub const PACKAGE_JSON: &str = "package.json";
pub const EMPTY_STRING: &str = "";

// Security limits to prevent DoS attacks
pub const MAX_DIRECTORY_DEPTH: usize = 10; // Max nested directory depth
pub const MAX_EXTENSIONS_COUNT: usize = 10000; // Max extensions to process
pub const MAX_PACKAGE_JSON_SIZE: u64 = 1024 * 1024; // 1MB max package.json
pub const MAX_STRING_LENGTH: usize = 10000; // Max string field length

// Parallel processing limits to prevent resource exhaustion
pub const MAX_CONCURRENT_PARSING: usize = 32; // Max concurrent extension parsing tasks
pub const BATCH_SIZE: usize = 16; // Process extensions in batches for memory efficiency

// Field name constants for zero-allocation parsing
pub mod fields {
    pub const NAME: &str = "name";
    pub const TITLE: &str = "title";
    pub const DESCRIPTION: &str = "description";
    pub const AUTHOR: &str = "author";
    pub const CATEGORIES: &str = "categories";
    pub const ICON: &str = "icon";
    pub const VERSION: &str = "version";
    pub const KEYWORDS: &str = "keywords";
    pub const COMMANDS: &str = "commands";
    pub const MODE: &str = "mode";
    pub const LICENSE: &str = "license";
    pub const HOMEPAGE: &str = "homepage";
    pub const REPOSITORY: &str = "repository";
    pub const PLACEHOLDER: &str = "placeholder";
    pub const REQUIRED: &str = "required";
    pub const TYPE: &str = "type";
    pub const PREFERENCES: &str = "preferences";
    pub const DEFAULT: &str = "default";
}

/// Error categories for classification
pub mod error_categories {
    pub const FILESYSTEM: &str = "filesystem";
    pub const SERIALIZATION: &str = "serialization";
}

/// Error severity levels
pub mod error_severity {
    pub const CRITICAL: &str = "critical";
    pub const HIGH: &str = "high";
}

/// Error codes for different error scenarios
pub mod error_codes {
    pub const DISCOVERY_FAILED: &str = "DISCOVERY_FAILED";
    pub const SERIALIZATION_ERROR: &str = "SERIALIZATION_ERROR";
}

/// Structured error response for production error handling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: ErrorDetails,
    pub context: ErrorContext,
    pub timestamp: String,
}

/// Detailed error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetails {
    pub code: String,
    pub message: String,
    pub category: String,
    pub severity: String,
}

/// Error context for debugging and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorContext {
    pub operation: String,
    pub path: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
    pub file_size: Option<u64>,
    pub directory_count: Option<usize>,
}
