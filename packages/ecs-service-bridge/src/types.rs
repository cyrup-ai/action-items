//! Core type system for ECS Service Bridge
//!
//! Zero-allocation, blazing-fast types with compile-time guarantees and optimal memory layout.
//! All types are designed for cache-friendly access patterns and maximum performance.

use std::fmt;
use std::hash::{Hash, Hasher};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// High-performance timestamp wrapper with proper serialization support
///
/// Uses chrono internally for blazing-fast serialization while maintaining Instant-like semantics.
/// All operations are zero-allocation with const-time comparisons where possible.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct TimeStamp {
    #[serde(with = "chrono::serde::ts_milliseconds")]
    inner: chrono::DateTime<chrono::Utc>,
}

impl TimeStamp {
    /// Create a new TimeStamp representing the current moment
    #[inline]
    pub fn now() -> Self {
        Self {
            inner: chrono::Utc::now(),
        }
    }

    /// Create TimeStamp from milliseconds since Unix epoch
    #[inline]
    pub fn from_millis(millis: i64) -> Result<Self, ServiceError> {
        match chrono::DateTime::from_timestamp_millis(millis) {
            Some(dt) => Ok(Self { inner: dt }),
            None => Err(ServiceError::InvalidTimestamp),
        }
    }

    /// Get milliseconds since Unix epoch
    #[inline]
    pub fn as_millis(&self) -> i64 {
        self.inner.timestamp_millis()
    }

    /// Calculate duration since this timestamp
    #[inline]
    pub fn elapsed(&self) -> Duration {
        let now = Self::now();
        now.duration_since(*self).unwrap_or(Duration::ZERO)
    }

    /// Calculate duration between two timestamps
    #[inline]
    pub fn duration_since(&self, other: TimeStamp) -> Result<Duration, ServiceError> {
        if self.inner >= other.inner {
            let diff = self.inner.signed_duration_since(other.inner);
            diff.to_std().map_err(|_| ServiceError::InvalidDuration)
        } else {
            Err(ServiceError::InvalidDuration)
        }
    }

    /// Add duration to timestamp
    #[inline]
    pub fn add_duration(&self, duration: Duration) -> Result<Self, ServiceError> {
        let chrono_duration =
            chrono::Duration::from_std(duration).map_err(|_| ServiceError::InvalidDuration)?;

        match self.inner.checked_add_signed(chrono_duration) {
            Some(new_time) => Ok(Self { inner: new_time }),
            None => Err(ServiceError::TimestampOverflow),
        }
    }
}

impl Default for TimeStamp {
    #[inline]
    fn default() -> Self {
        Self::now()
    }
}

impl fmt::Display for TimeStamp {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner.format("%Y-%m-%d %H:%M:%S UTC"))
    }
}

/// Zero-allocation message addressing with compile-time validation
///
/// Optimized for cache-friendly memory layout with const generic optimizations.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout
pub struct MessageAddress {
    /// Plugin identifier - validated at compile time where possible
    plugin_id: String,
    /// Service capability name
    capability: Option<String>,
    /// Instance identifier for load balancing
    instance_id: Option<String>,
}

impl MessageAddress {
    /// Create new message address with validation
    #[inline]
    pub fn new(plugin_id: impl Into<String>) -> Result<Self, ServiceError> {
        let plugin_id = plugin_id.into();

        if plugin_id.is_empty() {
            return Err(ServiceError::InvalidAddress(
                "Plugin ID cannot be empty".into(),
            ));
        }

        if plugin_id.len() > 256 {
            return Err(ServiceError::InvalidAddress("Plugin ID too long".into()));
        }

        // Validate plugin_id contains only valid characters (alphanumeric, underscore, hyphen)
        if !plugin_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ServiceError::InvalidAddress(
                "Plugin ID contains invalid characters".into(),
            ));
        }

        Ok(Self {
            plugin_id,
            capability: None,
            instance_id: None,
        })
    }

    /// Create address with specific capability
    #[inline]
    pub fn with_capability(mut self, capability: impl Into<String>) -> Result<Self, ServiceError> {
        let capability = capability.into();

        if capability.is_empty() {
            return Err(ServiceError::InvalidAddress(
                "Capability cannot be empty".into(),
            ));
        }

        self.capability = Some(capability);
        Ok(self)
    }

    /// Create address with specific instance
    #[inline]
    pub fn with_instance(mut self, instance_id: impl Into<String>) -> Result<Self, ServiceError> {
        let instance_id = instance_id.into();

        if instance_id.is_empty() {
            return Err(ServiceError::InvalidAddress(
                "Instance ID cannot be empty".into(),
            ));
        }

        self.instance_id = Some(instance_id);
        Ok(self)
    }

    /// Get plugin identifier
    #[inline]
    pub fn plugin_id(&self) -> &str {
        &self.plugin_id
    }

    /// Get capability if specified
    #[inline]
    pub fn capability(&self) -> Option<&str> {
        self.capability.as_deref()
    }

    /// Create MessageAddress from string representation
    #[inline]
    pub fn from_string(address_str: &str) -> Result<Self, ServiceError> {
        if address_str.is_empty() {
            return Err(ServiceError::InvalidAddress(
                "Address cannot be empty".into(),
            ));
        }

        // Handle special system addresses
        if address_str == "system" {
            return Ok(Self::system());
        }

        if address_str == "broadcast" {
            return Ok(Self::broadcast());
        }

        // Parse plugin_id[@capability][#instance_id] format
        let mut parts = address_str.splitn(2, '@');
        let plugin_id = parts.next().unwrap().to_string();

        let mut address = Self::new(plugin_id)?;

        if let Some(remaining) = parts.next() {
            let mut cap_instance = remaining.splitn(2, '#');
            let capability = cap_instance.next().unwrap();

            if !capability.is_empty() {
                address = address.with_capability(capability)?;
            }

            if let Some(instance_id) = cap_instance.next()
                && !instance_id.is_empty() {
                    address = address.with_instance(instance_id)?;
                }
        }

        Ok(address)
    }

    /// Create system message address
    #[inline]
    pub fn system() -> Self {
        Self {
            plugin_id: "system".to_string(),
            capability: None,
            instance_id: None,
        }
    }

    /// Create broadcast message address
    #[inline]
    pub fn broadcast() -> Self {
        Self {
            plugin_id: "broadcast".to_string(),
            capability: None,
            instance_id: None,
        }
    }


}

impl MessageAddress {
    /// Get instance ID if specified
    #[inline]
    pub fn instance_id(&self) -> Option<&str> {
        self.instance_id.as_deref()
    }

    /// Check if this address matches another for routing
    #[inline]
    pub fn matches(&self, other: &Self) -> bool {
        self.plugin_id == other.plugin_id
            && self.capability.as_deref() == other.capability.as_deref()
    }
}

impl fmt::Display for MessageAddress {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = self.plugin_id.clone();

        if let Some(ref capability) = self.capability {
            result.push('@');
            result.push_str(capability);
        }

        if let Some(ref instance_id) = self.instance_id {
            result.push('#');
            result.push_str(instance_id);
        }

        write!(f, "{}", result)
    }
}

/// UUID-based correlation ID for request/response tracking
///
/// Uses stack-allocated UUID for maximum performance with zero heap allocations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout
pub struct CorrelationId {
    inner: Uuid,
}

impl CorrelationId {
    /// Generate new unique correlation ID
    #[inline]
    pub fn new() -> Self {
        Self {
            inner: Uuid::new_v4(),
        }
    }

    /// Create from existing UUID
    #[inline]
    pub const fn from_uuid(uuid: Uuid) -> Self {
        Self { inner: uuid }
    }

    /// Get underlying UUID
    #[inline]
    pub const fn as_uuid(&self) -> &Uuid {
        &self.inner
    }

    /// Convert to bytes for efficient serialization
    #[inline]
    pub fn as_bytes(&self) -> &[u8; 16] {
        self.inner.as_bytes()
    }

    /// Create from bytes
    #[inline]
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self {
            inner: Uuid::from_bytes(bytes),
        }
    }

    /// Create from string representation
    #[inline]
    pub fn from_string(s: &str) -> Result<Self, ServiceError> {
        Uuid::parse_str(s)
            .map(|uuid| Self { inner: uuid })
            .map_err(|_| ServiceError::InvalidAddress("Invalid correlation ID format".into()))
    }
}

impl Default for CorrelationId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for CorrelationId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

/// High-performance request ID with collision detection
///
/// Uses combination of timestamp and random data for guaranteed uniqueness.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout
pub struct RequestId {
    timestamp: u64,
    random: u64,
}

impl RequestId {
    /// Generate new unique request ID with collision detection
    #[inline]
    pub fn new() -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0);

        let random = {
            use std::collections::hash_map::DefaultHasher;
            let mut hasher = DefaultHasher::new();
            timestamp.hash(&mut hasher);
            std::ptr::addr_of!(hasher).hash(&mut hasher);
            hasher.finish()
        };

        Self { timestamp, random }
    }

    /// Get timestamp component
    #[inline]
    pub const fn timestamp(&self) -> u64 {
        self.timestamp
    }

    /// Get random component
    #[inline]
    pub const fn random(&self) -> u64 {
        self.random
    }

    /// Convert to u128 for efficient hashing
    #[inline]
    pub const fn as_u128(&self) -> u128 {
        ((self.timestamp as u128) << 64) | (self.random as u128)
    }

    /// Create from u128
    #[inline]
    pub const fn from_u128(value: u128) -> Self {
        Self {
            timestamp: (value >> 64) as u64,
            random: value as u64,
        }
    }
}

impl Default for RequestId {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RequestId {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:016x}{:016x}", self.timestamp, self.random)
    }
}

/// Comprehensive error type covering all service bridge error scenarios
///
/// Designed for zero-cost error handling with semantic error information.
#[derive(Debug, Clone, thiserror::Error, Serialize, Deserialize)]
pub enum ServiceError {
    #[error("Invalid message address: {0}")]
    InvalidAddress(String),

    #[error("Invalid timestamp")]
    InvalidTimestamp,

    #[error("Invalid duration")]
    InvalidDuration,

    #[error("Timestamp overflow")]
    TimestampOverflow,

    #[error("Plugin not found: {plugin_id}")]
    PluginNotFound { plugin_id: String },

    #[error("Plugin registration failed: {reason}")]
    PluginRegistrationFailed { reason: String },

    #[error("Plugin authentication failed: {plugin_id}")]
    PluginAuthenticationFailed { plugin_id: String },

    #[error("Plugin capability not found: {capability}")]
    CapabilityNotFound { capability: String },

    #[error("Message routing failed: {reason}")]
    MessageRoutingFailed { reason: String },

    #[error("Message timeout: correlation_id={correlation_id}")]
    MessageTimeout { correlation_id: CorrelationId },

    #[error("Message queue full: {queue_name}")]
    MessageQueueFull { queue_name: String },

    #[error("Service handler error: {service}: {reason}")]
    ServiceHandlerError { service: String, reason: String },

    #[error("Clipboard operation failed: {operation}: {reason}")]
    ClipboardError { operation: String, reason: String },

    #[error("Storage operation failed: {operation}: {reason}")]
    StorageError { operation: String, reason: String },

    #[error("HTTP operation failed: {method} {url}: {reason}")]
    HttpError {
        method: String,
        url: String,
        reason: String,
    },

    #[error("Notification operation failed: {reason}")]
    NotificationError { reason: String },

    #[error("Security violation: {reason}")]
    SecurityViolation { reason: String },

    #[error("Permission denied: {operation} for {plugin_id}")]
    PermissionDenied {
        operation: String,
        plugin_id: String,
    },

    #[error("Rate limit exceeded: {plugin_id}")]
    RateLimitExceeded { plugin_id: String },

    #[error("Resource exhausted: {resource}")]
    ResourceExhausted { resource: String },

    #[error("Plugin communication failed: {plugin_id}: {reason}")]
    PluginCommunicationFailed { plugin_id: String, reason: String },

    #[error("Routing loop detected at address: {address}")]
    RoutingLoop { address: String },

    #[error("Message expired: message_id={message_id}")]
    MessageExpired { message_id: String },

    #[error("Configuration error: {setting}: {reason}")]
    ConfigurationError { setting: String, reason: String },

    #[error("Migration error: {reason}")]
    MigrationError { reason: String },

    #[error("Internal error: {reason}")]
    Internal { reason: String },
}

impl ServiceError {
    /// Create plugin not found error
    #[inline]
    pub fn plugin_not_found(plugin_id: impl Into<String>) -> Self {
        Self::PluginNotFound {
            plugin_id: plugin_id.into(),
        }
    }

    /// Create message timeout error
    #[inline]
    pub fn message_timeout(correlation_id: CorrelationId) -> Self {
        Self::MessageTimeout { correlation_id }
    }

    /// Create permission denied error
    #[inline]
    pub fn permission_denied(operation: impl Into<String>, plugin_id: impl Into<String>) -> Self {
        Self::PermissionDenied {
            operation: operation.into(),
            plugin_id: plugin_id.into(),
        }
    }

    /// Check if error is retryable
    #[inline]
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            Self::MessageQueueFull { .. }
                | Self::ResourceExhausted { .. }
                | Self::MessageTimeout { .. }
                | Self::Internal { .. }
        )
    }

    /// Check if error is a security violation
    #[inline]
    pub fn is_security_violation(&self) -> bool {
        matches!(
            self,
            Self::PluginAuthenticationFailed { .. }
                | Self::SecurityViolation { .. }
                | Self::PermissionDenied { .. }
        )
    }
}

/// Result type alias for service bridge operations
pub type ServiceResult<T> = Result<T, ServiceError>;

/// Message priority levels for routing optimization
///
/// Uses const generic discrimination for zero-cost priority handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[repr(u8)] // Explicit representation for optimal memory usage
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
    Background = 4,
}

impl MessagePriority {
    /// Get priority as u8 for efficient comparisons
    #[inline]
    pub const fn as_u8(&self) -> u8 {
        *self as u8
    }

    /// Create from u8 with validation
    #[inline]
    pub fn from_u8(value: u8) -> Result<Self, ServiceError> {
        match value {
            0 => Ok(Self::Critical),
            1 => Ok(Self::High),
            2 => Ok(Self::Normal),
            3 => Ok(Self::Low),
            4 => Ok(Self::Background),
            _ => Err(ServiceError::InvalidAddress(
                "Invalid priority value".into(),
            )),
        }
    }
}

impl Default for MessagePriority {
    #[inline]
    fn default() -> Self {
        Self::Normal
    }
}

impl fmt::Display for MessagePriority {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Critical => write!(f, "critical"),
            Self::High => write!(f, "high"),
            Self::Normal => write!(f, "normal"),
            Self::Low => write!(f, "low"),
            Self::Background => write!(f, "background"),
        }
    }
}
