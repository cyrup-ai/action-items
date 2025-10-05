//! Certificate Authority domain object and builders

use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use rand::Rng;
use serde::{Deserialize, Serialize};
use x509_cert::der::Decode;

use crate::tls::certificate::parse_certificate_from_pem;
use crate::tls::errors::TlsError;
use crate::tls::types::ParsedCertificate;

#[cfg(target_os = "macos")]
#[allow(unused_imports)]
use security_framework::{
    certificate::SecCertificate,
    item::{ItemClass, ItemSearchOptions, Limit, SearchResult, Reference},
    trust::SecTrust,
    trust_settings::{TrustSettings, Domain, TrustSettingsForCertificate},
};

/// Format subject/issuer name from HashMap to readable string
pub fn format_subject_name(name_map: &std::collections::HashMap<String, String>) -> String {
    let mut parts = Vec::new();

    // Common name first if present
    if let Some(cn) = name_map.get("CN") {
        parts.push(format!("CN={}", cn));
    }

    // Organization unit
    if let Some(ou) = name_map.get("OU") {
        parts.push(format!("OU={}", ou));
    }

    // Organization
    if let Some(o) = name_map.get("O") {
        parts.push(format!("O={}", o));
    }

    // Locality/City
    if let Some(l) = name_map.get("L") {
        parts.push(format!("L={}", l));
    }

    // State/Province
    if let Some(st) = name_map.get("ST") {
        parts.push(format!("ST={}", st));
    }

    // Country
    if let Some(c) = name_map.get("C") {
        parts.push(format!("C={}", c));
    }

    // Add any remaining fields that weren't handled above
    for (key, value) in name_map {
        if !["CN", "OU", "O", "L", "ST", "C"].contains(&key.as_str()) {
            parts.push(format!("{}={}", key, value));
        }
    }

    if parts.is_empty() {
        "Unknown".to_string()
    } else {
        parts.join(", ")
    }
}

/// Extract key algorithm from parsed certificate
pub fn extract_key_algorithm(parsed_cert: &ParsedCertificate) -> Result<String, TlsError> {
    // Parse the public key DER to get algorithm information
    let spki: x509_cert::spki::SubjectPublicKeyInfoRef =
        x509_cert::spki::SubjectPublicKeyInfo::from_der(&parsed_cert.public_key_der).map_err(
            |e| TlsError::CertificateParsing(format!("Failed to parse public key: {}", e)),
        )?;

    // Map algorithm OID to string representation
    match spki.algorithm.oid.to_string().as_str() {
        "1.2.840.113549.1.1.1" => Ok("RSA".to_string()),
        "1.2.840.10045.2.1" => Ok("ECDSA".to_string()),
        "1.3.101.112" => Ok("Ed25519".to_string()),
        "1.3.101.113" => Ok("Ed448".to_string()),
        oid => Err(TlsError::CertificateParsing(format!(
            "Unsupported key algorithm OID: {}",
            oid
        ))),
    }
}

/// Extract key size from parsed certificate
pub fn extract_key_size(parsed_cert: &ParsedCertificate) -> Option<u32> {
    // Parse the public key DER to get key size information
    let spki: x509_cert::spki::SubjectPublicKeyInfoRef =
        x509_cert::spki::SubjectPublicKeyInfo::from_der(&parsed_cert.public_key_der).ok()?;

    // Extract key size based on algorithm
    match spki.algorithm.oid.to_string().as_str() {
        "1.2.840.113549.1.1.1" => {
            // RSA - estimate from public key size
            let key_bytes = spki.subject_public_key.raw_bytes();
            // RSA public keys: rough estimation based on DER structure
            match key_bytes.len() {
                162..=170 => Some(1024u32), // 1024-bit RSA
                294..=310 => Some(2048u32), // 2048-bit RSA
                550..=570 => Some(4096u32), // 4096-bit RSA
                _ => Some(((key_bytes.len() - 22) * 8) as u32), /* Rough estimate minus DER
                                              * overhead */
            }
        },
        "1.2.840.10045.2.1" => {
            // ECDSA - standard curve sizes
            let key_bytes = spki.subject_public_key.raw_bytes();
            match key_bytes.len() {
                65 => Some(256u32),  // P-256 uncompressed
                97 => Some(384u32),  // P-384 uncompressed
                133 => Some(521u32), // P-521 uncompressed
                33 => Some(256u32),  // P-256 compressed
                49 => Some(384u32),  // P-384 compressed
                _ => None,           // Cannot determine curve size from key length
            }
        },
        "1.3.101.112" => Some(256u32), // Ed25519
        "1.3.101.113" => Some(448u32), // Ed448
        _ => None,
    }
}

/// Certificate Authority domain object with serialization support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthority {
    pub name: String,
    pub certificate_pem: String,
    pub private_key_pem: String,
    pub metadata: CaMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaMetadata {
    pub subject: String,
    pub issuer: String,
    pub serial_number: String,
    pub valid_from: SystemTime,
    pub valid_until: SystemTime,
    pub key_algorithm: String,
    pub key_size: Option<u32>,
    pub created_at: SystemTime,
    pub source: CaSource,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CaSource {
    Filesystem { path: PathBuf },
    Keychain,
    Remote { url: String },
    Generated,
}

impl CertificateAuthority {
    /// Check if the certificate authority is currently valid
    pub fn is_valid(&self) -> bool {
        let now = SystemTime::now();
        now >= self.metadata.valid_from && now <= self.metadata.valid_until
    }

    /// Get duration until expiry
    pub fn expires_in(&self) -> Result<Duration, TlsError> {
        let now = SystemTime::now();
        self.metadata.valid_until.duration_since(now).map_err(|_| {
            TlsError::CertificateExpired("Certificate authority has expired".to_string())
        })
    }

    /// Check if this CA can sign certificates for the given domain
    pub fn can_sign_for_domain(&self, domain: &str) -> bool {
        if !self.is_valid() {
            return false;
        }

        // Validate domain format
        if domain.is_empty() || domain.len() > 255 {
            return false;
        }

        // Check for valid domain characters and structure
        let parts: Vec<&str> = domain.split('.').collect();
        if parts.len() < 2 {
            return false; // Domain must have at least two parts
        }

        for part in &parts {
            if part.is_empty() || part.len() > 63 {
                return false;
            }

            // Check that part contains only valid domain characters
            if !part.chars().all(|c| c.is_alphanumeric() || c == '-') {
                return false;
            }

            // Cannot start or end with hyphen
            if part.starts_with('-') || part.ends_with('-') {
                return false;
            }
        }

        // Check if CA has domain constraints (if implemented)
        // For now, allow all valid domains if CA is valid
        true
    }
}

/// Builder for certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityBuilder {
    name: String,
}

impl AuthorityBuilder {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
        }
    }

    /// Work with filesystem-based certificate authority
    pub fn path<P: AsRef<Path>>(self, path: P) -> AuthorityFilesystemBuilder {
        AuthorityFilesystemBuilder {
            name: self.name,
            path: path.as_ref().to_path_buf(),
            common_name: None,
            valid_for_years: 10,
            key_size: 2048,
        }
    }

    /// Work with keychain-based certificate authority (macOS/Windows)
    pub fn keychain(self) -> AuthorityKeychainBuilder {
        AuthorityKeychainBuilder { name: self.name }
    }

    /// Work with remote certificate authority
    pub fn url(self, url: &str) -> AuthorityRemoteBuilder {
        AuthorityRemoteBuilder {
            name: self.name,
            url: url.to_string(),
            timeout: Duration::from_secs(30),
        }
    }
}

/// Builder for filesystem certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityFilesystemBuilder {
    name: String,
    path: PathBuf,
    common_name: Option<String>,
    valid_for_years: u32,
    #[allow(dead_code)] // CA key size configuration - used in future CA generation logic
    key_size: u32,
}

impl AuthorityFilesystemBuilder {
    /// Set common name for certificate authority creation
    pub fn common_name(self, cn: &str) -> Self {
        Self {
            common_name: Some(cn.to_string()),
            ..self
        }
    }

    /// Set validity period in years for certificate authority creation
    pub fn valid_for_years(self, years: u32) -> Self {
        Self {
            valid_for_years: years,
            ..self
        }
    }

    /// Set key size for certificate authority creation
    pub fn key_size(self, bits: u32) -> Self {
        Self {
            key_size: bits,
            ..self
        }
    }

    /// Create a new certificate authority
    pub async fn create(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;

        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair};

        // Create directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&self.path).await {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to create directory: {}", e)],
                files_created: vec![],
            };
        }

        // Generate CA certificate
        let mut params = match CertificateParams::new(vec![]) {
            Ok(p) => p,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![format!("Failed to create certificate params: {}", e)],
                    files_created: vec![],
                };
            },
        };

        let mut distinguished_name = DistinguishedName::new();
        let common_name = match self.common_name {
            Some(cn) => cn,
            None => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![
                        "Common name must be explicitly specified for CA certificate generation"
                            .to_string(),
                    ],
                    files_created: vec![],
                };
            },
        };
        distinguished_name.push(DnType::CommonName, &common_name);
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);
        params.distinguished_name = distinguished_name;

        // Set validity period
        let now = SystemTime::now();
        params.not_before = now.into();
        params.not_after = (now
            + std::time::Duration::from_secs(365 * 24 * 3600 * self.valid_for_years as u64))
        .into();

        // Generate key pair
        let key_pair =
            KeyPair::generate().map_err(|e| format!("Failed to generate key pair: {}", e));

        let key_pair = match key_pair {
            Ok(kp) => kp,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![e],
                    files_created: vec![],
                };
            },
        };

        let cert = match params.self_signed(&key_pair) {
            Ok(c) => c,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![format!("Failed to generate certificate: {}", e)],
                    files_created: vec![],
                };
            },
        };

        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // Both cert_pem and key_pem are now String, no error handling needed

        // Save files
        let cert_path = self.path.join("ca.crt");
        let key_path = self.path.join("ca.key");
        let mut files_created = vec![];

        if let Err(e) = tokio::fs::write(&cert_path, &cert_pem).await {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to write certificate: {}", e)],
                files_created,
            };
        }
        files_created.push(cert_path);

        if let Err(e) = tokio::fs::write(&key_path, &key_pem).await {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::CreateFailed,
                issues: vec![format!("Failed to write private key: {}", e)],
                files_created,
            };
        }
        files_created.push(key_path);

        // Parse the generated certificate to extract metadata
        let parsed_cert = match parse_certificate_from_pem(&cert_pem) {
            Ok(cert) => cert,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![format!("Failed to parse generated CA certificate: {}", e)],
                    files_created,
                };
            },
        };

        // Extract key algorithm from certificate
        let key_algorithm = match extract_key_algorithm(&parsed_cert) {
            Ok(alg) => alg,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::CreateFailed,
                    issues: vec![format!(
                        "Failed to extract key algorithm from generated CA certificate: {}",
                        e
                    )],
                    files_created,
                };
            },
        };

        // Extract key size from certificate
        let key_size = extract_key_size(&parsed_cert);

        // Generate unique serial number
        let serial_number = hex::encode(rand::rng().random::<[u8; 16]>());

        // Create authority object
        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: key_pem,
            metadata: CaMetadata {
                subject: common_name.clone(),
                issuer: common_name,
                serial_number,
                valid_from: now,
                valid_until: now
                    + std::time::Duration::from_secs(365 * 24 * 3600 * self.valid_for_years as u64),
                key_algorithm,
                key_size,
                created_at: now,
                source: CaSource::Generated,
            },
        };

        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Created,
            issues: vec![],
            files_created,
        }
    }

    /// Load existing certificate authority from filesystem
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        use std::time::SystemTime;

        use crate::tls::certificate::parse_certificate_from_pem;

        let cert_path = self.path.join("ca.crt");
        let key_path = self.path.join("ca.key");

        // Check if both files exist
        if !cert_path.exists() || !key_path.exists() {
            return super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("CA files not found at {:?}", self.path)],
                files_created: vec![],
            };
        }

        // Read certificate and key files
        let cert_pem = match tokio::fs::read_to_string(&cert_path).await {
            Ok(content) => content,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to read certificate: {}", e)],
                    files_created: vec![],
                };
            },
        };

        let key_pem = match tokio::fs::read_to_string(&key_path).await {
            Ok(content) => content,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to read private key: {}", e)],
                    files_created: vec![],
                };
            },
        };

        // Parse certificate to extract metadata
        let parsed_cert = match parse_certificate_from_pem(&cert_pem) {
            Ok(cert) => cert,
            Err(e) => {
                return super::responses::CertificateAuthorityResponse {
                    success: false,
                    authority: None,
                    operation: super::responses::CaOperation::LoadFailed,
                    issues: vec![format!("Failed to parse certificate: {}", e)],
                    files_created: vec![],
                };
            },
        };

        let authority = CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: key_pem,
            metadata: CaMetadata {
                subject: format_subject_name(&parsed_cert.subject),
                issuer: format_subject_name(&parsed_cert.issuer),
                serial_number: hex::encode(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                key_algorithm: extract_key_algorithm(&parsed_cert)
                    .unwrap_or_else(|_| "Unknown".to_string()),
                key_size: extract_key_size(&parsed_cert),
                created_at: SystemTime::now(),
                source: CaSource::Filesystem {
                    path: self.path.clone(),
                },
            },
        };

        super::responses::CertificateAuthorityResponse {
            success: true,
            authority: Some(authority),
            operation: super::responses::CaOperation::Loaded,
            issues: vec![],
            files_created: vec![],
        }
    }
}

/// Builder for keychain certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityKeychainBuilder {
    #[allow(dead_code)] // Keychain CA name - used in macOS keychain integration
    name: String,
}

impl AuthorityKeychainBuilder {
    /// Load certificate authority from system keychain
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        match self.load_from_keychain().await {
            Ok(authority) => super::responses::CertificateAuthorityResponse {
                success: true,
                authority: Some(authority),
                operation: super::responses::CaOperation::Loaded,
                issues: vec![],
                files_created: vec![],
            },
            Err(e) => super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Failed to load from keychain: {}", e)],
                files_created: vec![],
            },
        }
    }

    /// Load certificate from platform-specific keychain
    async fn load_from_keychain(&self) -> Result<CertificateAuthority, TlsError> {
        #[cfg(target_os = "macos")]
        {
            self.load_from_macos_keychain().await
        }
        #[cfg(target_os = "windows")]
        {
            self.load_from_windows_store().await
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows")))]
        {
            Err(TlsError::UnsupportedPlatform(
                "Keychain loading not supported on this platform".to_string(),
            ))
        }
    }

    #[cfg(target_os = "macos")]
    async fn load_from_macos_keychain(&self) -> Result<CertificateAuthority, TlsError> {
        // Use TCC permissions through ecs-permissions for proper macOS integration
        use action_items_ecs_permissions::platforms::macos::tcc_permissions;
        use action_items_ecs_permissions::types::{PermissionStatus, PermissionType};

        // Check if we have the necessary TCC permissions for keychain access
        let status =
            tcc_permissions::check_permission(PermissionType::DesktopFolder).map_err(|e| {
                TlsError::PermissionError(format!("TCC permission check failed: {}", e))
            })?;

        match status {
            PermissionStatus::Authorized => {
                // TCC permissions granted, proceed with keychain loading
                #[cfg(target_os = "macos")]
                {
                    // use security_framework::certificate::SecCertificate; // Unused import
                    // use security_framework::identity::SecIdentity; // Unused import
                    use security_framework::item::{ItemClass, ItemSearchOptions};
                    // use core_foundation::base::TCFType; // Unused import
                    // use base64::Engine; // Unused import

                    tracing::info!("Accessing macOS keychain for certificates");

                    // Search for certificates in the keychain
                    let mut search_options = ItemSearchOptions::new();
                    search_options.class(ItemClass::certificate())
                        .load_refs(true)
                        .limit(Limit::All);

                    let search_results = match search_options.search() {
                        Ok(results) => results,
                        Err(e) => {
                            let error_msg = format!("{}", e);
                            if error_msg.contains("permission") || error_msg.contains("access") {
                                return Err(TlsError::PermissionDenied(
                                    "macOS keychain access requires user permission. Please grant access when prompted.".to_string()
                                ));
                            } else if error_msg.contains("user interaction") {
                                return Err(TlsError::PermissionError(
                                    "Keychain access requires user interaction. Please run in an interactive environment.".to_string()
                                ));
                            } else {
                                return Err(TlsError::KeychainError(format!("Failed to search keychain: {}", e)));
                            }
                        }
                    };

                    if search_results.is_empty() {
                        tracing::info!("No certificates found in keychain, trying system root certificates");
                        return load_first_system_root_certificate();
                    }

                    let user_trust = TrustSettings::new(Domain::User);
                    let system_trust = TrustSettings::new(Domain::System);

                    for result in search_results {
                        if let SearchResult::Ref(Reference::Certificate(cert)) = result {
                            // Check certificate trust before including
                            let is_trusted = check_certificate_trust(&cert, &user_trust, &system_trust);
                            
                            if !is_trusted {
                                tracing::warn!("Skipping untrusted certificate");
                                continue;
                            }
                            
                            match convert_sec_certificate_to_authority(cert, CaSource::Keychain) {
                                Ok(authority) => {
                                    tracing::debug!("Loaded certificate: {}", authority.metadata.subject);
                                    return Ok(authority);
                                },
                                Err(e) => {
                                    tracing::warn!("Failed to convert certificate: {}", e);
                                    continue;
                                }
                            }
                        }
                    }

                    tracing::warn!("No trusted certificates found in keychain, falling back to system roots");
                    load_first_system_root_certificate()
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Err(TlsError::UnsupportedPlatform(
                        "Keychain access only available on macOS".to_string(),
                    ))
                }
            },
            PermissionStatus::Denied => Err(TlsError::PermissionError(
                "TCC permissions denied for keychain access".to_string(),
            )),
            PermissionStatus::NotDetermined => {
                // Request TCC permissions
                let _ = tcc_permissions::request_permission(PermissionType::DesktopFolder)
                    .map_err(|e| {
                        TlsError::PermissionError(format!("TCC permission request failed: {}", e))
                    })?;

                Err(TlsError::PermissionError(
                    "TCC permissions required for keychain access - please grant access in System \
                     Preferences"
                        .to_string(),
                ))
            },
            PermissionStatus::Restricted => Err(TlsError::PermissionError(
                "TCC permissions restricted by system policy for keychain access".to_string(),
            )),
            PermissionStatus::Unknown => Err(TlsError::PermissionError(
                "TCC permission status unknown for keychain access".to_string(),
            )),
        }
    }

    #[cfg(target_os = "windows")]
    async fn load_from_windows_store(&self) -> Result<CertificateAuthority, TlsError> {
        use std::ffi::CString;

        use windows_sys::Win32::Security::Cryptography::{
            CERT_CLOSE_STORE_FORCE_FLAG, CERT_CONTEXT, CERT_FIND_SUBJECT_STR_A, CertCloseStore,
            CertFindCertificateInStore, CertFreeCertificateContext, CertOpenSystemStoreA,
        };

        unsafe {
            // Open the Windows Certificate Store (ROOT store for CA certificates)
            let store_name = CString::new("ROOT")
                .map_err(|e| TlsError::KeychainError(format!("Invalid store name: {}", e)))?;

            let store_handle = CertOpenSystemStoreA(0, store_name.as_ptr() as *const u8);
            if store_handle.is_null() {
                return Err(TlsError::KeychainError(
                    "Failed to open Windows certificate store".to_string(),
                ));
            }

            // Search for certificate by subject name
            let subject_name = CString::new(self.name.as_str())
                .map_err(|e| TlsError::KeychainError(format!("Invalid subject name: {}", e)))?;

            let cert_context = CertFindCertificateInStore(
                store_handle,
                0x00000001, // X509_ASN_ENCODING
                0,
                CERT_FIND_SUBJECT_STR_A,
                subject_name.as_ptr() as *const _,
                std::ptr::null_mut(),
            );

            if cert_context.is_null() {
                CertCloseStore(store_handle, CERT_CLOSE_STORE_FORCE_FLAG);
                return Err(TlsError::CertificateNotFound(format!(
                    "No certificate found with subject: {}",
                    self.name
                )));
            }

            // Extract certificate data and convert to PEM
            let cert_context_ref = &*(cert_context as *const CERT_CONTEXT);
            let cert_der = std::slice::from_raw_parts(
                cert_context_ref.pbCertEncoded,
                cert_context_ref.cbCertEncoded as usize,
            );
            let cert_pem = der_to_pem(cert_der)?;

            // Parse certificate to extract metadata
            let parsed_cert = crate::tls::certificate::parse_certificate_from_pem(&cert_pem)?;

            // Clean up Windows resources
            CertFreeCertificateContext(cert_context);
            CertCloseStore(store_handle, CERT_CLOSE_STORE_FORCE_FLAG);

            Ok(CertificateAuthority {
                name: self.name.clone(),
                certificate_pem: cert_pem,
                private_key_pem: String::new(), /* Windows store typically doesn't export private
                                                 * keys */
                metadata: CaMetadata {
                    subject: format_subject_name(&parsed_cert.subject),
                    issuer: format_subject_name(&parsed_cert.issuer),
                    serial_number: hex::encode(&parsed_cert.serial_number),
                    valid_from: parsed_cert.not_before,
                    valid_until: parsed_cert.not_after,
                    key_algorithm: extract_key_algorithm(&parsed_cert)
                        .unwrap_or_else(|_| "Unknown".to_string()),
                    key_size: extract_key_size(&parsed_cert).map(|s| s as u32),
                    created_at: SystemTime::now(),
                    source: CaSource::Keychain,
                },
            })
        }
    }


}

/// Builder for remote certificate authority operations
#[derive(Debug, Clone)]
pub struct AuthorityRemoteBuilder {
    name: String,
    url: String,
    timeout: Duration,
}

impl AuthorityRemoteBuilder {
    /// Set timeout for remote operations
    pub fn with_timeout(self, timeout: Duration) -> Self {
        Self { timeout, ..self }
    }

    /// Load certificate authority from remote URL
    pub async fn load(self) -> super::responses::CertificateAuthorityResponse {
        match self.fetch_from_remote().await {
            Ok(authority) => super::responses::CertificateAuthorityResponse {
                success: true,
                authority: Some(authority),
                operation: super::responses::CaOperation::Loaded,
                issues: vec![],
                files_created: vec![],
            },
            Err(e) => super::responses::CertificateAuthorityResponse {
                success: false,
                authority: None,
                operation: super::responses::CaOperation::LoadFailed,
                issues: vec![format!("Failed to load from remote: {}", e)],
                files_created: vec![],
            },
        }
    }

    /// Fetch certificate from remote URL with proper validation
    async fn fetch_from_remote(&self) -> Result<CertificateAuthority, TlsError> {
        // Create HTTP client with configured timeout and security settings
        let client = reqwest::Client::builder()
            .timeout(self.timeout)
            .user_agent("ActionItems-TLS/1.0")
            .danger_accept_invalid_certs(false) // Always validate remote certificates
            .build()
            .map_err(|e| TlsError::NetworkError(format!("Failed to create HTTP client: {}", e)))?;

        // Validate URL format and security
        if !self.url.starts_with("https://") && !self.url.starts_with("http://") {
            return Err(TlsError::InvalidUrl(format!(
                "Invalid URL scheme: {}",
                self.url
            )));
        }

        // Warn about insecure HTTP for certificate fetching
        if self.url.starts_with("http://") {
            tracing::warn!(
                "Using insecure HTTP to fetch certificate from: {}",
                self.url
            );
        }

        // Fetch certificate from URL
        let response = client
            .get(&self.url)
            .header(
                "Accept",
                "application/x-pem-file, application/pkix-cert, text/plain, */*",
            )
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    TlsError::NetworkTimeout(format!("Request timeout after {:?}", self.timeout))
                } else if e.is_connect() {
                    TlsError::NetworkError(format!("Connection failed: {}", e))
                } else {
                    TlsError::NetworkError(format!("Request failed: {}", e))
                }
            })?;

        // Check HTTP response status
        if !response.status().is_success() {
            let status_text = response
                .status()
                .canonical_reason()
                .map(|reason| {
                    format!(
                        "HTTP {} {}: Failed to fetch certificate",
                        response.status().as_u16(),
                        reason
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "HTTP {}: Failed to fetch certificate",
                        response.status().as_u16()
                    )
                });
            return Err(TlsError::HttpError(status_text));
        }

        // Validate content type if provided
        if let Some(content_type) = response.headers().get("content-type") {
            match content_type.to_str() {
                Ok(content_type_str) => {
                    if !content_type_str.contains("pem")
                        && !content_type_str.contains("cert")
                        && !content_type_str.contains("text")
                        && !content_type_str.contains("application/x-x509")
                    {
                        tracing::warn!(
                            "Unexpected content-type for certificate: {}",
                            content_type_str
                        );
                    }
                },
                Err(_) => {
                    tracing::warn!(
                        "Invalid UTF-8 in content-type header, proceeding with download"
                    );
                },
            }
        }

        // Read response body
        let cert_data = response
            .text()
            .await
            .map_err(|e| TlsError::NetworkError(format!("Failed to read response body: {}", e)))?;

        // Validate that we received certificate data
        if cert_data.is_empty() {
            return Err(TlsError::InvalidCertificate(
                "Empty certificate data received".to_string(),
            ));
        }

        // Detect and handle different certificate formats
        let cert_pem = if cert_data.contains("-----BEGIN CERTIFICATE-----") {
            // Already in PEM format
            cert_data
        } else if cert_data.starts_with('\u{30}') || cert_data.as_bytes().first() == Some(&0x30) {
            // Looks like DER format, convert to PEM
            der_to_pem(cert_data.as_bytes())?
        } else {
            // Try to parse as base64 encoded DER
            use base64::Engine;
            match base64::engine::general_purpose::STANDARD.decode(&cert_data) {
                Ok(der_bytes) => der_to_pem(&der_bytes)?,
                Err(_) => {
                    return Err(TlsError::InvalidCertificate(
                        "Certificate data is not in PEM, DER, or base64 format".to_string(),
                    ));
                },
            }
        };

        // Parse and validate the certificate
        let parsed_cert =
            crate::tls::certificate::parse_certificate_from_pem(&cert_pem).map_err(|e| {
                TlsError::InvalidCertificate(format!("Failed to parse certificate: {}", e))
            })?;

        // Additional validation: check certificate is not expired
        let now = SystemTime::now();
        if now > parsed_cert.not_after {
            return Err(TlsError::CertificateExpired(format!(
                "Certificate expired on {:?}",
                parsed_cert.not_after
            )));
        }

        if now < parsed_cert.not_before {
            return Err(TlsError::CertificateNotYetValid(format!(
                "Certificate not valid until {:?}",
                parsed_cert.not_before
            )));
        }

        // Create authority object with parsed data
        Ok(CertificateAuthority {
            name: self.name.clone(),
            certificate_pem: cert_pem,
            private_key_pem: String::new(), // Remote CAs don't provide private keys
            metadata: CaMetadata {
                subject: format_subject_name(&parsed_cert.subject),
                issuer: format_subject_name(&parsed_cert.issuer),
                serial_number: hex::encode(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                key_algorithm: extract_key_algorithm(&parsed_cert)
                    .unwrap_or_else(|_| "Unknown".to_string()),
                key_size: extract_key_size(&parsed_cert),
                created_at: SystemTime::now(),
                source: CaSource::Remote {
                    url: self.url.clone(),
                },
            },
        })
    }


}

#[cfg(target_os = "macos")]
fn check_certificate_trust(
    cert: &SecCertificate,
    user_trust: &TrustSettings,
    system_trust: &TrustSettings,
) -> bool {
    // Check user domain trust settings first
    match user_trust.tls_trust_settings_for_certificate(cert) {
        Ok(Some(TrustSettingsForCertificate::TrustRoot)) => return true,
        Ok(Some(TrustSettingsForCertificate::TrustAsRoot)) => return true,
        Ok(Some(TrustSettingsForCertificate::Deny)) => return false,
        Ok(Some(TrustSettingsForCertificate::Unspecified)) => {}, // Continue to system check
        Ok(Some(TrustSettingsForCertificate::Invalid)) => {}, // Invalid trust settings, continue to system check
        Ok(None) => {}, // No specific settings, continue to system check
        Err(_) => {}, // No trust settings found, continue to system check
    }
    
    // Check system domain trust settings as fallback
    match system_trust.tls_trust_settings_for_certificate(cert) {
        Ok(Some(TrustSettingsForCertificate::TrustRoot)) => true,
        Ok(Some(TrustSettingsForCertificate::TrustAsRoot)) => true,
        Ok(Some(TrustSettingsForCertificate::Deny)) => false,
        _ => {
            // Default to trusted when trust cannot be determined
            // This provides graceful degradation for certificates without explicit trust settings
            tracing::debug!("Certificate trust could not be determined, defaulting to trusted");
            true
        }
    }
}

#[cfg(target_os = "macos")]
fn load_first_system_root_certificate() -> Result<CertificateAuthority, TlsError> {
    tracing::info!("Loading first system root certificate as fallback");
    
    let system_certs = SecTrust::copy_anchor_certificates()
        .map_err(|e| TlsError::KeychainError(format!("Failed to load system root certificates: {}", e)))?;
    
    if system_certs.is_empty() {
        return Err(TlsError::KeychainError(
            "No system root certificates available".to_string()
        ));
    }
    
    for cert in system_certs {
        match convert_sec_certificate_to_authority(cert, CaSource::Keychain) {
            Ok(authority) => {
                tracing::debug!("Loaded system root certificate: {}", authority.metadata.subject);
                return Ok(authority);
            },
            Err(e) => {
                tracing::warn!("Failed to convert system root certificate: {}", e);
                continue;
            }
        }
    }
    
    Err(TlsError::KeychainError(
        "Failed to convert any system root certificates".to_string()
    ))
}

#[cfg(target_os = "macos")]
fn convert_sec_certificate_to_authority(
    cert: SecCertificate,
    source: CaSource,
) -> Result<CertificateAuthority, TlsError> {
    // Convert SecCertificate to DER format
    let der_data = cert.to_der();
    
    // Convert DER to PEM format for compatibility with existing TLS code
    let cert_pem = der_to_pem(&der_data)?;
    
    // Parse certificate to extract metadata
    let parsed_cert = crate::tls::certificate::parse_certificate_from_pem(&cert_pem)
        .map_err(|e| TlsError::CertificateParsing(format!("Failed to parse keychain certificate: {}", e)))?;
    
    // Validate certificate is not expired
    let now = SystemTime::now();
    if now > parsed_cert.not_after {
        return Err(TlsError::CertificateExpired(format!(
            "Keychain certificate expired on {:?}",
            parsed_cert.not_after
        )));
    }
    
    if now < parsed_cert.not_before {
        return Err(TlsError::CertificateNotYetValid(format!(
            "Keychain certificate not valid until {:?}",
            parsed_cert.not_before
        )));
    }
    
    // Create CertificateAuthority object
    Ok(CertificateAuthority {
        name: format!("Keychain-{}", 
            parsed_cert.subject.get("CN")
                .unwrap_or(&"Unknown".to_string())
                .clone()
        ),
        certificate_pem: cert_pem,
        private_key_pem: String::new(), // Keychain certificates typically don't export private keys
        metadata: CaMetadata {
            subject: format_subject_name(&parsed_cert.subject),
            issuer: format_subject_name(&parsed_cert.issuer),
            serial_number: hex::encode(&parsed_cert.serial_number),
            valid_from: parsed_cert.not_before,
            valid_until: parsed_cert.not_after,
            key_algorithm: extract_key_algorithm(&parsed_cert)
                .unwrap_or_else(|_| "Unknown".to_string()),
            key_size: extract_key_size(&parsed_cert),
            created_at: SystemTime::now(),
            source,
        },
    })
}

#[cfg(target_os = "macos")]
fn der_to_pem(der_data: &[u8]) -> Result<String, TlsError> {
    use base64::Engine;
    let encoded = base64::engine::general_purpose::STANDARD.encode(der_data);
    let mut pem = String::from("-----BEGIN CERTIFICATE-----\n");
    
    // Split base64 into 64-character lines for proper PEM format
    for chunk in encoded.as_bytes().chunks(64) {
        pem.push_str(&String::from_utf8_lossy(chunk));
        pem.push('\n');
    }
    
    pem.push_str("-----END CERTIFICATE-----\n");
    Ok(pem)
}
