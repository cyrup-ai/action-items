//! TLS configuration builders and management

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use anyhow::Result;
use rustls::pki_types::{CertificateDer, PrivateKeyDer, PrivatePkcs8KeyDer};
use rustls::{ClientConfig, RootCertStore, ServerConfig};
use tracing::info;

use super::certificate::parse_certificate_from_pem;
use super::certificate::parsing::verify_peer_certificate;
use super::crl_cache;
use super::errors::TlsError;
use super::ocsp::OcspCache;

/// Production TLS manager with comprehensive certificate lifecycle management
pub struct TlsManager {
    #[allow(dead_code)]
    cert_dir: std::path::PathBuf,
    #[allow(dead_code)] // Used in server_config, client_config, validate_certificate_chain, etc.
    ca_cert: CertificateDer<'static>,
    #[allow(dead_code)]
    ca_key: PrivatePkcs8KeyDer<'static>,
    #[allow(dead_code)] // Used in server_config, client_config methods
    server_cert: CertificateDer<'static>,
    #[allow(dead_code)] // Used in server_config, client_config methods
    server_key: PrivatePkcs8KeyDer<'static>,
    ocsp_cache: OcspCache,
    crl_cache: crl_cache::CrlCache,
    // Cache statistics tracking
    cache_hits: AtomicU64,
    cache_misses: AtomicU64,
}

impl TlsManager {
    /// Create a new TLS manager with self-signed certificates
    pub async fn new(cert_dir: std::path::PathBuf) -> Result<Self> {
        let (ca_cert, ca_key, server_cert, server_key, ocsp_cache, crl_cache) =
            super::certificate::new(cert_dir.clone()).await?;

        let tls_manager = Self {
            cert_dir,
            ca_cert,
            ca_key,
            server_cert,
            server_key,
            ocsp_cache,
            crl_cache,
            cache_hits: AtomicU64::new(0),
            cache_misses: AtomicU64::new(0),
        };

        // Cache cleanup is now handled via Bevy ECS systems in TlsCleanupPlugin
        // See cleanup_systems.rs for implementation

        Ok(tls_manager)
    }

    /// Get server TLS configuration
    #[allow(dead_code)] // Public TLS API method - not currently called but needed for TLS server setup
    pub fn server_config(&self) -> Result<ServerConfig> {
        let mut root_store = RootCertStore::empty();
        root_store.add(self.ca_cert.clone())?;

        let config = ServerConfig::builder()
            .with_client_cert_verifier(
                rustls::server::WebPkiClientVerifier::builder(Arc::new(root_store)).build()?,
            )
            .with_single_cert(
                vec![self.server_cert.clone()],
                PrivateKeyDer::Pkcs8(self.server_key.clone_key()),
            )?;

        Ok(config)
    }

    /// Get client TLS configuration
    #[allow(dead_code)] // Public TLS API method - not currently called but needed for TLS client setup
    pub fn client_config(&self) -> Result<ClientConfig> {
        let mut root_store = RootCertStore::empty();
        root_store.add(self.ca_cert.clone())?;

        let config = ClientConfig::builder()
            .with_root_certificates(root_store)
            .with_client_auth_cert(
                vec![self.server_cert.clone()],
                PrivateKeyDer::Pkcs8(self.server_key.clone_key()),
            )?;

        Ok(config)
    }

    /// Create a TLS cache holder component for Bevy ECS
    /// This replaces the old tokio::spawn-based cleanup tasks with proper Bevy systems
    #[allow(dead_code)] // Public API for ECS integration
    pub fn create_cache_holder(&self) -> super::cleanup_systems::TlsCacheHolder {
        super::cleanup_systems::TlsCacheHolder {
            ocsp_cache: self.ocsp_cache.clone(),
            crl_cache: self.crl_cache.clone(),
        }
    }

    /// Get a reference to the OCSP cache for manual operations
    #[allow(dead_code)] // Public API for cache access
    pub fn ocsp_cache(&self) -> &OcspCache {
        &self.ocsp_cache
    }

    /// Get a reference to the CRL cache for manual operations
    #[allow(dead_code)] // Public cache access method - available for external CRL operations
    pub fn crl_cache(&self) -> &crl_cache::CrlCache {
        &self.crl_cache
    }

    /// Validate certificate using OCSP (Online Certificate Status Protocol)
    pub async fn validate_certificate_ocsp(
        &self,
        cert_pem: &str,
        issuer_cert_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        let parsed_cert = parse_certificate_from_pem(cert_pem)?;

        // Parse issuer certificate if provided
        let issuer_cert = if let Some(issuer_pem) = issuer_cert_pem {
            Some(parse_certificate_from_pem(issuer_pem)?)
        } else {
            None
        };

        // Check if OCSP response is already cached
        let cache_key = hex::encode(&parsed_cert.serial_number);
        let was_cached = self.check_ocsp_cache_exists(&cache_key).await;

        let result = self
            .ocsp_cache
            .check_certificate(&parsed_cert, issuer_cert.as_ref())
            .await;

        // Track cache statistics based on whether entry was cached before operation
        if was_cached {
            self.record_cache_hit();
        } else {
            self.record_cache_miss();
        }

        match result {
            Ok(super::ocsp::OcspStatus::Good) => {
                tracing::info!("OCSP validation successful: certificate is valid");
                Ok(())
            },
            Ok(super::ocsp::OcspStatus::Revoked) => Err(TlsError::OcspValidation(
                "Certificate has been revoked".to_string(),
            )),
            Ok(super::ocsp::OcspStatus::Unknown) => {
                tracing::warn!("OCSP status unknown, proceeding with validation");
                Ok(())
            },
            Err(e) => {
                tracing::warn!("OCSP validation failed: {}, proceeding without OCSP", e);
                Ok(())
            },
        }
    }

    /// Validate certificate using CRL (Certificate Revocation List)  
    pub async fn validate_certificate_crl(&self, cert_pem: &str) -> Result<(), TlsError> {
        let parsed_cert = parse_certificate_from_pem(cert_pem)?;

        // Check if CRL entry is already cached
        let cache_key = hex::encode(&parsed_cert.serial_number);
        let was_cached = self.check_crl_cache_exists(&cache_key).await;

        let result = self
            .crl_cache
            .check_certificate_revocation(&parsed_cert)
            .await;

        // Track cache statistics based on whether entry was cached before operation
        if was_cached {
            self.record_cache_hit();
        } else {
            self.record_cache_miss();
        }

        match result {
            Ok(false) => {
                tracing::info!("CRL validation successful: certificate is not revoked");
                Ok(())
            },
            Ok(true) => Err(TlsError::CrlValidation(
                "Certificate has been revoked according to CRL".to_string(),
            )),
            Err(e) => {
                tracing::warn!("CRL validation failed: {}, proceeding without CRL", e);
                Ok(())
            },
        }
    }

    /// Validate certificate chain to root CA
    #[allow(dead_code)] // Public API for certificate validation
    pub async fn validate_certificate_chain(&self, cert_chain_pem: &str) -> Result<(), TlsError> {
        super::certificate::validate_certificate_chain(cert_chain_pem, &self.ca_cert).await
    }

    /// Verify peer certificate against expected hostname
    #[allow(dead_code)] // Public API for peer certificate verification
    pub fn verify_peer_certificate(
        cert_pem: &str,
        expected_hostname: &str,
    ) -> Result<(), TlsError> {
        verify_peer_certificate(cert_pem, expected_hostname)
    }

    /// Verify peer certificate with OCSP validation
    #[allow(dead_code)] // Public API for OCSP certificate verification
    pub async fn verify_peer_certificate_with_ocsp(
        &self,
        cert_pem: &str,
        expected_hostname: &str,
        issuer_cert_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        // Perform standard certificate validation
        Self::verify_peer_certificate(cert_pem, expected_hostname)?;

        // Additional OCSP validation
        self.validate_certificate_ocsp(cert_pem, issuer_cert_pem)
            .await?;

        info!(
            "Successfully verified peer certificate with OCSP for hostname: {}",
            expected_hostname
        );
        Ok(())
    }

    /// Verify peer certificate with comprehensive revocation checking (OCSP + CRL + Chain)
    #[allow(dead_code)] // Public API for comprehensive certificate verification
    pub async fn verify_peer_certificate_comprehensive(
        &self,
        cert_pem: &str,
        expected_hostname: &str,
        full_chain_pem: Option<&str>,
    ) -> Result<(), TlsError> {
        super::certificate::validation::verify_peer_certificate_comprehensive(
            self,
            cert_pem,
            expected_hostname,
            full_chain_pem,
            &self.ca_cert,
        )
        .await
    }

    /// Record cache hit for statistics tracking
    pub fn record_cache_hit(&self) {
        self.cache_hits.fetch_add(1, Ordering::Relaxed);
    }

    /// Record cache miss for statistics tracking  
    pub fn record_cache_miss(&self) {
        self.cache_misses.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current cache hit count
    pub fn get_cache_hits(&self) -> u64 {
        self.cache_hits.load(Ordering::Relaxed)
    }

    /// Get current cache miss count
    pub fn get_cache_misses(&self) -> u64 {
        self.cache_misses.load(Ordering::Relaxed)
    }

    /// Check if OCSP cache entry exists for given key
    async fn check_ocsp_cache_exists(&self, cache_key: &str) -> bool {
        self.ocsp_cache.has_cached_entry(cache_key).await
    }

    /// Check if CRL cache entry exists for given key
    async fn check_crl_cache_exists(&self, cache_key: &str) -> bool {
        self.crl_cache.has_cached_entry(cache_key).await
    }

    /// Start OCSP cleanup task
    /// Note: Modern implementation uses Bevy ECS systems - this method is for compatibility
    #[allow(dead_code)] // Legacy API compatibility method
    pub fn start_ocsp_cleanup_task(&self) {
        // OCSP cleanup is now handled by TlsCleanupPlugin in Bevy ECS
        // This method exists for API compatibility but delegates to ECS systems
        tracing::info!("OCSP cleanup managed by TlsCleanupPlugin - no explicit task needed");
    }

    /// Start CRL cleanup task  
    /// Note: Modern implementation uses Bevy ECS systems - this method is for compatibility
    #[allow(dead_code)] // Legacy API compatibility method
    #[allow(dead_code)] // Legacy API method - CRL cleanup now handled by Bevy TlsCleanupPlugin
    pub fn start_crl_cleanup_task(&self) {
        // CRL cleanup is now handled by TlsCleanupPlugin in Bevy ECS
        // This method exists for API compatibility but delegates to ECS systems
        tracing::info!("CRL cleanup managed by TlsCleanupPlugin - no explicit task needed");
    }
}
