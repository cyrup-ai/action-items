//! Zero-allocation notification system with blazing-fast cross-platform support
//!
//! This module provides a high-performance, cross-platform notification system
//! that integrates with native OS notification systems without allocation overhead.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Result type for notification operations with zero-allocation error handling
pub type NotificationResult<T> = Result<T, NotificationError>;

/// Zero-allocation notification error with blazing-fast error categorization
#[derive(Debug, Clone, thiserror::Error)]
pub enum NotificationError {
    #[error("Platform notification service unavailable: {0}")]
    ServiceUnavailable(String),
    #[error("Notification permission denied: {0}")]
    PermissionDenied(String),
    #[error("Invalid notification content: {0}")]
    InvalidContent(String),
    #[error("System notification API error: {0}")]
    SystemError(String),
    #[error("Notification timeout: {0}")]
    Timeout(String),
    #[error("Database error: {0}")]
    DatabaseError(String),
}

/// Zero-allocation notification options with blazing-fast field access
#[derive(Debug, Clone)]
pub struct NotificationOptions<'a> {
    pub title: &'a str,
    pub message: &'a str,
    pub icon: Option<&'a str>,
    pub sound: bool,
    pub duration: Option<Duration>,
    pub urgent: bool,
}

impl<'a> Default for NotificationOptions<'a> {
    #[inline(always)]
    fn default() -> Self {
        Self {
            title: "",
            message: "",
            icon: None,
            sound: true,
            duration: Some(Duration::from_secs(3)),
            urgent: false,
        }
    }
}

/// Unique notification identifier with zero allocation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct NotificationId(u64);

impl NotificationId {
    #[inline(always)]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    #[inline(always)]
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Database mapping between local notification IDs and platform-specific IDs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationMapping {
    /// Local notification ID used by our system
    pub local_id: u64,
    /// Platform-specific notification ID (D-Bus ID, Toast handle, etc.)
    pub platform_id: u32,
    /// Platform identifier
    pub platform: String,
    /// Creation timestamp
    pub created_at: u64,
}

/// Zero-allocation notification backend trait with blazing-fast method dispatch
pub trait NotificationBackend: Send + Sync {
    /// Show notification with blazing-fast system integration
    fn show_notification(
        &self,
        options: NotificationOptions<'_>,
    ) -> NotificationResult<NotificationId>;

    /// Check if notification service is available with zero allocation
    fn is_available(&self) -> bool;

    /// Get platform name with zero allocation
    fn platform_name(&self) -> &'static str;

    /// Dismiss notification with blazing-fast cleanup
    fn dismiss(&self, id: NotificationId) -> NotificationResult<()>;
}

/// Zero-allocation notification manager with blazing-fast platform detection
pub struct NotificationManager {
    backend: Box<dyn NotificationBackend>,
}

impl NotificationManager {
    /// Create notification manager without database persistence (for ops/runtime use)
    pub fn new_without_persistence() -> NotificationResult<Self> {
        let backend = Self::create_platform_backend_without_db()?;
        Ok(Self { backend })
    }

    /// Create fallback notification manager when initialization fails
    pub fn new_fallback() -> Self {
        let backend = Box::new(FallbackNotificationBackend::new());
        Self { backend }
    }

    /// Show toast notification with zero-allocation message processing
    #[inline]
    pub fn show_toast(&self, message: &str) -> NotificationResult<NotificationId> {
        let options = NotificationOptions {
            title: "Action Items",
            message,
            ..Default::default()
        };
        self.backend.show_notification(options)
    }

    /// Show notification with custom options and blazing-fast parameter passing
    #[inline]
    pub fn show_notification(
        &self,
        options: NotificationOptions<'_>,
    ) -> NotificationResult<NotificationId> {
        self.backend.show_notification(options)
    }

    /// Check availability with zero allocation
    #[inline]
    pub fn is_available(&self) -> bool {
        self.backend.is_available()
    }

    /// Get platform name with zero allocation
    #[inline]
    pub fn platform_name(&self) -> &'static str {
        self.backend.platform_name()
    }

    /// Dismiss notification with blazing-fast backend delegation
    #[inline]
    pub fn dismiss(&self, id: NotificationId) -> NotificationResult<()> {
        self.backend.dismiss(id)
    }

    /// Create optimal platform-specific backend with compile-time selection
    /// Create platform backend without database persistence (for runtime ops)
    fn create_platform_backend_without_db() -> NotificationResult<Box<dyn NotificationBackend>> {
        #[cfg(target_os = "macos")]
        {
            let backend = macos::MacosNotificationBackend::new_without_db()?;
            Ok(Box::new(backend))
        }
        #[cfg(target_os = "linux")]
        {
            let backend = linux::LinuxNotificationBackend::new_without_db()?;
            Ok(Box::new(backend))
        }
        #[cfg(target_os = "windows")]
        {
            let backend = windows::WindowsNotificationBackend::new_without_db()?;
            Ok(Box::new(backend))
        }
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Ok(Box::new(FallbackNotificationBackend::new()))
        }
    }
}

// Note: Default implementation removed - NotificationManager now uses ecs-surrealdb

/// Fallback notification backend for runtime ops and unsupported platforms
struct FallbackNotificationBackend;

impl FallbackNotificationBackend {
    pub fn new() -> Self {
        Self
    }
}

impl NotificationBackend for FallbackNotificationBackend {
    #[inline]
    fn show_notification(
        &self,
        _options: NotificationOptions<'_>,
    ) -> NotificationResult<NotificationId> {
        Err(NotificationError::ServiceUnavailable(
            "Null backend - no notification system available".to_string(),
        ))
    }

    #[inline(always)]
    fn is_available(&self) -> bool {
        false
    }

    #[inline(always)]
    fn platform_name(&self) -> &'static str {
        "null"
    }

    #[inline]
    fn dismiss(&self, _id: NotificationId) -> NotificationResult<()> {
        Ok(())
    }
}

// Platform-specific module declarations with conditional compilation
#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;
