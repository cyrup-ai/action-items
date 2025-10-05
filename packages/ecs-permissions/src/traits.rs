//! Permission handler traits and interfaces

use std::sync::mpsc::Sender;

use crate::types::{PermissionError, PermissionStatus, PermissionType};

/// Trait for platform-specific permission handling
pub trait PermissionHandler: Send + Sync {
    /// Check the current status of a permission without requesting it
    fn check_permission(&self, typ: PermissionType) -> Result<PermissionStatus, PermissionError>;

    /// Request a permission from the user, sending result via channel
    fn request_permission(
        &self,
        typ: PermissionType,
        tx: Sender<Result<PermissionStatus, PermissionError>>,
    );
}
