//! Core error types for the Action Items Core library
//!
//! Provides comprehensive error handling for all core functionality including
//! configuration management, plugin operations, discovery, and service integration.

use std::error::Error as StdError;
use std::fmt;

/// Main error type for Action Items Core operations
#[derive(Debug, Clone)]
pub enum Error {
    /// Configuration-related errors
    Config(ConfigError),
    /// Plugin-related errors  
    Plugin(PluginError),
    /// Discovery-related errors
    Discovery(DiscoveryError),
    /// Search-related errors
    Search(SearchError),
    /// Service bridge communication errors
    ServiceBridge(ServiceBridgeError),
    /// File system operation errors
    FileSystem(FileSystemError),
    /// Generic I/O errors
    Io(String),
    /// Serialization/deserialization errors
    Serialization(SerializationError),
    /// Runtime errors
    Runtime(RuntimeError),

    // Additional error variants needed by the codebase
    /// Configuration-related errors (direct variant for backward compatibility)
    ConfigurationError(String),
    /// System-level operation errors
    SystemError(String),
    /// I/O operation errors (direct variant)
    IoError(String),
    /// Code execution errors
    ExecutionError(String),
    /// Extism plugin runtime errors
    Extism(String),
    /// Plugin loading errors
    PluginLoadError(String),
    /// Runtime operation errors (direct variant)
    RuntimeError(String),
    /// Serialization errors (direct variant)
    SerializationError(String),
    /// Plugin not found errors
    PluginNotFound(String),
    /// Plugin-related errors (direct variant for backward compatibility)
    PluginError(String),
}

/// Configuration management errors
#[derive(Debug, Clone)]
pub enum ConfigError {
    /// Configuration file not found
    NotFound(String),
    /// Invalid configuration format
    InvalidFormat(String),
    /// Configuration validation failed
    ValidationFailed(String),
    /// Permission denied accessing config
    PermissionDenied(String),
    /// Configuration backup/restore failed
    BackupFailed(String),
}

/// Plugin operation errors
#[derive(Debug, Clone)]
pub enum PluginError {
    /// Plugin not found
    NotFound(String),
    /// Plugin loading failed
    LoadFailed(String),
    /// Plugin initialization failed
    InitializationFailed(String),
    /// Plugin execution failed
    ExecutionFailed(String),
    /// Plugin manifest invalid
    InvalidManifest(String),
    /// Plugin dependencies not met
    DependencyError(String),
    /// Plugin communication failed
    CommunicationError(String),
}

/// Plugin discovery errors
#[derive(Debug, Clone)]
pub enum DiscoveryError {
    /// Discovery scan failed
    ScanFailed(String),
    /// Plugin build failed
    BuildFailed(String),
    /// Wrapper creation failed
    WrapperCreationFailed(String),
    /// Discovery timeout
    Timeout(String),
    /// Invalid plugin format
    InvalidFormat(String),
}

/// Search operation errors
#[derive(Debug, Clone)]
pub enum SearchError {
    /// Search query failed
    QueryFailed(String),
    /// Search index error
    IndexError(String),
    /// Search result processing failed
    ResultProcessingFailed(String),
    /// Search timeout
    Timeout(String),
}

/// Service bridge communication errors
#[derive(Debug, Clone)]
pub enum ServiceBridgeError {
    /// Service not available
    ServiceUnavailable(String),
    /// Message routing failed
    RoutingFailed(String),
    /// Communication timeout
    Timeout(String),
    /// Invalid message format
    InvalidMessage(String),
}

/// File system operation errors
#[derive(Debug, Clone)]
pub enum FileSystemError {
    /// File not found
    FileNotFound(String),
    /// Permission denied
    PermissionDenied(String),
    /// Disk full
    DiskFull(String),
    /// Invalid path
    InvalidPath(String),
}

/// Serialization/deserialization errors
#[derive(Debug, Clone)]
pub enum SerializationError {
    /// JSON serialization failed
    JsonError(String),
    /// TOML serialization failed
    TomlError(String),
    /// Binary serialization failed
    BinaryError(String),
    /// Invalid data format
    InvalidFormat(String),
}

/// Runtime operation errors
#[derive(Debug, Clone)]
pub enum RuntimeError {
    /// Runtime initialization failed
    InitializationFailed(String),
    /// Runtime operation failed
    OperationFailed(String),
    /// Runtime shutdown failed
    ShutdownFailed(String),
    /// Resource allocation failed
    ResourceAllocationFailed(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Config(e) => write!(f, "Configuration error: {}", e),
            Error::Plugin(e) => write!(f, "Plugin error: {}", e),
            Error::Discovery(e) => write!(f, "Discovery error: {}", e),
            Error::Search(e) => write!(f, "Search error: {}", e),
            Error::ServiceBridge(e) => write!(f, "Service bridge error: {}", e),
            Error::FileSystem(e) => write!(f, "File system error: {}", e),
            Error::Io(e) => write!(f, "I/O error: {}", e),
            Error::Serialization(e) => write!(f, "Serialization error: {}", e),
            Error::Runtime(e) => write!(f, "Runtime error: {}", e),

            // Additional error variants needed by the codebase
            Error::ConfigurationError(msg) => write!(f, "Configuration error: {}", msg),
            Error::SystemError(msg) => write!(f, "System error: {}", msg),
            Error::IoError(msg) => write!(f, "I/O error: {}", msg),
            Error::ExecutionError(msg) => write!(f, "Execution error: {}", msg),
            Error::Extism(msg) => write!(f, "Extism plugin error: {}", msg),
            Error::PluginLoadError(msg) => write!(f, "Plugin load error: {}", msg),
            Error::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            Error::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            Error::PluginNotFound(msg) => write!(f, "Plugin not found: {}", msg),
            Error::PluginError(msg) => write!(f, "Plugin error: {}", msg),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::NotFound(path) => write!(f, "Configuration file not found: {}", path),
            ConfigError::InvalidFormat(msg) => write!(f, "Invalid configuration format: {}", msg),
            ConfigError::ValidationFailed(msg) => {
                write!(f, "Configuration validation failed: {}", msg)
            },
            ConfigError::PermissionDenied(path) => {
                write!(f, "Permission denied accessing config: {}", path)
            },
            ConfigError::BackupFailed(msg) => write!(f, "Configuration backup failed: {}", msg),
        }
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PluginError::NotFound(name) => write!(f, "Plugin not found: {}", name),
            PluginError::LoadFailed(name) => write!(f, "Plugin loading failed: {}", name),
            PluginError::InitializationFailed(name) => {
                write!(f, "Plugin initialization failed: {}", name)
            },
            PluginError::ExecutionFailed(name) => write!(f, "Plugin execution failed: {}", name),
            PluginError::InvalidManifest(name) => write!(f, "Invalid plugin manifest: {}", name),
            PluginError::DependencyError(msg) => write!(f, "Plugin dependency error: {}", msg),
            PluginError::CommunicationError(msg) => {
                write!(f, "Plugin communication error: {}", msg)
            },
        }
    }
}

impl fmt::Display for DiscoveryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DiscoveryError::ScanFailed(msg) => write!(f, "Discovery scan failed: {}", msg),
            DiscoveryError::BuildFailed(msg) => write!(f, "Plugin build failed: {}", msg),
            DiscoveryError::WrapperCreationFailed(msg) => {
                write!(f, "Wrapper creation failed: {}", msg)
            },
            DiscoveryError::Timeout(msg) => write!(f, "Discovery timeout: {}", msg),
            DiscoveryError::InvalidFormat(msg) => write!(f, "Invalid plugin format: {}", msg),
        }
    }
}

impl fmt::Display for SearchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SearchError::QueryFailed(msg) => write!(f, "Search query failed: {}", msg),
            SearchError::IndexError(msg) => write!(f, "Search index error: {}", msg),
            SearchError::ResultProcessingFailed(msg) => {
                write!(f, "Search result processing failed: {}", msg)
            },
            SearchError::Timeout(msg) => write!(f, "Search timeout: {}", msg),
        }
    }
}

impl fmt::Display for ServiceBridgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServiceBridgeError::ServiceUnavailable(name) => {
                write!(f, "Service unavailable: {}", name)
            },
            ServiceBridgeError::RoutingFailed(msg) => write!(f, "Message routing failed: {}", msg),
            ServiceBridgeError::Timeout(msg) => write!(f, "Communication timeout: {}", msg),
            ServiceBridgeError::InvalidMessage(msg) => write!(f, "Invalid message format: {}", msg),
        }
    }
}

impl fmt::Display for FileSystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FileSystemError::FileNotFound(path) => write!(f, "File not found: {}", path),
            FileSystemError::PermissionDenied(path) => write!(f, "Permission denied: {}", path),
            FileSystemError::DiskFull(path) => write!(f, "Disk full: {}", path),
            FileSystemError::InvalidPath(path) => write!(f, "Invalid path: {}", path),
        }
    }
}

impl fmt::Display for SerializationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SerializationError::JsonError(msg) => write!(f, "JSON error: {}", msg),
            SerializationError::TomlError(msg) => write!(f, "TOML error: {}", msg),
            SerializationError::BinaryError(msg) => {
                write!(f, "Binary serialization error: {}", msg)
            },
            SerializationError::InvalidFormat(msg) => write!(f, "Invalid data format: {}", msg),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::InitializationFailed(msg) => {
                write!(f, "Runtime initialization failed: {}", msg)
            },
            RuntimeError::OperationFailed(msg) => write!(f, "Runtime operation failed: {}", msg),
            RuntimeError::ShutdownFailed(msg) => write!(f, "Runtime shutdown failed: {}", msg),
            RuntimeError::ResourceAllocationFailed(msg) => {
                write!(f, "Resource allocation failed: {}", msg)
            },
        }
    }
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

impl StdError for ConfigError {}
impl StdError for PluginError {}
impl StdError for DiscoveryError {}
impl StdError for SearchError {}
impl StdError for ServiceBridgeError {}
impl StdError for FileSystemError {}
impl StdError for SerializationError {}
impl StdError for RuntimeError {}

// Conversion from standard I/O errors
impl From<std::io::Error> for Error {
    fn from(error: std::io::Error) -> Self {
        Error::Io(error.to_string())
    }
}

// Conversion from serde_json errors
impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::Serialization(SerializationError::JsonError(error.to_string()))
    }
}

// Conversion from ServiceBridgeError
impl From<crate::service_bridge::bridge::core::health::ServiceBridgeError> for Error {
    fn from(error: crate::service_bridge::bridge::core::health::ServiceBridgeError) -> Self {
        Error::ServiceBridge(ServiceBridgeError::RoutingFailed(error.to_string()))
    }
}

// Convenience result type
pub type Result<T> = std::result::Result<T, Error>;
