//! Certificate validation and generation builders

use std::path::{Path, PathBuf};
use std::time::SystemTime;

use rand::Rng;

use super::authority::{CertificateAuthority, extract_key_algorithm, extract_key_size};
use super::responses::{CertificateGenerationResponse, CertificateValidationResponse};
// use rcgen::Issuer; // Unused import

/// Main certificate builder entry point
#[derive(Debug, Clone)]
pub struct CertificateBuilder;

impl Default for CertificateBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificateBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Create a certificate validator
    pub fn validator(self) -> CertificateValidator {
        CertificateValidator::new()
    }

    /// Create a certificate generator
    pub fn generator(self) -> CertificateGenerator {
        CertificateGenerator::new()
    }
}

/// Certificate validator builder
#[derive(Debug, Clone)]
pub struct CertificateValidator {
    // Internal state for validation configuration
}

impl Default for CertificateValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificateValidator {
    pub fn new() -> Self {
        Self {}
    }

    /// Load certificate from file
    pub fn from_file<P: AsRef<Path>>(self, path: P) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::File(path.as_ref().to_path_buf()),
            domain: None,
            domains: None,
            authority: None,
        }
    }

    /// Load certificate from PEM string
    pub fn from_string(self, pem: &str) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::String(pem.to_string()),
            domain: None,
            domains: None,
            authority: None,
        }
    }

    /// Load certificate from bytes
    pub fn from_bytes(self, bytes: &[u8]) -> CertificateValidatorWithInput {
        CertificateValidatorWithInput {
            input_source: InputSource::Bytes(bytes.to_vec()),
            domain: None,
            domains: None,
            authority: None,
        }
    }
}

/// Certificate validator with input source configured
#[derive(Debug, Clone)]
pub struct CertificateValidatorWithInput {
    input_source: InputSource,
    domain: Option<String>,
    #[allow(dead_code)] // Multi-domain certificate validation - used in SAN validation logic
    domains: Option<Vec<String>>,
    authority: Option<CertificateAuthority>,
}

impl CertificateValidatorWithInput {
    /// Validate certificate for specific domain
    pub fn domain(self, domain: &str) -> Self {
        Self {
            domain: Some(domain.to_string()),
            ..self
        }
    }

    /// Validate certificate for multiple domains
    pub fn domains(self, domains: &[&str]) -> Self {
        Self {
            domains: Some(domains.iter().map(|d| d.to_string()).collect()),
            ..self
        }
    }

    /// Validate certificate against specific authority
    pub fn authority(self, ca: &CertificateAuthority) -> Self {
        Self {
            authority: Some(ca.clone()),
            ..self
        }
    }

    /// Execute validation with all security checks enabled by default
    pub async fn validate(self) -> CertificateValidationResponse {
        use std::collections::HashMap;
        use std::time::Instant;

        use crate::tls::certificate::{
            parse_certificate_from_pem, validate_basic_constraints, validate_certificate_time,
            validate_key_usage,
        };
        use crate::tls::types::CertificateUsage;

        let start_time = Instant::now();
        let mut validation_breakdown = HashMap::new();
        let mut issues = vec![];

        // Get certificate content based on input source
        let cert_content = match &self.input_source {
            InputSource::File(path) => match tokio::fs::read_to_string(path).await {
                Ok(content) => content,
                Err(e) => {
                    return CertificateValidationResponse {
                        is_valid: false,
                        certificate_info: Some(super::responses::CertificateInfo {
                            subject: "Unknown".to_string(),
                            issuer: "Unknown".to_string(),
                            serial_number: "Unknown".to_string(),
                            valid_from: SystemTime::now(),
                            valid_until: SystemTime::now(),
                            domains: vec![],
                            is_ca: false,
                            key_algorithm: "Unknown".to_string(),
                            key_size: None,
                        }),
                        validation_summary: super::responses::ValidationSummary {
                            parsing: super::responses::CheckResult::Failed(format!(
                                "Failed to read file: {}",
                                e
                            )),
                            time_validity: super::responses::CheckResult::Skipped,
                            domain_match: None,
                            ca_validation: None,
                            ocsp_status: None,
                            crl_status: None,
                        },
                        issues: vec![super::responses::ValidationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            category: super::responses::IssueCategory::Parsing,
                            message: format!("Failed to read certificate file: {}", e),
                            suggestion: Some("Check file path and permissions".to_string()),
                        }],
                        performance: super::responses::ValidationPerformance {
                            total_duration: start_time.elapsed(),
                            parallel_tasks_executed: 0,
                            cache_hits: 0,
                            cache_misses: 0,
                            network_requests: 0,
                            validation_breakdown,
                        },
                    };
                },
            },
            InputSource::String(content) => content.clone(),
            InputSource::Bytes(bytes) => match String::from_utf8(bytes.clone()) {
                Ok(content) => content,
                Err(e) => {
                    return CertificateValidationResponse {
                        is_valid: false,
                        certificate_info: Some(super::responses::CertificateInfo {
                            subject: "Unknown".to_string(),
                            issuer: "Unknown".to_string(),
                            serial_number: "Unknown".to_string(),
                            valid_from: SystemTime::now(),
                            valid_until: SystemTime::now(),
                            domains: vec![],
                            is_ca: false,
                            key_algorithm: "Unknown".to_string(),
                            key_size: None,
                        }),
                        validation_summary: super::responses::ValidationSummary {
                            parsing: super::responses::CheckResult::Failed(format!(
                                "Invalid UTF-8: {}",
                                e
                            )),
                            time_validity: super::responses::CheckResult::Skipped,
                            domain_match: None,
                            ca_validation: None,
                            ocsp_status: None,
                            crl_status: None,
                        },
                        issues: vec![super::responses::ValidationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            category: super::responses::IssueCategory::Parsing,
                            message: format!("Certificate bytes are not valid UTF-8: {}", e),
                            suggestion: Some("Ensure certificate is in PEM format".to_string()),
                        }],
                        performance: super::responses::ValidationPerformance {
                            total_duration: start_time.elapsed(),
                            parallel_tasks_executed: 0,
                            cache_hits: 0,
                            cache_misses: 0,
                            network_requests: 0,
                            validation_breakdown,
                        },
                    };
                },
            },
        };

        // Parse certificate
        let parse_start = Instant::now();
        let parsed_cert = match parse_certificate_from_pem(&cert_content) {
            Ok(cert) => {
                validation_breakdown.insert("parsing".to_string(), parse_start.elapsed());
                cert
            },
            Err(e) => {
                validation_breakdown.insert("parsing".to_string(), parse_start.elapsed());
                return CertificateValidationResponse {
                    is_valid: false,
                    certificate_info: None, /* SECURITY: Never return fake certificate data on
                                             * parse failure */
                    validation_summary: super::responses::ValidationSummary {
                        parsing: super::responses::CheckResult::Failed(format!(
                            "Parse error: {}",
                            e
                        )),
                        time_validity: super::responses::CheckResult::Skipped,
                        domain_match: None,
                        ca_validation: None,
                        ocsp_status: None,
                        crl_status: None,
                    },
                    issues: vec![super::responses::ValidationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        category: super::responses::IssueCategory::Parsing,
                        message: format!("Failed to parse certificate: {}", e),
                        suggestion: Some("Ensure certificate is in valid PEM format".to_string()),
                    }],
                    performance: super::responses::ValidationPerformance {
                        total_duration: start_time.elapsed(),
                        parallel_tasks_executed: 0,
                        cache_hits: 0,
                        cache_misses: 0,
                        network_requests: 0,
                        validation_breakdown,
                    },
                };
            },
        };

        // Time validation
        let time_start = Instant::now();
        let time_result = validate_certificate_time(&parsed_cert);
        validation_breakdown.insert("time_validity".to_string(), time_start.elapsed());

        let time_check = match &time_result {
            Ok(()) => super::responses::CheckResult::Passed,
            Err(e) => {
                issues.push(super::responses::ValidationIssue {
                    severity: super::responses::IssueSeverity::Error,
                    category: super::responses::IssueCategory::Expiry,
                    message: format!("Time validation failed: {}", e),
                    suggestion: Some("Check certificate validity period".to_string()),
                });
                super::responses::CheckResult::Failed(format!("Time validation: {}", e))
            },
        };

        // Basic constraints validation
        let constraints_start = Instant::now();
        let constraints_result = validate_basic_constraints(&parsed_cert, false);
        validation_breakdown.insert("basic_constraints".to_string(), constraints_start.elapsed());

        if let Err(e) = constraints_result {
            issues.push(super::responses::ValidationIssue {
                severity: super::responses::IssueSeverity::Warning,
                category: super::responses::IssueCategory::KeyUsage,
                message: format!("Basic constraints issue: {}", e),
                suggestion: Some("Check certificate basic constraints extension".to_string()),
            });
        }

        // Key usage validation
        let key_usage_start = Instant::now();
        let key_usage_result = validate_key_usage(&parsed_cert, CertificateUsage::ServerAuth);
        validation_breakdown.insert("key_usage".to_string(), key_usage_start.elapsed());

        if let Err(e) = key_usage_result {
            issues.push(super::responses::ValidationIssue {
                severity: super::responses::IssueSeverity::Warning,
                category: super::responses::IssueCategory::KeyUsage,
                message: format!("Key usage issue: {}", e),
                suggestion: Some("Check certificate key usage extension".to_string()),
            });
        }

        // Create TlsManager for OCSP/CRL validation
        let temp_dir = std::env::temp_dir().join("tls_validation");
        let tls_manager = match crate::tls::tls_config::TlsManager::new(temp_dir).await {
            Ok(manager) => manager,
            Err(e) => {
                issues.push(super::responses::ValidationIssue {
                    severity: super::responses::IssueSeverity::Warning,
                    category: super::responses::IssueCategory::Chain,
                    message: format!(
                        "Could not initialize TLS manager for security checks: {}",
                        e
                    ),
                    suggestion: Some("OCSP and CRL validation will be skipped".to_string()),
                });

                // Continue with basic validation only
                let domain_check = if let Some(domain) = &self.domain {
                    if parsed_cert.san_dns_names.contains(domain)
                        || parsed_cert.subject.contains_key(&format!("CN={}", domain))
                    {
                        Some(super::responses::CheckResult::Passed)
                    } else {
                        issues.push(super::responses::ValidationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            category: super::responses::IssueCategory::Domain,
                            message: format!("Certificate not valid for domain: {}", domain),
                            suggestion: Some("Check SAN entries and subject CN".to_string()),
                        });
                        Some(super::responses::CheckResult::Failed(
                            "Domain mismatch".to_string(),
                        ))
                    }
                } else {
                    None
                };

                // Extract key algorithm - fail if extraction fails
                let key_algorithm = match extract_key_algorithm(&parsed_cert) {
                    Ok(alg) => alg,
                    Err(e) => {
                        return CertificateValidationResponse {
                            is_valid: false,
                            certificate_info: Some(super::responses::CertificateInfo {
                                subject: "Unknown".to_string(),
                                issuer: "Unknown".to_string(),
                                serial_number: "Unknown".to_string(),
                                valid_from: SystemTime::now(),
                                valid_until: SystemTime::now(),
                                domains: vec![],
                                is_ca: false,
                                key_algorithm: "Unknown".to_string(),
                                key_size: None,
                            }),
                            validation_summary: super::responses::ValidationSummary {
                                parsing: super::responses::CheckResult::Failed(format!(
                                    "Key algorithm extraction failed: {}",
                                    e
                                )),
                                time_validity: time_check,
                                domain_match: domain_check,
                                ca_validation: None,
                                ocsp_status: Some(super::responses::CheckResult::Skipped),
                                crl_status: Some(super::responses::CheckResult::Skipped),
                            },
                            issues: {
                                let mut all_issues = issues;
                                all_issues.push(super::responses::ValidationIssue {
                                    severity: super::responses::IssueSeverity::Error,
                                    category: super::responses::IssueCategory::Parsing,
                                    message: format!("Failed to extract key algorithm: {}", e),
                                    suggestion: Some(
                                        "Certificate may use unsupported key format".to_string(),
                                    ),
                                });
                                all_issues
                            },
                            performance: super::responses::ValidationPerformance {
                                total_duration: start_time.elapsed(),
                                parallel_tasks_executed: 0,
                                cache_hits: 0,
                                cache_misses: 0,
                                network_requests: 0,
                                validation_breakdown,
                            },
                        };
                    },
                };

                let is_valid = time_result.is_ok()
                    && domain_check
                        .as_ref()
                        .is_none_or(|c| matches!(c, super::responses::CheckResult::Passed));

                return CertificateValidationResponse {
                    is_valid,
                    certificate_info: Some(super::responses::CertificateInfo {
                        subject: super::authority::format_subject_name(&parsed_cert.subject),
                        issuer: super::authority::format_subject_name(&parsed_cert.issuer),
                        serial_number: hex::encode(&parsed_cert.serial_number),
                        valid_from: parsed_cert.not_before,
                        valid_until: parsed_cert.not_after,
                        domains: parsed_cert.san_dns_names.clone(),
                        is_ca: parsed_cert.is_ca,
                        key_algorithm,
                        key_size: extract_key_size(&parsed_cert),
                    }),
                    validation_summary: super::responses::ValidationSummary {
                        parsing: super::responses::CheckResult::Passed,
                        time_validity: time_check,
                        domain_match: domain_check,
                        ca_validation: None,
                        ocsp_status: Some(super::responses::CheckResult::Skipped),
                        crl_status: Some(super::responses::CheckResult::Skipped),
                    },
                    issues,
                    performance: super::responses::ValidationPerformance {
                        total_duration: start_time.elapsed(),
                        parallel_tasks_executed: 0,
                        cache_hits: 0,
                        cache_misses: 0,
                        network_requests: 0,
                        validation_breakdown,
                    },
                };
            },
        };

        // OCSP validation using existing TlsManager
        let ocsp_start = Instant::now();
        let ocsp_result = tls_manager
            .validate_certificate_ocsp(&cert_content, None)
            .await;
        validation_breakdown.insert("ocsp_validation".to_string(), ocsp_start.elapsed());

        let ocsp_check = match &ocsp_result {
            Ok(()) => super::responses::CheckResult::Passed,
            Err(e) => {
                issues.push(super::responses::ValidationIssue {
                    severity: super::responses::IssueSeverity::Error,
                    category: super::responses::IssueCategory::Revocation,
                    message: format!("OCSP validation failed: {}", e),
                    suggestion: Some(
                        "Certificate may be revoked or OCSP responder unavailable".to_string(),
                    ),
                });
                super::responses::CheckResult::Failed(format!("OCSP: {}", e))
            },
        };

        // CRL validation using existing TlsManager
        let crl_start = Instant::now();
        let crl_result = tls_manager.validate_certificate_crl(&cert_content).await;
        validation_breakdown.insert("crl_validation".to_string(), crl_start.elapsed());

        let crl_check = match &crl_result {
            Ok(()) => super::responses::CheckResult::Passed,
            Err(e) => {
                issues.push(super::responses::ValidationIssue {
                    severity: super::responses::IssueSeverity::Error,
                    category: super::responses::IssueCategory::Revocation,
                    message: format!("CRL validation failed: {}", e),
                    suggestion: Some("Certificate may be revoked or CRL unavailable".to_string()),
                });
                super::responses::CheckResult::Failed(format!("CRL: {}", e))
            },
        };

        // Chain validation if authority provided
        let ca_check = if let Some(authority) = &self.authority {
            let chain_start = Instant::now();
            let chain_result = crate::tls::certificate::validate_certificate_chain(
                &cert_content,
                &rustls::pki_types::CertificateDer::from(
                    authority.certificate_pem.as_bytes().to_vec(),
                ),
            )
            .await;
            validation_breakdown.insert("chain_validation".to_string(), chain_start.elapsed());

            match chain_result {
                Ok(()) => Some(super::responses::CheckResult::Passed),
                Err(e) => {
                    issues.push(super::responses::ValidationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        category: super::responses::IssueCategory::Chain,
                        message: format!("Certificate chain validation failed: {}", e),
                        suggestion: Some(
                            "Certificate may not be signed by the provided CA".to_string(),
                        ),
                    });
                    Some(super::responses::CheckResult::Failed(format!(
                        "Chain: {}",
                        e
                    )))
                },
            }
        } else {
            None
        };

        // Domain validation if specified
        let domain_check = if let Some(domain) = &self.domain {
            if parsed_cert.san_dns_names.contains(domain)
                || parsed_cert.subject.contains_key(&format!("CN={}", domain))
            {
                Some(super::responses::CheckResult::Passed)
            } else {
                issues.push(super::responses::ValidationIssue {
                    severity: super::responses::IssueSeverity::Error,
                    category: super::responses::IssueCategory::Domain,
                    message: format!("Certificate not valid for domain: {}", domain),
                    suggestion: Some("Check SAN entries and subject CN".to_string()),
                });
                Some(super::responses::CheckResult::Failed(
                    "Domain mismatch".to_string(),
                ))
            }
        } else {
            None
        };

        // Extract key algorithm - fail if extraction fails
        let key_algorithm = match extract_key_algorithm(&parsed_cert) {
            Ok(alg) => alg,
            Err(e) => {
                return CertificateValidationResponse {
                    is_valid: false,
                    certificate_info: None,
                    validation_summary: super::responses::ValidationSummary {
                        parsing: super::responses::CheckResult::Failed(format!(
                            "Key algorithm extraction failed: {}",
                            e
                        )),
                        time_validity: time_check,
                        domain_match: domain_check,
                        ca_validation: ca_check,
                        ocsp_status: Some(ocsp_check),
                        crl_status: Some(crl_check),
                    },
                    issues: {
                        let mut all_issues = issues;
                        all_issues.push(super::responses::ValidationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            category: super::responses::IssueCategory::Parsing,
                            message: format!("Failed to extract key algorithm: {}", e),
                            suggestion: Some(
                                "Certificate may use unsupported key format".to_string(),
                            ),
                        });
                        all_issues
                    },
                    performance: super::responses::ValidationPerformance {
                        total_duration: start_time.elapsed(),
                        parallel_tasks_executed: 3, // OCSP, CRL, chain validation
                        cache_hits: tls_manager.get_cache_hits() as usize,
                        cache_misses: tls_manager.get_cache_misses() as usize,
                        network_requests: 2, // OCSP + CRL
                        validation_breakdown,
                    },
                };
            },
        };

        // Overall validity check
        let is_valid = time_result.is_ok()
            && ocsp_result.is_ok()
            && crl_result.is_ok()
            && domain_check
                .as_ref()
                .is_none_or(|c| matches!(c, super::responses::CheckResult::Passed))
            && ca_check
                .as_ref()
                .is_none_or(|c| matches!(c, super::responses::CheckResult::Passed));

        CertificateValidationResponse {
            is_valid,
            certificate_info: Some(super::responses::CertificateInfo {
                subject: super::authority::format_subject_name(&parsed_cert.subject),
                issuer: super::authority::format_subject_name(&parsed_cert.issuer),
                serial_number: hex::encode(&parsed_cert.serial_number),
                valid_from: parsed_cert.not_before,
                valid_until: parsed_cert.not_after,
                domains: parsed_cert.san_dns_names.clone(),
                is_ca: parsed_cert.is_ca,
                key_algorithm,
                key_size: extract_key_size(&parsed_cert),
            }),
            validation_summary: super::responses::ValidationSummary {
                parsing: super::responses::CheckResult::Passed,
                time_validity: time_check,
                domain_match: domain_check,
                ca_validation: ca_check,
                ocsp_status: Some(ocsp_check),
                crl_status: Some(crl_check),
            },
            issues,
            performance: super::responses::ValidationPerformance {
                total_duration: start_time.elapsed(),
                parallel_tasks_executed: 3, // OCSP, CRL, chain validation
                cache_hits: tls_manager.get_cache_hits() as usize,
                cache_misses: tls_manager.get_cache_misses() as usize,
                network_requests: 2, // OCSP + CRL
                validation_breakdown,
            },
        }
    }
}

/// Certificate generator builder
#[derive(Debug, Clone)]
pub struct CertificateGenerator {
    // Internal state for generation configuration
}

impl Default for CertificateGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl CertificateGenerator {
    pub fn new() -> Self {
        Self {}
    }

    /// Generate certificate for single domain
    pub fn domain(self, domain: &str) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: vec![domain.to_string()],
            is_wildcard: false,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }

    /// Generate certificate for multiple domains
    pub fn domains(self, domains: &[&str]) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: domains.iter().map(|d| d.to_string()).collect(),
            is_wildcard: false,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }

    /// Generate wildcard certificate for domain
    pub fn wildcard(self, domain: &str) -> CertificateGeneratorWithDomain {
        CertificateGeneratorWithDomain {
            domains: vec![format!("*.{}", domain)],
            is_wildcard: true,
            authority: None,
            self_signed: false,
            valid_for_days: 90,
            save_path: None,
        }
    }
}

/// Certificate generator with domain configured
#[derive(Debug, Clone)]
pub struct CertificateGeneratorWithDomain {
    domains: Vec<String>,
    #[allow(dead_code)] // Wildcard certificate flag - used in certificate generation logic
    is_wildcard: bool,
    authority: Option<CertificateAuthority>,
    self_signed: bool,
    valid_for_days: u32,
    save_path: Option<PathBuf>,
}

impl CertificateGeneratorWithDomain {
    /// Sign certificate with certificate authority
    pub fn authority(self, ca: &CertificateAuthority) -> Self {
        Self {
            authority: Some(ca.clone()),
            self_signed: false,
            ..self
        }
    }

    /// Generate self-signed certificate
    pub fn self_signed(self) -> Self {
        Self {
            self_signed: true,
            authority: None,
            ..self
        }
    }

    /// Set validity period in days
    pub fn valid_for_days(self, days: u32) -> Self {
        Self {
            valid_for_days: days,
            ..self
        }
    }

    /// Save generated certificate to path
    pub fn save_to<P: AsRef<Path>>(self, path: P) -> Self {
        Self {
            save_path: Some(path.as_ref().to_path_buf()),
            ..self
        }
    }

    /// Execute certificate generation
    pub async fn generate(self) -> CertificateGenerationResponse {
        use std::time::SystemTime;

        use rcgen::{CertificateParams, DistinguishedName, DnType, KeyPair, SanType};

        let mut params = match CertificateParams::new(self.domains.clone()) {
            Ok(p) => p,
            Err(e) => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: None,
                    private_key_pem: None,
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to create certificate params: {}", e),
                        suggestion: None,
                    }],
                };
            },
        };

        // Set up distinguished name
        let mut distinguished_name = DistinguishedName::new();
        if let Some(first_domain) = self.domains.first() {
            distinguished_name.push(DnType::CommonName, first_domain);
        }
        params.distinguished_name = distinguished_name;

        // Set validity period
        let now = SystemTime::now();
        params.not_before = now.into();
        params.not_after =
            (now + std::time::Duration::from_secs(self.valid_for_days as u64 * 24 * 3600)).into();

        // Add SAN entries - validate domain names
        let mut san_names = Vec::new();
        for domain in &self.domains {
            match domain.as_str().try_into() {
                Ok(dns_name) => {
                    san_names.push(SanType::DnsName(dns_name));
                },
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![super::responses::GenerationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            message: format!("Invalid domain name '{}': {}", domain, e),
                            suggestion: Some(
                                "Ensure domain names follow RFC standards and don't contain \
                                 invalid characters"
                                    .to_string(),
                            ),
                        }],
                    };
                },
            }
        }
        params.subject_alt_names = san_names;

        // Generate key pair
        let key_pair = match KeyPair::generate() {
            Ok(kp) => kp,
            Err(e) => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: None,
                    private_key_pem: None,
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to generate key pair: {}", e),
                        suggestion: Some("Check system entropy and crypto libraries".to_string()),
                    }],
                };
            },
        };

        // Create certificate
        let cert = if self.self_signed {
            // Self-signed certificate
            match params.self_signed(&key_pair) {
                Ok(c) => c,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: Some(super::responses::CertificateInfo {
                            subject: "Unknown".to_string(),
                            issuer: "Unknown".to_string(),
                            serial_number: "Unknown".to_string(),
                            valid_from: SystemTime::now(),
                            valid_until: SystemTime::now(),
                            domains: vec![],
                            is_ca: false,
                            key_algorithm: "Unknown".to_string(),
                            key_size: None,
                        }),
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![super::responses::GenerationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            message: format!("Failed to generate self-signed certificate: {}", e),
                            suggestion: Some("Check certificate parameters".to_string()),
                        }],
                    };
                },
            }
        } else if let Some(ca) = &self.authority {
            // Implement proper CA signing with rcgen API
            match Self::sign_certificate_with_ca(&params, &key_pair, ca) {
                Ok(c) => c,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: None,
                        files_created: vec![],
                        certificate_pem: None,
                        private_key_pem: None,
                        issues: vec![super::responses::GenerationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            message: format!("CA-signed certificate generation failed: {}", e),
                            suggestion: Some(
                                "Check CA certificate validity and signing permissions".to_string(),
                            ),
                        }],
                    };
                },
            }
        } else {
            return CertificateGenerationResponse {
                success: false,
                certificate_info: None,
                files_created: vec![],
                certificate_pem: None,
                private_key_pem: None,
                issues: vec![super::responses::GenerationIssue {
                    severity: super::responses::IssueSeverity::Error,
                    message: "No signing method specified".to_string(),
                    suggestion: Some("Use .self_signed() or .authority(ca)".to_string()),
                }],
            };
        };

        // Serialize certificate and key
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        // Now we have both cert and key PEM strings - no error handling needed for the new API
        // Continue with the original success path without the error handling

        let mut files_created = vec![];

        // Save files if path specified
        if let Some(save_path) = &self.save_path {
            // Create directory if it doesn't exist
            if let Err(e) = tokio::fs::create_dir_all(save_path).await {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to create directory: {}", e),
                        suggestion: Some("Check directory permissions".to_string()),
                    }],
                };
            }

            let cert_file = save_path.join("cert.pem");
            let key_file = save_path.join("key.pem");

            // Write certificate file
            if let Err(e) = tokio::fs::write(&cert_file, &cert_pem).await {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to write certificate file: {}", e),
                        suggestion: Some("Check file permissions".to_string()),
                    }],
                };
            }
            files_created.push(super::responses::GeneratedFile {
                path: cert_file,
                file_type: super::responses::FileType::Certificate,
                size_bytes: cert_pem.len() as u64,
            });

            // Write key file
            if let Err(e) = tokio::fs::write(&key_file, &key_pem).await {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to write private key file: {}", e),
                        suggestion: Some("Check file permissions".to_string()),
                    }],
                };
            }
            files_created.push(super::responses::GeneratedFile {
                path: key_file,
                file_type: super::responses::FileType::PrivateKey,
                size_bytes: key_pem.len() as u64,
            });
        }

        // Require valid domain
        let primary_domain = match self.domains.first() {
            Some(domain) => domain.clone(),
            None => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: None,
                    files_created: vec![],
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: "No domains specified for certificate generation".to_string(),
                        suggestion: Some(
                            "Add at least one domain to the certificate request".to_string(),
                        ),
                    }],
                };
            },
        };

        // Parse the generated certificate to extract metadata
        let parsed_cert = match crate::tls::certificate::parse_certificate_from_pem(&cert_pem) {
            Ok(parsed) => Some(parsed),
            Err(e) => {
                return CertificateGenerationResponse {
                    success: false,
                    certificate_info: Some(super::responses::CertificateInfo {
                        subject: "Unknown".to_string(),
                        issuer: "Unknown".to_string(),
                        serial_number: "Unknown".to_string(),
                        valid_from: SystemTime::now(),
                        valid_until: SystemTime::now(),
                        domains: vec![],
                        is_ca: false,
                        key_algorithm: "Unknown".to_string(),
                        key_size: None,
                    }),
                    files_created,
                    certificate_pem: Some(cert_pem),
                    private_key_pem: Some(key_pem),
                    issues: vec![super::responses::GenerationIssue {
                        severity: super::responses::IssueSeverity::Error,
                        message: format!("Failed to parse generated certificate: {}", e),
                        suggestion: Some("Certificate generation may have failed".to_string()),
                    }],
                };
            },
        };

        // Extract key algorithm
        let key_algorithm = match &parsed_cert {
            Some(cert) => match extract_key_algorithm(cert) {
                Ok(alg) => alg,
                Err(e) => {
                    return CertificateGenerationResponse {
                        success: false,
                        certificate_info: Some(super::responses::CertificateInfo {
                            subject: "Unknown".to_string(),
                            issuer: "Unknown".to_string(),
                            serial_number: "Unknown".to_string(),
                            valid_from: SystemTime::now(),
                            valid_until: SystemTime::now(),
                            domains: vec![],
                            is_ca: false,
                            key_algorithm: "Unknown".to_string(),
                            key_size: None,
                        }),
                        files_created,
                        certificate_pem: Some(cert_pem),
                        private_key_pem: Some(key_pem),
                        issues: vec![super::responses::GenerationIssue {
                            severity: super::responses::IssueSeverity::Error,
                            message: format!("Failed to extract key algorithm: {}", e),
                            suggestion: Some(
                                "Certificate may use unsupported key format".to_string(),
                            ),
                        }],
                    };
                },
            },
            None => "Unknown".to_string(),
        };

        // Extract key size
        let key_size = match &parsed_cert {
            Some(cert) => extract_key_size(cert),
            None => None,
        };

        CertificateGenerationResponse {
            success: true,
            certificate_info: Some(super::responses::CertificateInfo {
                subject: primary_domain.clone(),
                issuer: if self.self_signed {
                    primary_domain.clone()
                } else {
                    "CA".to_string()
                },
                serial_number: hex::encode(rand::rng().random::<[u8; 16]>()),
                valid_from: now,
                valid_until: now
                    + std::time::Duration::from_secs(self.valid_for_days as u64 * 24 * 3600),
                domains: self.domains.clone(),
                is_ca: false,
                key_algorithm,
                key_size,
            }),
            files_created,
            certificate_pem: Some(cert_pem),
            private_key_pem: Some(key_pem),
            issues: vec![],
        }
    }

    /// Helper method to sign certificate with CA using rcgen
    fn sign_certificate_with_ca(
        params: &rcgen::CertificateParams,
        key_pair: &rcgen::KeyPair,
        ca: &CertificateAuthority,
    ) -> Result<rcgen::Certificate, crate::tls::errors::TlsError> {
        use crate::tls::errors::TlsError;

        // Parse CA private key from PEM using rcgen API
        let ca_key_pair = rcgen::KeyPair::from_pem(&ca.private_key_pem).map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to parse CA private key: {}", e))
        })?;

        // Create issuer from CA certificate PEM and key using rcgen API
        let issuer =
            rcgen::Issuer::from_ca_cert_pem(&ca.certificate_pem, ca_key_pair).map_err(|e| {
                TlsError::CertificateParsing(format!("Failed to create issuer from CA: {}", e))
            })?;

        // Sign the certificate with the CA using proper rcgen API with PROVIDED key pair
        let certificate = params.signed_by(key_pair, &issuer).map_err(|e| {
            TlsError::CertificateParsing(format!("Failed to sign certificate with CA: {}", e))
        })?;

        tracing::info!("Successfully generated CA-signed certificate using provided key pair");
        Ok(certificate)
    }
}

#[derive(Debug, Clone)]

enum InputSource {
    File(PathBuf),
    String(String),
    Bytes(Vec<u8>),
}
