//! Comprehensive error types for raycast operations
//!
//! This module provides a complete error type system for all raycast operations
//! including discovery, execution, and validation errors with proper error chaining,
//! context preservation, and conversion traits.

use std::error::Error;
use std::{fmt, io};

use serde::{Deserialize, Serialize};

/// Comprehensive error type for raycast discovery operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RaycastDiscoveryError {
    /// Filesystem-related discovery errors
    Filesystem {
        path: String,
        operation: String,
        source: String,
    },
    /// JSON/YAML parsing errors during discovery
    Parsing {
        file_path: String,
        line: Option<u32>,
        column: Option<u32>,
        message: String,
    },
    /// Invalid extension structure
    InvalidStructure {
        path: String,
        field: String,
        expected: String,
        found: String,
    },
    /// Missing required files
    MissingFiles {
        extension_path: String,
        missing_files: Vec<String>,
    },
    /// Permission denied during discovery
    PermissionDenied { path: String, operation: String },
    /// Security violation during discovery
    SecurityViolation {
        path: String,
        violation: String,
        message: String,
    },
}
/// Comprehensive error type for raycast execution operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RaycastExecutionError {
    /// Runtime errors during command execution
    Runtime {
        command: String,
        exit_code: Option<i32>,
        stdout: String,
        stderr: String,
        duration_ms: u64,
    },
    /// Command not found or invalid
    CommandNotFound {
        command: String,
        available_commands: Vec<String>,
    },
    /// Timeout during execution
    Timeout {
        command: String,
        timeout_ms: u64,
        partial_output: String,
    },
    /// Resource exhaustion (memory, CPU, etc.)
    ResourceExhausted {
        resource_type: String,
        limit: String,
        requested: String,
    },
    /// Plugin crash or unexpected termination
    PluginCrash {
        plugin_id: String,
        signal: Option<String>,
        last_output: String,
    },
    /// Environment setup failure
    EnvironmentError {
        variable: String,
        expected: String,
        found: Option<String>,
    },
}
/// Comprehensive error type for raycast validation operations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RaycastValidationError {
    /// Schema validation errors
    Schema {
        field: String,
        expected_type: String,
        found_type: String,
        value: String,
    },
    /// Security validation errors
    Security {
        violation_type: String,
        resource: String,
        attempted_action: String,
        reason: String,
    },
    /// Required field missing
    MissingRequired { field: String, context: String },
    /// Invalid value format
    InvalidFormat {
        field: String,
        expected_format: String,
        provided_value: String,
    },
    /// Constraint violation
    ConstraintViolation {
        field: String,
        constraint: String,
        value: String,
    },
    /// Dependency validation failure
    DependencyFailure {
        dependent_field: String,
        required_field: String,
        condition: String,
    },
}
/// Main raycast error type that encompasses all error categories
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RaycastError {
    /// Discovery-related errors
    Discovery(RaycastDiscoveryError),
    /// Execution-related errors
    Execution(RaycastExecutionError),
    /// Validation-related errors
    Validation(RaycastValidationError),
    /// Generic errors with context
    Generic {
        message: String,
        context: Option<String>,
        source: Option<String>,
    },
}

impl fmt::Display for RaycastDiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaycastDiscoveryError::Filesystem {
                path,
                operation,
                source,
            } => {
                write!(
                    f,
                    "Filesystem error during {} on '{}': {}",
                    operation, path, source
                )
            },
            RaycastDiscoveryError::Parsing {
                file_path,
                line,
                column,
                message,
            } => match (line, column) {
                (Some(l), Some(c)) => write!(
                    f,
                    "Parse error in '{}' at {}:{}: {}",
                    file_path, l, c, message
                ),
                (Some(l), None) => write!(
                    f,
                    "Parse error in '{}' at line {}: {}",
                    file_path, l, message
                ),
                _ => write!(f, "Parse error in '{}': {}", file_path, message),
            },
            RaycastDiscoveryError::InvalidStructure {
                path,
                field,
                expected,
                found,
            } => {
                write!(
                    f,
                    "Invalid structure in '{}': field '{}' expected {}, found {}",
                    path, field, expected, found
                )
            },
            RaycastDiscoveryError::MissingFiles {
                extension_path,
                missing_files,
            } => {
                write!(
                    f,
                    "Missing files in extension '{}': {}",
                    extension_path,
                    missing_files.join(", ")
                )
            },
            RaycastDiscoveryError::PermissionDenied { path, operation } => {
                write!(f, "Permission denied for {} on '{}'", operation, path)
            },
            RaycastDiscoveryError::SecurityViolation {
                path,
                violation,
                message,
            } => {
                write!(
                    f,
                    "Security violation in '{}': {} - {}",
                    path, violation, message
                )
            },
        }
    }
}

impl fmt::Display for RaycastExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaycastExecutionError::Runtime {
                command,
                exit_code,
                stderr,
                duration_ms,
                ..
            } => match exit_code {
                Some(code) => write!(
                    f,
                    "Command '{}' failed with exit code {} after {}ms: {}",
                    command, code, duration_ms, stderr
                ),
                None => write!(
                    f,
                    "Command '{}' failed after {}ms: {}",
                    command, duration_ms, stderr
                ),
            },
            RaycastExecutionError::CommandNotFound {
                command,
                available_commands,
            } => {
                write!(
                    f,
                    "Command '{}' not found. Available: {}",
                    command,
                    available_commands.join(", ")
                )
            },
            RaycastExecutionError::Timeout {
                command,
                timeout_ms,
                partial_output,
            } => {
                write!(
                    f,
                    "Command '{}' timed out after {}ms. Partial output: {}",
                    command, timeout_ms, partial_output
                )
            },
            RaycastExecutionError::ResourceExhausted {
                resource_type,
                limit,
                requested,
            } => {
                write!(
                    f,
                    "Resource exhausted: {} limit {} exceeded, requested {}",
                    resource_type, limit, requested
                )
            },
            RaycastExecutionError::PluginCrash {
                plugin_id,
                signal,
                last_output,
            } => match signal {
                Some(sig) => write!(
                    f,
                    "Plugin '{}' crashed with signal {}: {}",
                    plugin_id, sig, last_output
                ),
                None => write!(f, "Plugin '{}' crashed: {}", plugin_id, last_output),
            },
            RaycastExecutionError::EnvironmentError {
                variable,
                expected,
                found,
            } => match found {
                Some(val) => write!(
                    f,
                    "Environment variable '{}' expected '{}', found '{}'",
                    variable, expected, val
                ),
                None => write!(
                    f,
                    "Environment variable '{}' expected '{}', not found",
                    variable, expected
                ),
            },
        }
    }
}
impl fmt::Display for RaycastValidationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaycastValidationError::Schema {
                field,
                expected_type,
                found_type,
                value,
            } => {
                write!(
                    f,
                    "Schema validation failed for field '{}': expected {}, found {} (value: {})",
                    field, expected_type, found_type, value
                )
            },
            RaycastValidationError::Security {
                violation_type,
                resource,
                attempted_action,
                reason,
            } => {
                write!(
                    f,
                    "Security violation: {} on resource '{}' attempted '{}': {}",
                    violation_type, resource, attempted_action, reason
                )
            },
            RaycastValidationError::MissingRequired { field, context } => {
                write!(
                    f,
                    "Required field '{}' missing in context '{}'",
                    field, context
                )
            },
            RaycastValidationError::InvalidFormat {
                field,
                expected_format,
                provided_value,
            } => {
                write!(
                    f,
                    "Invalid format for field '{}': expected {}, got '{}'",
                    field, expected_format, provided_value
                )
            },
            RaycastValidationError::ConstraintViolation {
                field,
                constraint,
                value,
            } => {
                write!(
                    f,
                    "Constraint violation for field '{}': {} (value: {})",
                    field, constraint, value
                )
            },
            RaycastValidationError::DependencyFailure {
                dependent_field,
                required_field,
                condition,
            } => {
                write!(
                    f,
                    "Dependency validation failed: field '{}' requires '{}' when {}",
                    dependent_field, required_field, condition
                )
            },
        }
    }
}

impl fmt::Display for RaycastError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RaycastError::Discovery(err) => write!(f, "Discovery error: {}", err),
            RaycastError::Execution(err) => write!(f, "Execution error: {}", err),
            RaycastError::Validation(err) => write!(f, "Validation error: {}", err),
            RaycastError::Generic {
                message,
                context,
                source,
            } => match (context, source) {
                (Some(ctx), Some(src)) => {
                    write!(f, "{} (context: {}, source: {})", message, ctx, src)
                },
                (Some(ctx), None) => write!(f, "{} (context: {})", message, ctx),
                (None, Some(src)) => write!(f, "{} (source: {})", message, src),
                (None, None) => write!(f, "{}", message),
            },
        }
    }
}

// Error trait implementations
impl Error for RaycastDiscoveryError {}
impl Error for RaycastExecutionError {}
impl Error for RaycastValidationError {}
impl Error for RaycastError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            RaycastError::Discovery(err) => Some(err),
            RaycastError::Execution(err) => Some(err),
            RaycastError::Validation(err) => Some(err),
            RaycastError::Generic { .. } => None,
        }
    }
} // Error conversion traits for proper error chaining
impl From<io::Error> for RaycastDiscoveryError {
    fn from(err: io::Error) -> Self {
        RaycastDiscoveryError::Filesystem {
            path: "unknown".to_string(),
            operation: "io".to_string(),
            source: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for RaycastDiscoveryError {
    fn from(err: serde_json::Error) -> Self {
        RaycastDiscoveryError::Parsing {
            file_path: "unknown".to_string(),
            line: Some(err.line() as u32),
            column: Some(err.column() as u32),
            message: err.to_string(),
        }
    }
}

impl From<RaycastDiscoveryError> for RaycastError {
    fn from(err: RaycastDiscoveryError) -> Self {
        RaycastError::Discovery(err)
    }
}

impl From<RaycastExecutionError> for RaycastError {
    fn from(err: RaycastExecutionError) -> Self {
        RaycastError::Execution(err)
    }
}

impl From<RaycastValidationError> for RaycastError {
    fn from(err: RaycastValidationError) -> Self {
        RaycastError::Validation(err)
    }
}

// Context preservation methods
impl RaycastDiscoveryError {
    /// Add filesystem context to the error
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        match &mut self {
            RaycastDiscoveryError::Filesystem { path: p, .. } => *p = path.into(),
            RaycastDiscoveryError::Parsing { file_path, .. } => *file_path = path.into(),
            RaycastDiscoveryError::InvalidStructure { path: p, .. } => *p = path.into(),
            RaycastDiscoveryError::MissingFiles { extension_path, .. } => {
                *extension_path = path.into()
            },
            RaycastDiscoveryError::PermissionDenied { path: p, .. } => *p = path.into(),
            RaycastDiscoveryError::SecurityViolation { path: p, .. } => *p = path.into(),
        }
        self
    }
}

impl RaycastExecutionError {
    /// Add command context to the error
    pub fn with_command(mut self, command: impl Into<String>) -> Self {
        match &mut self {
            RaycastExecutionError::Runtime { command: c, .. } => *c = command.into(),
            RaycastExecutionError::CommandNotFound { command: c, .. } => *c = command.into(),
            RaycastExecutionError::Timeout { command: c, .. } => *c = command.into(),
            RaycastExecutionError::PluginCrash { plugin_id, .. } => *plugin_id = command.into(),
            _ => {},
        }
        self
    }
}

impl RaycastValidationError {
    /// Add field context to the error
    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        match &mut self {
            RaycastValidationError::Schema { field: f, .. } => *f = field.into(),
            RaycastValidationError::MissingRequired { field: f, .. } => *f = field.into(),
            RaycastValidationError::InvalidFormat { field: f, .. } => *f = field.into(),
            RaycastValidationError::ConstraintViolation { field: f, .. } => *f = field.into(),
            RaycastValidationError::DependencyFailure {
                dependent_field, ..
            } => *dependent_field = field.into(),
            _ => {},
        }
        self
    }
}

impl RaycastError {
    /// Create a generic error with context
    pub fn generic(message: impl Into<String>) -> Self {
        RaycastError::Generic {
            message: message.into(),
            context: None,
            source: None,
        }
    }

    /// Add context to any error type
    pub fn with_context(mut self, context: impl Into<String>) -> Self {
        if let RaycastError::Generic { context: c, .. } = &mut self {
            *c = Some(context.into())
        }
        self
    }

    /// Add source information to any error type
    pub fn with_source(mut self, source: impl Into<String>) -> Self {
        if let RaycastError::Generic { source: s, .. } = &mut self {
            *s = Some(source.into())
        }
        self
    }
}
