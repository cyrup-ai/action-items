//! Error types for Bluetooth operations

use thiserror::Error;

/// Result type for Bluetooth operations
pub type BluetoothResult<T> = Result<T, BluetoothError>;

/// Bluetooth operation errors
#[derive(Error, Debug, Clone)]
pub enum BluetoothError {
    #[error("Bluetooth adapter not available")]
    AdapterNotAvailable,

    #[error("Bluetooth is powered off")]
    PoweredOff,

    #[error("Bluetooth access denied - check permissions")]
    PermissionDenied,

    #[error("Device not found: {id}")]
    DeviceNotFoundWithId { id: String },

    #[error("Device not found")]
    DeviceNotFound,

    #[error("Connection failed: {reason}")]
    ConnectionFailed { reason: String },

    #[error("Operation timeout")]
    Timeout,

    #[error("Invalid device address: {address}")]
    InvalidAddress { address: String },

    #[error("Invalid device ID")]
    InvalidDeviceId,

    #[error("Scan operation failed: {reason}")]
    ScanFailed { reason: String },

    #[error("Platform error: {message}")]
    PlatformError { message: String },

    #[error("Operation not supported on this platform")]
    NotSupported,

    #[error("Bluetooth system not initialized")]
    NotInitialized,

    #[error("Bluetooth adapter not ready")]
    AdapterNotReady,

    #[error("Internal error")]
    InternalError,

    #[error("Internal error: {message}")]
    Internal { message: String },
}

impl BluetoothError {
    /// Create a platform-specific error
    pub fn platform(message: impl Into<String>) -> Self {
        Self::PlatformError {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }
}
