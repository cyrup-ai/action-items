# PRODFIX_1: Security Verification Implementation

## OBJECTIVE
Complete the remaining production-grade security verification features for native plugins.

## PRIORITY
**P0 - CRITICAL SECURITY VULNERABILITY**

## OUTSTANDING WORK

1. **macOS Code Signing**
   * File: `packages/core/src/plugins/security/signature.rs`
   * Implement `verify_macos_codesign` using `objc2-security` (`SecStaticCodeCheckValidity`, certificate chain validation).
   * Current code only checks that `macos_bundle_path` exists.

2. **Linux Signature Verification**
   * File: `packages/core/src/plugins/security/signature.rs`
   * Implement `verify_linux_signature` with `polkit` authorization checks and manifest signature validation (GPG/trusted keyring).
   * Current code only validates presence of `polkit_action` metadata.

3. **Windows Authenticode Verification**
   * File: `packages/core/src/plugins/security/signature.rs`
   * Implement `verify_windows_authenticode` using WinTrust (`WinVerifyTrust`, certificate/revocation validation).
   * Current code only verifies that `windows_binary_path` exists.

## NOTES
- OS-level permission verification (`os_permissions.rs`) and audit logging (`audit.rs`) are production-ready.
- Manifest hashing and capability mapping meet requirements; focus on platform signature integrations above.

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