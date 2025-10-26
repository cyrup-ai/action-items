use super::audit::AuditLogger;
use super::os_permissions::OsPermissionChecker;
use super::signature::SignatureVerifier;
use crate::error::{Error, Result, SecurityError};
use crate::plugins::interface::PluginManifest;

/// Production-grade capability verifier with cryptographic signatures and OS-level checks
pub struct CapabilityVerifier {
    signature_verifier: SignatureVerifier,
    os_checker: OsPermissionChecker,
    audit_logger: AuditLogger,
}

impl CapabilityVerifier {
    /// Initialize verifier with system configuration
    pub fn new() -> Result<Self> {
        Ok(Self {
            signature_verifier: SignatureVerifier::new()?,
            os_checker: OsPermissionChecker::new()?,
            audit_logger: AuditLogger::new()?,
        })
    }

    /// Comprehensive security verification with fail-secure design
    pub fn verify_capability(
        &self,
        plugin_id: &str,
        manifest: &PluginManifest,
        action_id: &str,
    ) -> Result<bool> {
        // Start audit log entry
        let audit_id = self.audit_logger.begin_verification(plugin_id, action_id);

        // Step 1: Verify manifest signature (cryptographic verification)
        if let Err(e) = self.signature_verifier.verify_manifest(manifest) {
            self.audit_logger
                .log_failure(audit_id, "signature_verification", &e);
            return Err(e);
        }

        // Step 2: Check OS-level permissions
        if let Err(e) = self.os_checker.check_permissions(manifest) {
            self.audit_logger
                .log_failure(audit_id, "os_permission_check", &e);
            return Err(e);
        }

        // Step 3: Validate specific capability grant for this action
        let granted = self.validate_action_capability(manifest, action_id)?;

        if granted {
            self.audit_logger.log_success(audit_id, action_id);
            Ok(true)
        } else {
            self.audit_logger
                .log_denial(audit_id, action_id, "capability_not_granted");
            Ok(false)
        }
    }

    /// Validate that the manifest grants the capability needed for this action
    fn validate_action_capability(
        &self,
        manifest: &PluginManifest,
        action_id: &str,
    ) -> Result<bool> {
        let capabilities = &manifest.capabilities;

        // Map action_id to required capability
        let granted = match action_id {
            "search" => capabilities.search,
            "execute" => capabilities.system_commands,
            "read_file" | "write_file" => capabilities.file_system_access,
            "http_request" => capabilities.network_access,
            "clipboard_read" | "clipboard_write" => capabilities.clipboard_access,
            "notify" => capabilities.notifications,
            _ => return Err(Error::SecurityVerification(SecurityError::CapabilityNotGranted(
                format!("Unknown action '{}' requested", action_id),
            ))),
        };

        Ok(granted)
    }
}
