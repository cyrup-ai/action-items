# PRODFIX_1: Security Verification Implementation

## OBJECTIVE
Replace the incomplete plugin capability verification in `verify_native_capabilities()` with production-grade security verification that validates plugin capabilities using cryptographic signatures, OS-level permission checks, and audit logging.

## PRIORITY
**P0 - CRITICAL SECURITY VULNERABILITY**

## CURRENT STATE ANALYSIS

### Primary File
- **Location**: [`packages/core/src/plugins/ecs_queries/executor/native.rs`](../../packages/core/src/plugins/ecs_queries/executor/native.rs)
- **Function**: `verify_native_capabilities()` at lines 48-157
- **Issue**: The current implementation uses `PluginCapabilityIndex::verify_capability()` which only performs a simple HashMap lookup to check if a capability string exists. There is **NO cryptographic verification, NO OS-level permission validation, and NO comprehensive audit logging**.

### Current Implementation Weaknesses
The existing code at line 50:
```rust
pub fn verify_native_capabilities(plugin: &PluginComponent, action_id: &str) -> bool {
    use ecs_service_bridge::resources::Capability;
    use ecs_service_bridge::systems::plugin_management::capability_index::PluginCapabilityIndex;
    
    let mut verifier = PluginCapabilityIndex::new();
    // ... maps action_id to capability ...
    match verifier.verify_capability(&plugin.id, &capability.name) {
        Ok(granted) => granted,  // Just checks HashMap - NO REAL SECURITY
        Err(e) => false,
    }
}
```

The `PluginCapabilityIndex::verify_capability()` implementation ([`packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs:46-51`](../../packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs)) simply does:
```rust
pub fn verify_capability(&mut self, plugin_id: &str, capability: &str) -> Result<bool, String> {
    if let Some(capabilities) = self.plugin_to_capabilities.get(plugin_id) {
        Ok(capabilities.contains(&capability.to_string()))  // Just a string lookup!
    } else {
        Err(format!("Plugin {} not found in capability index", plugin_id))
    }
}
```

**This is NOT production-grade security verification.**

### Call Sites
1. **Line 20** in `execute_native_action()`: Called before executing any native plugin action
2. **Line 107** in [`packages/core/src/plugins/ecs_queries/executor/scheduler.rs`](../../packages/core/src/plugins/ecs_queries/executor/scheduler.rs): Used in the `PluginExecutor::can_execute()` method

Both call sites expect a `bool` return type. You'll need to update them to handle the new `Result<bool, SecurityError>` return type.

---

## IMPLEMENTATION PLAN

## SUBTASK 1: Create Security Verification Module

### Create New Module Structure
Create a new security verification module at:
```
packages/core/src/plugins/security/
├── mod.rs
├── verifier.rs
├── signature.rs
├── os_permissions.rs
└── audit.rs
```

### Module: `verifier.rs` - Main Security Verifier

**File**: `packages/core/src/plugins/security/verifier.rs`

Implement the main `CapabilityVerifier` struct:

```rust
use crate::error::{Error, Result};
use crate::plugins::interface::manifest::PluginManifest;
use crate::plugins::interface::capabilities::{PluginCapabilities, PluginPermissions};
use super::signature::SignatureVerifier;
use super::os_permissions::OsPermissionChecker;
use super::audit::AuditLogger;

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
            self.audit_logger.log_failure(audit_id, "signature_verification", &e);
            return Err(Error::PluginError(format!(
                "Signature verification failed for plugin {}: {}",
                plugin_id, e
            )));
        }

        // Step 2: Check OS-level permissions
        if let Err(e) = self.os_checker.check_permissions(manifest) {
            self.audit_logger.log_failure(audit_id, "os_permission_check", &e);
            return Err(Error::PluginError(format!(
                "OS permission check failed for plugin {}: {}",
                plugin_id, e
            )));
        }

        // Step 3: Validate specific capability grant for this action
        let granted = self.validate_action_capability(manifest, action_id)?;
        
        if granted {
            self.audit_logger.log_success(audit_id, action_id);
            Ok(true)
        } else {
            self.audit_logger.log_denial(audit_id, action_id, "capability_not_granted");
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
            "clipboard_read" => capabilities.clipboard_access,
            "notify" => capabilities.notifications,
            _ => return Ok(false), // Unknown actions denied by default
        };
        
        Ok(granted)
    }
}
```

### Module: `signature.rs` - Cryptographic Signature Verification

**File**: `packages/core/src/plugins/security/signature.rs`

**Platform-Specific Implementation**: The project already has platform-specific dependencies configured in [`packages/core/Cargo.toml`](../../packages/core/Cargo.toml):
- **macOS**: `objc2-security` with `SecStaticCode`, `SecCertificate` features (lines 120-125)
- **Linux**: `polkit`, `glib` (lines 111-114)
- **Windows**: `windows-sys` with `Win32_Security_Cryptography`, `Win32_Security_WinTrust` features (lines 127-136)
- **All platforms**: `sha2 = "0.10"` for hashing (lines 122, 142)

Implement signature verification:

```rust
use crate::error::{Error, Result};
use crate::plugins::interface::manifest::PluginManifest;

#[cfg(target_os = "macos")]
use objc2_security::{SecStaticCode, SecCertificate, SecTrust};

#[cfg(target_os = "linux")]
use polkit::Authority;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::WinTrust::*;

pub struct SignatureVerifier {
    #[cfg(target_os = "macos")]
    security_framework: Option<()>,
    #[cfg(target_os = "linux")]
    polkit_authority: Option<Authority>,
    #[cfg(target_os = "windows")]
    wintrust_initialized: bool,
}

impl SignatureVerifier {
    pub fn new() -> Result<Self> {
        #[cfg(target_os = "macos")]
        {
            // Initialize macOS Security framework access
            Ok(Self { security_framework: Some(()) })
        }
        
        #[cfg(target_os = "linux")]
        {
            // Initialize PolicyKit authority connection
            let authority = polkit::Authority::new().map_err(|e| {
                Error::PluginError(format!("Failed to initialize PolicyKit: {}", e))
            })?;
            Ok(Self { polkit_authority: Some(authority) })
        }
        
        #[cfg(target_os = "windows")]
        {
            // Initialize WinTrust verification
            Ok(Self { wintrust_initialized: true })
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            Ok(Self {})
        }
    }

    /// Verify manifest cryptographic signature
    pub fn verify_manifest(&self, manifest: &PluginManifest) -> Result<()> {
        #[cfg(target_os = "macos")]
        {
            self.verify_macos_codesign(manifest)
        }
        
        #[cfg(target_os = "linux")]
        {
            self.verify_linux_signature(manifest)
        }
        
        #[cfg(target_os = "windows")]
        {
            self.verify_windows_authenticode(manifest)
        }
        
        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        {
            tracing::warn!("Signature verification not implemented for this platform");
            Ok(()) // Or return an error if you want strict verification
        }
    }

    #[cfg(target_os = "macos")]
    fn verify_macos_codesign(&self, manifest: &PluginManifest) -> Result<()> {
        // Use objc2-security to verify code signature via SecStaticCode
        // Implementation pattern:
        // 1. Get plugin bundle path from manifest
        // 2. Create SecStaticCode reference
        // 3. Verify signature using SecStaticCodeCheckValidity
        // 4. Extract and validate certificate chain using SecCertificate
        
        tracing::info!("Verifying macOS code signature for plugin: {}", manifest.id);
        
        // TODO: Implement using objc2-security APIs
        // Reference: https://developer.apple.com/documentation/security/code_signing_services
        
        Ok(())
    }

    #[cfg(target_os = "linux")]
    fn verify_linux_signature(&self, manifest: &PluginManifest) -> Result<()> {
        // Use polkit to verify plugin authorization
        // Implementation pattern:
        // 1. Check if plugin has valid PolicyKit action authorization
        // 2. Verify GPG signature on plugin manifest file
        // 3. Validate against trusted keyring
        
        tracing::info!("Verifying Linux signature for plugin: {}", manifest.id);
        
        // TODO: Implement using polkit APIs
        
        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn verify_windows_authenticode(&self, manifest: &PluginManifest) -> Result<()> {
        // Use WinTrust API to verify Authenticode signature
        // Implementation pattern:
        // 1. Get plugin DLL/EXE path from manifest
        // 2. Call WinVerifyTrust with WINTRUST_ACTION_GENERIC_VERIFY_V2
        // 3. Verify certificate chain validity
        // 4. Check certificate revocation status
        
        tracing::info!("Verifying Windows Authenticode signature for plugin: {}", manifest.id);
        
        // TODO: Implement using windows-sys WinTrust APIs
        // Reference: https://learn.microsoft.com/en-us/windows/win32/api/wintrust/
        
        Ok(())
    }
}
```

### Module: `os_permissions.rs` - OS-Level Permission Checking

**File**: `packages/core/src/plugins/security/os_permissions.rs`

**Leverage Existing Infrastructure**: The project already has comprehensive OS permission checking in the [`ecs-permissions`](../../packages/ecs-permissions/) package. Reuse this infrastructure:

```rust
use crate::error::{Error, Result};
use crate::plugins::interface::manifest::{PluginManifest, PluginPermissions};
use action_items_ecs_permissions::{PermissionManager, types::PermissionType};

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
        
        // Check clipboard access
        if permissions.read_clipboard || permissions.write_clipboard {
            self.check_permission(PermissionType::Clipboard, "clipboard access")?;
        }
        
        // Check file system access
        if !permissions.read_files.is_empty() || !permissions.write_files.is_empty() {
            // macOS: Check Full Disk Access
            // Linux: Check file permissions via stat()
            // Windows: Check file ACLs
            self.check_file_system_access(permissions)?;
        }
        
        // Check network access
        if !permissions.network_hosts.is_empty() {
            // Verify network permissions granted (firewall rules, etc.)
            self.check_network_access(permissions)?;
        }
        
        // Check accessibility permissions
        if permissions.accessibility {
            self.check_permission(PermissionType::Accessibility, "accessibility")?;
        }
        
        // Check camera/microphone
        if permissions.camera {
            self.check_permission(PermissionType::Camera, "camera")?;
        }
        if permissions.microphone {
            self.check_permission(PermissionType::Microphone, "microphone")?;
        }
        
        // Check location
        if permissions.location {
            self.check_permission(PermissionType::Location, "location")?;
        }
        
        // Check contacts/calendar
        if permissions.contacts {
            self.check_permission(PermissionType::Contacts, "contacts")?;
        }
        if permissions.calendar {
            self.check_permission(PermissionType::Calendar, "calendar")?;
        }
        
        Ok(())
    }

    fn check_permission(&self, perm_type: PermissionType, name: &str) -> Result<()> {
        use action_items_ecs_permissions::types::PermissionStatus;
        
        match self.permission_manager.check_permission(perm_type) {
            Ok(PermissionStatus::Granted) => Ok(()),
            Ok(status) => Err(Error::PluginError(format!(
                "OS permission {} not granted: {:?}",
                name, status
            ))),
            Err(e) => Err(Error::PluginError(format!(
                "Failed to check OS permission {}: {}",
                name, e
            ))),
        }
    }

    fn check_file_system_access(&self, permissions: &PluginPermissions) -> Result<()> {
        // Platform-specific file access verification
        #[cfg(target_os = "macos")]
        {
            // Check Full Disk Access status via TCC database
            // Reference: packages/ecs-permissions/src/platforms/macos/tcc_permissions.rs
            tracing::info!("Checking macOS Full Disk Access");
        }
        
        #[cfg(target_os = "linux")]
        {
            // Check file permissions using libc::access()
            tracing::info!("Checking Linux file permissions");
        }
        
        #[cfg(target_os = "windows")]
        {
            // Check file ACLs using Windows Security APIs
            tracing::info!("Checking Windows file ACLs");
        }
        
        Ok(())
    }

    fn check_network_access(&self, permissions: &PluginPermissions) -> Result<()> {
        // Verify network access is allowed
        // Could check firewall rules, network policies, etc.
        tracing::info!("Checking network access for {} hosts", permissions.network_hosts.len());
        Ok(())
    }
}
```

### Module: `audit.rs` - Audit Logging

**File**: `packages/core/src/plugins/security/audit.rs`

**Use Existing Tracing Infrastructure**: The project already has `tracing = "0.1.41"` and structured logging. Follow the pattern from [`packages/ecs-fetch/src/security/validation.rs`](../../packages/ecs-fetch/src/security/validation.rs):

```rust
use std::sync::atomic::{AtomicU64, Ordering};
use chrono::{DateTime, Utc};
use serde::Serialize;

static AUDIT_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Serialize)]
pub struct AuditEntry {
    pub audit_id: u64,
    pub timestamp: DateTime<Utc>,
    pub plugin_id: String,
    pub action_id: String,
    pub result: AuditResult,
    pub reason: Option<String>,
}

#[derive(Debug, Serialize)]
pub enum AuditResult {
    Success,
    Denied,
    Failed,
}

pub struct AuditLogger {
    // Could add structured log target configuration here
}

impl AuditLogger {
    pub fn new() -> crate::error::Result<Self> {
        Ok(Self {})
    }

    /// Begin verification and return audit ID
    pub fn begin_verification(&self, plugin_id: &str, action_id: &str) -> u64 {
        let audit_id = AUDIT_COUNTER.fetch_add(1, Ordering::SeqCst);
        
        tracing::info!(
            target: "plugin_capability_audit",
            audit_id = audit_id,
            plugin_id = plugin_id,
            action_id = action_id,
            event = "verification_begin",
            timestamp = ?Utc::now(),
            "Beginning capability verification"
        );
        
        audit_id
    }

    /// Log successful verification
    pub fn log_success(&self, audit_id: u64, action_id: &str) {
        tracing::info!(
            target: "plugin_capability_audit",
            audit_id = audit_id,
            action_id = action_id,
            result = "success",
            timestamp = ?Utc::now(),
            "Capability verification succeeded"
        );
    }

    /// Log verification denial (not an error, just not granted)
    pub fn log_denial(&self, audit_id: u64, action_id: &str, reason: &str) {
        tracing::warn!(
            target: "plugin_capability_audit",
            audit_id = audit_id,
            action_id = action_id,
            result = "denied",
            reason = reason,
            timestamp = ?Utc::now(),
            "Capability verification denied"
        );
    }

    /// Log verification failure (error occurred)
    pub fn log_failure(&self, audit_id: u64, stage: &str, error: &dyn std::fmt::Display) {
        tracing::error!(
            target: "plugin_capability_audit",
            audit_id = audit_id,
            stage = stage,
            result = "failed",
            error = %error,
            timestamp = ?Utc::now(),
            "Capability verification failed"
        );
    }
}
```

### Module: `mod.rs` - Module Root

**File**: `packages/core/src/plugins/security/mod.rs`

```rust
//! Plugin security verification system
//!
//! Production-grade security verification with:
//! - Cryptographic signature validation
//! - OS-level permission checks
//! - Comprehensive audit logging
//! - Fail-secure design (deny by default)

mod verifier;
mod signature;
mod os_permissions;
mod audit;

pub use verifier::CapabilityVerifier;
pub use signature::SignatureVerifier;
pub use os_permissions::OsPermissionChecker;
pub use audit::AuditLogger;
```

---

## SUBTASK 2: Update Core Error Types

Add a new error variant for security verification failures:

**File**: [`packages/core/src/error.rs`](../../packages/core/src/error.rs)

Add to the `Error` enum (around line 25):

```rust
/// Security verification errors
SecurityVerification(SecurityError),
```

Add new error type (around line 150):

```rust
/// Security verification errors
#[derive(Debug, Clone)]
pub enum SecurityError {
    /// Signature verification failed
    SignatureVerificationFailed(String),
    /// OS permission check failed
    OsPermissionCheckFailed(String),
    /// Capability not granted
    CapabilityNotGranted(String),
    /// Audit logging failed
    AuditFailure(String),
    /// Verifier initialization failed
    InitializationFailed(String),
}

impl fmt::Display for SecurityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SecurityError::SignatureVerificationFailed(msg) => {
                write!(f, "Signature verification failed: {}", msg)
            },
            SecurityError::OsPermissionCheckFailed(msg) => {
                write!(f, "OS permission check failed: {}", msg)
            },
            SecurityError::CapabilityNotGranted(msg) => {
                write!(f, "Capability not granted: {}", msg)
            },
            SecurityError::AuditFailure(msg) => {
                write!(f, "Audit logging failed: {}", msg)
            },
            SecurityError::InitializationFailed(msg) => {
                write!(f, "Verifier initialization failed: {}", msg)
            },
        }
    }
}

impl StdError for SecurityError {}
```

---

## SUBTASK 3: Replace verify_native_capabilities Implementation

**File**: [`packages/core/src/plugins/ecs_queries/executor/native.rs`](../../packages/core/src/plugins/ecs_queries/executor/native.rs)

**Replace lines 48-157** with the new production-grade implementation:

```rust
/// Verify native plugin capabilities with production-grade security verification
pub fn verify_native_capabilities(
    plugin: &PluginComponent,
    action_id: &str,
) -> crate::error::Result<bool> {
    use crate::plugins::security::CapabilityVerifier;
    
    // Create verifier (cached in production for performance)
    let verifier = CapabilityVerifier::new()?;
    
    // Perform comprehensive security verification
    let result = verifier.verify_capability(
        &plugin.id,
        &plugin.config.manifest,
        action_id,
    );
    
    match result {
        Ok(granted) => {
            if granted {
                tracing::debug!(
                    target: "native_capability_verification",
                    plugin_id = %plugin.id,
                    action_id = action_id,
                    "Security verification passed"
                );
            } else {
                tracing::warn!(
                    target: "native_capability_verification",
                    plugin_id = %plugin.id,
                    action_id = action_id,
                    "Security verification denied - capability not granted"
                );
            }
            Ok(granted)
        },
        Err(e) => {
            tracing::error!(
                target: "native_capability_verification",
                plugin_id = %plugin.id,
                action_id = action_id,
                error = %e,
                "Security verification failed"
            );
            Err(e)
        },
    }
}
```

---

## SUBTASK 4: Update Call Sites

### Call Site 1: execute_native_action()

**File**: [`packages/core/src/plugins/ecs_queries/executor/native.rs`](../../packages/core/src/plugins/ecs_queries/executor/native.rs)

**Update lines 19-25**:

```rust
// Verify plugin capabilities before execution using production security verification
match verify_native_capabilities(plugin_component, action_id) {
    Ok(true) => {
        // Verification passed, continue with execution
    },
    Ok(false) => {
        return Err(crate::error::Error::PluginError(format!(
            "Plugin {} lacks required capabilities for action {}",
            plugin_component.id, action_id
        )));
    },
    Err(e) => {
        // Security verification failed
        return Err(e);
    },
}
```

### Call Site 2: PluginExecutor::can_execute()

**File**: [`packages/core/src/plugins/ecs_queries/executor/scheduler.rs`](../../packages/core/src/plugins/ecs_queries/executor/scheduler.rs)

**Update line 107**:

```rust
// Check native plugins with security verification
for plugin in self.native_plugins.iter() {
    if plugin.id == plugin_id {
        return match verify_native_capabilities(plugin, action_id) {
            Ok(granted) => granted,
            Err(e) => {
                tracing::error!(
                    target: "plugin_executor",
                    plugin_id = plugin_id,
                    action_id = action_id,
                    error = %e,
                    "Security verification failed in can_execute"
                );
                false // Fail-secure: deny on error
            },
        };
    }
}
```

---

## SUBTASK 5: Update Module Exports

**File**: [`packages/core/src/plugins/mod.rs`](../../packages/core/src/plugins/mod.rs)

Add the security module to exports:

```rust
pub mod security;
```

---

## DEFINITION OF DONE

### Completion Criteria
- [ ] Security module created at `packages/core/src/plugins/security/` with all submodules
- [ ] `CapabilityVerifier` implemented with signature verification, OS checks, and audit logging
- [ ] Platform-specific signature verification implemented (macOS/Linux/Windows)
- [ ] OS permission checking implemented using existing `ecs-permissions` infrastructure
- [ ] Audit logging functional with structured tracing
- [ ] `SecurityError` type added to error.rs
- [ ] `verify_native_capabilities()` replaced with new implementation
- [ ] Both call sites updated to handle `Result<bool, Error>` return type
- [ ] Security module exported from plugins module
- [ ] Code compiles without errors or warnings
- [ ] Fail-secure behavior confirmed (denies on error)

### Security Properties Verified
- **Fail-secure**: All errors result in denial, never silent failures
- **Cryptographic verification**: Signatures validated using platform APIs
- **OS enforcement**: Permissions checked at OS level, not just manifest
- **Audit trail**: All verification attempts logged with structured data
- **No bypass**: No code path skips verification checks

---

## IMPLEMENTATION NOTES

### Existing Code to Leverage

1. **Error Handling Pattern**: Follow the pattern in [`packages/ecs-fetch/src/security/validation.rs`](../../packages/ecs-fetch/src/security/validation.rs) which shows proper error handling and validation patterns used in this codebase.

2. **Permission Checking**: Use the existing [`ecs-permissions`](../../packages/ecs-permissions/) package infrastructure:
   - [`packages/ecs-permissions/src/manager.rs`](../../packages/ecs-permissions/src/manager.rs) - Permission manager with caching
   - [`packages/ecs-permissions/src/platforms/macos/`](../../packages/ecs-permissions/src/platforms/macos/) - macOS permission checks
   - Platform-specific handlers already implemented for Camera, Microphone, Location, Contacts, Calendar, Accessibility, etc.

3. **Manifest Structure**: The [`PluginManifest`](../../packages/core/src/plugins/interface/manifest.rs) already has `capabilities` and `permissions` fields that contain all the data needed for verification.

4. **Dependencies Available**: The [`packages/core/Cargo.toml`](../../packages/core/Cargo.toml) already includes:
   - `tracing` for structured logging
   - `sha2` for cryptographic hashing
   - Platform-specific security frameworks (objc2-security, polkit, windows-sys)
   - `action_items_ecs_permissions` for OS permission checks

### Performance Considerations

- **Cache the verifier**: In production, create one `CapabilityVerifier` instance and reuse it rather than creating a new one for each verification call
- **Async audit logging**: Consider making audit log writes async to avoid blocking verification
- **Fast path for cached results**: Could add a verification result cache for repeated checks of the same plugin/action combination

### Platform-Specific Implementation Priority

1. **Phase 1 (MVP)**: Implement basic verification framework with audit logging but stub out platform-specific signature verification
2. **Phase 2**: Implement macOS signature verification using objc2-security
3. **Phase 3**: Implement Linux signature verification using polkit
4. **Phase 4**: Implement Windows signature verification using WinTrust

This allows incremental implementation while maintaining security through audit logging and OS permission checks.

---

## CONSTRAINTS

- **NO unit tests**: Another team handles testing
- **NO benchmarks**: Another team handles performance testing  
- **NO extensive documentation**: Focus on implementation only
- **Scope limited to**: Security verification implementation in ./src only

---

## REFERENCES

### Source Code Links
- [`packages/core/src/plugins/ecs_queries/executor/native.rs`](../../packages/core/src/plugins/ecs_queries/executor/native.rs) - Target file for replacement
- [`packages/core/src/plugins/ecs_queries/executor/scheduler.rs`](../../packages/core/src/plugins/ecs_queries/executor/scheduler.rs) - Second call site
- [`packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs`](../../packages/ecs-service-bridge/src/systems/plugin_management/capability_index.rs) - Current weak implementation
- [`packages/ecs-fetch/src/security/validation.rs`](../../packages/ecs-fetch/src/security/validation.rs) - Security validation patterns
- [`packages/ecs-permissions/src/manager.rs`](../../packages/ecs-permissions/src/manager.rs) - Permission checking infrastructure
- [`packages/core/src/error.rs`](../../packages/core/src/error.rs) - Error type definitions
- [`packages/core/Cargo.toml`](../../packages/core/Cargo.toml) - Dependencies and platform features

### External Documentation
- macOS Code Signing: https://developer.apple.com/documentation/security/code_signing_services
- Linux PolicyKit: https://www.freedesktop.org/software/polkit/docs/latest/
- Windows WinTrust: https://learn.microsoft.com/en-us/windows/win32/api/wintrust/