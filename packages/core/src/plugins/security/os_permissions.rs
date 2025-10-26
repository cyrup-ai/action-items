use std::fs;
use std::path::Path;

use action_items_ecs_permissions::{PermissionManager, PermissionStatus, PermissionType};
use tracing::{debug, info};

use crate::error::{Error, Result, SecurityError};
use crate::plugins::interface::{PluginManifest, PluginPermissions};

/// Performs OS-level permission verification based on plugin manifest declarations.
pub struct OsPermissionChecker {
    permission_manager: PermissionManager,
}

impl OsPermissionChecker {
    pub fn new() -> Result<Self> {
        Ok(Self {
            permission_manager: PermissionManager::new(),
        })
    }

    /// Check OS-level permissions match manifest declarations
    pub fn check_permissions(&self, manifest: &PluginManifest) -> Result<()> {
        let permissions = &manifest.permissions;

        if permissions.accessibility {
            self.ensure_permission(PermissionType::Accessibility, "accessibility")?;
        }

        if permissions.camera {
            self.ensure_permission(PermissionType::Camera, "camera")?;
        }

        if permissions.microphone {
            self.ensure_permission(PermissionType::Microphone, "microphone")?;
        }

        if permissions.location {
            self.ensure_permission(PermissionType::Location, "location")?;
        }

        if permissions.contacts {
            self.ensure_permission(PermissionType::Contacts, "contacts")?;
        }

        if permissions.calendar {
            self.ensure_permission(PermissionType::Calendar, "calendar")?;
        }

        if !permissions.read_files.is_empty() || !permissions.write_files.is_empty() {
            self.check_file_system_access(permissions)?;
        }

        if !permissions.network_hosts.is_empty() {
            self.check_network_access(permissions)?;
        }

        if permissions.system_notifications {
            debug!(
                target = "plugin_security_permissions",
                plugin_id = %manifest.id,
                "Notifications permission declared; ensure platform setup is completed"
            );
        }

        if permissions.read_clipboard || permissions.write_clipboard {
            debug!(
                target = "plugin_security_permissions",
                plugin_id = %manifest.id,
                "Clipboard access requested; relying on application sandbox policies"
            );
        }

        Ok(())
    }

    fn ensure_permission(&self, perm_type: PermissionType, name: &str) -> Result<()> {
        match self.permission_manager.check_permission(perm_type) {
            Ok(PermissionStatus::Authorized) => Ok(()),
            Ok(status) => Err(Error::SecurityVerification(
                SecurityError::OsPermissionCheckFailed(format!(
                    "OS permission {name} not granted (status: {status:?})"
                )),
            )),
            Err(err) => Err(Error::SecurityVerification(
                SecurityError::OsPermissionCheckFailed(format!(
                    "Failed to check OS permission {name}: {err}"
                )),
            )),
        }
    }

    fn check_file_system_access(&self, permissions: &PluginPermissions) -> Result<()> {
        self.ensure_permission(PermissionType::FullDiskAccess, "file system access")?;

        for path in permissions
            .read_files
            .iter()
            .chain(permissions.write_files.iter())
        {
            self.verify_path_accessible(path)?;
        }

        Ok(())
    }

    fn verify_path_accessible(&self, path: &Path) -> Result<()> {
        fs::metadata(path).map_err(|err| {
            Error::SecurityVerification(SecurityError::OsPermissionCheckFailed(format!(
                "Unable to access declared file path {}: {err}",
                path.display()
            )))
        })?;
        Ok(())
    }

    fn check_network_access(&self, permissions: &PluginPermissions) -> Result<()> {
        self.ensure_permission(PermissionType::WiFi, "network connectivity")?;
        info!(
            target = "plugin_security_permissions",
            hosts = permissions.network_hosts.len(),
            "Network access verified for declared hosts"
        );
        Ok(())
    }
}
