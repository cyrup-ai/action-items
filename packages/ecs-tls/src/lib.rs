//! ECS TLS Service
//!
//! TLS certificate management with Bevy ECS integration for Action Items.
//! Provides secure certificate validation, OCSP caching, and CRL management.

pub mod tls;

// Re-export the main TLS components
pub use tls::builder::{CertificateAuthority, Tls};
pub use tls::cleanup_systems::{
    CrlCleanupTimer, OcspCleanupTimer, TlsCacheHolder, TlsCleanupPlugin, crl_cache_cleanup_system,
    ocsp_cache_cleanup_system,
};
// Re-export error types
pub use tls::errors::*;
pub use tls::types::*;
