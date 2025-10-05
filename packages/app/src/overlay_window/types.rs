//! Error types and window attributes for overlay window configuration

/// Configuration result with detailed status information
#[derive(Debug, Clone)]
pub struct OverlayConfigurationResult {
    pub platform: String,
    pub success: bool,
    pub details: String,
}

impl OverlayConfigurationResult {
    pub fn success(platform: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            success: true,
            details: details.into(),
        }
    }

    pub fn failure(platform: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            success: false,
            details: details.into(),
        }
    }
}

impl std::fmt::Display for OverlayConfigurationResult {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} overlay configuration {}: {}",
            self.platform,
            if self.success { "successful" } else { "failed" },
            self.details
        )
    }
}

/// Cross-platform overlay configuration error
#[derive(Debug, thiserror::Error)]
pub enum OverlayConfigurationError {
    #[error("Platform-specific overlay error: {0}")]
    PlatformError(#[from] OverlayError),
    #[error("All platform configurations failed")]
    AllPlatformsFailed,
}

/// Errors that can occur during overlay window configuration
#[derive(Debug, thiserror::Error)]
pub enum OverlayError {
    #[error("Failed to access window handle")]
    HandleAccess,
    #[error("Platform mismatch - window handle doesn't match expected platform")]
    PlatformMismatch,
    #[cfg(all(feature = "wayland-client", target_os = "linux"))]
    #[error("Failed to connect to Wayland display: {0}")]
    WaylandConnection(String),
    #[cfg(all(feature = "wayland-protocols-wlr", target_os = "linux"))]
    #[error("Wayland protocol error: {protocol} - {details}")]
    WaylandProtocol { protocol: String, details: String },
    #[cfg(target_os = "macos")]
    #[error("macOS NSWindow setLevel failed")]
    MacOSSetLevelFailed,
    #[cfg(target_os = "windows")]
    #[error("Windows GetWindowLongPtrW failed")]
    WindowsGetStyleFailed,
    #[cfg(target_os = "windows")]
    #[error("Windows SetWindowLongPtrW failed")]
    WindowsSetStyleFailed,
    #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
    #[error("X11 XChangeWindowAttributes failed: {0}")]
    X11ConfigureFailed(String),
    #[cfg(all(feature = "xcb", unix, not(target_os = "macos")))]
    #[error("XCB change_window_attributes failed: {0}")]
    XcbConfigureFailed(String),
}

impl OverlayError {
    /// Create a Wayland connection error with context
    #[cfg(all(feature = "wayland-client", target_os = "linux"))]
    #[inline]
    pub fn wayland_connection(details: impl Into<String>) -> Self {
        Self::WaylandConnection(details.into())
    }

    /// Create a Wayland protocol error with context
    #[cfg(all(feature = "wayland-protocols-wlr", target_os = "linux"))]
    #[inline]
    pub fn wayland_protocol(protocol: impl Into<String>, details: impl Into<String>) -> Self {
        Self::WaylandProtocol {
            protocol: protocol.into(),
            details: details.into(),
        }
    }

    /// Create an X11 configuration error with context
    #[cfg(all(feature = "x11", unix, not(target_os = "macos")))]
    #[inline]
    pub fn x11_configure_failed(details: impl Into<String>) -> Self {
        Self::X11ConfigureFailed(details.into())
    }

    /// Create an XCB configuration error with context
    #[cfg(all(feature = "xcb", unix, not(target_os = "macos")))]
    #[inline]
    pub fn xcb_configure_failed(details: impl Into<String>) -> Self {
        Self::XcbConfigureFailed(details.into())
    }

    /// Check if error is platform-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_platform_error(&self) -> bool {
        matches!(self, Self::PlatformMismatch)
    }

    /// Check if error is handle-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_handle_error(&self) -> bool {
        matches!(self, Self::HandleAccess)
    }

    /// Check if error is Wayland-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_wayland_error(&self) -> bool {
        #[cfg(all(feature = "wayland-client", target_os = "linux"))]
        {
            matches!(
                self,
                Self::WaylandConnection(_) | Self::WaylandProtocol { .. }
            )
        }
        #[cfg(not(all(feature = "wayland-client", target_os = "linux")))]
        {
            false
        }
    }

    /// Check if error is Windows-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_windows_error(&self) -> bool {
        #[cfg(target_os = "windows")]
        {
            matches!(
                self,
                Self::WindowsGetStyleFailed | Self::WindowsSetStyleFailed
            )
        }
        #[cfg(not(target_os = "windows"))]
        {
            false
        }
    }

    /// Check if error is X11/XCB-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_x11_error(&self) -> bool {
        #[cfg(all(unix, not(target_os = "macos")))]
        {
            matches!(
                self,
                Self::X11ConfigureFailed(_) | Self::XcbConfigureFailed(_)
            )
        }
        #[cfg(not(all(unix, not(target_os = "macos"))))]
        {
            false
        }
    }

    /// Check if error is macOS-related
    #[allow(dead_code)] // Used in diagnostics - false positive due to conditional compilation
    #[inline]
    pub fn is_macos_error(&self) -> bool {
        matches!(self, Self::MacOSSetLevelFailed)
    }
}
