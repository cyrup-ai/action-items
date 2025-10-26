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
