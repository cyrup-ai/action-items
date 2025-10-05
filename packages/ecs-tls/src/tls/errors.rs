//! TLS-specific error types for detailed error handling

/// TLS-specific error types for detailed error handling
#[derive(Debug, thiserror::Error)]
pub enum TlsError {
    #[error("Certificate parsing failed: {0}")]
    CertificateParsing(String),
    #[error("Certificate validation failed: {0}")]
    CertificateValidation(String),
    #[error("Key encryption/decryption failed: {0}")]
    KeyProtection(String),
    #[error("Certificate chain invalid: {0}")]
    ChainValidation(String),
    #[error("Peer verification failed: {0}")]
    PeerVerification(String),
    #[error("Certificate expired: {0}")]
    CertificateExpired(String),
    #[error("File operation failed: {0}")]
    FileOperation(String),
    #[error("OCSP validation failed: {0}")]
    OcspValidation(String),
    #[error("CRL validation failed: {0}")]
    CrlValidation(String),
    #[error("Network error during validation: {0}")]
    NetworkError(String),
    #[error("Keychain error: {0}")]
    KeychainError(String),
    #[error("Certificate not found: {0}")]
    CertificateNotFound(String),
    #[error("Network timeout: {0}")]
    NetworkTimeout(String),
    #[error("HTTP error: {0}")]
    HttpError(String),
    #[error("Invalid URL: {0}")]
    InvalidUrl(String),
    #[error("Certificate not yet valid: {0}")]
    CertificateNotYetValid(String),
    #[error("Unsupported platform: {0}")]
    UnsupportedPlatform(String),
    #[error("Invalid certificate: {0}")]
    InvalidCertificate(String),
    #[error("HTTP client initialization failed: {0}")]
    HttpClientInitialization(String),
    #[error("Permission error: {0}")]
    PermissionError(String),
    #[error("Permission denied: {0}")]
    PermissionDenied(String),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
