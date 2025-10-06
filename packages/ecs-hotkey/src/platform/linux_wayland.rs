//! Wayland global hotkey backend coordinator
//!
//! Routes to appropriate Wayland backend based on compositor detection.

use crate::{HotkeyBinding, HotkeyDefinition};
use tracing::{error, info};

#[path = "linux_wayland_kde.rs"]
mod kde;
#[path = "linux_wayland_portal.rs"]
mod portal;

use kde::KdeGlobalAccelBackend;
use portal::XdgPortalBackend;

/// Wayland hotkey backend abstraction
#[async_trait::async_trait]
pub trait WaylandBackend: Send + Sync {
    /// Initialize the backend
    async fn init(&mut self) -> Result<(), WaylandError>;

    /// Register a global hotkey
    async fn register(&mut self, binding: &HotkeyBinding) -> Result<(), WaylandError>;

    /// Unregister a global hotkey
    async fn unregister(&mut self, action_id: &str) -> Result<(), WaylandError>;

    /// Poll for hotkey events (returns action IDs)
    async fn poll_events(&mut self) -> Result<Vec<String>, WaylandError>;

    /// Check if backend is available
    async fn is_available() -> bool
    where
        Self: Sized;
}

/// Wayland backend manager
pub struct WaylandHotkeyManager {
    backend: Box<dyn WaylandBackend>,
}

impl WaylandHotkeyManager {
    /// Create new Wayland hotkey manager with auto-detected backend
    pub async fn new() -> Result<Self, WaylandError> {
        let compositor = super::detect_compositor();

        info!("Initializing Wayland hotkey backend for {:?}", compositor);

        // Try backends in order of preference
        let backend: Box<dyn WaylandBackend> = match compositor {
            super::LinuxCompositor::Kde => {
                if KdeGlobalAccelBackend::is_available().await {
                    info!("Using KDE kglobalaccel backend");
                    let mut backend = Box::new(KdeGlobalAccelBackend::new().await?);
                    backend.init().await?;
                    backend
                } else {
                    return Err(WaylandError::BackendUnavailable(
                        "KDE kglobalaccel service not available".to_string(),
                    ));
                }
            }
            super::LinuxCompositor::Hyprland => {
                if XdgPortalBackend::is_available().await {
                    info!("Using XDG Desktop Portal backend");
                    let mut backend = Box::new(XdgPortalBackend::new().await?);
                    backend.init().await?;
                    backend
                } else {
                    return Err(WaylandError::BackendUnavailable(
                        "XDG Desktop Portal not available".to_string(),
                    ));
                }
            }
            _ => {
                return Err(WaylandError::UnsupportedCompositor(format!(
                    "{:?}",
                    compositor
                )));
            }
        };

        Ok(Self { backend })
    }

    /// Register hotkey binding
    pub async fn register(&mut self, binding: &HotkeyBinding) -> Result<(), WaylandError> {
        self.backend.register(binding).await
    }

    /// Unregister hotkey binding
    pub async fn unregister(&mut self, action_id: &str) -> Result<(), WaylandError> {
        self.backend.unregister(action_id).await
    }

    /// Poll for triggered hotkeys
    pub async fn poll_events(&mut self) -> Result<Vec<String>, WaylandError> {
        self.backend.poll_events().await
    }
}

/// Wayland-specific errors
#[derive(Debug, thiserror::Error)]
pub enum WaylandError {
    #[error("Unsupported Wayland compositor: {0}")]
    UnsupportedCompositor(String),

    #[error("Backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("DBus error: {0}")]
    DBusError(#[from] zbus::Error),

    #[error("Hotkey already registered: {0}")]
    AlreadyRegistered(String),

    #[error("Hotkey not found: {0}")]
    NotFound(String),

    #[error("Invalid shortcut format: {0}")]
    InvalidShortcut(String),
}
