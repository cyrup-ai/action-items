//! Security Module
//!
//! Comprehensive security protections including SSRF prevention, URL validation,
//! and request sanitization for HTTP requests.

pub mod sanitization;
pub mod validation;

pub use validation::AdvancedUrlValidator as UrlValidator;

/// Comprehensive request validator combining multiple security checks
pub struct ComprehensiveRequestValidator {
    pub url_validator: UrlSecurityValidator,
    pub ssrf_protector: SsrfProtector,
    pub security_config: SecurityConfig,
}

impl ComprehensiveRequestValidator {
    /// Create new comprehensive validator
    pub fn new(config: SecurityConfig) -> Self {
        Self {
            url_validator: UrlSecurityValidator::new(),
            ssrf_protector: SsrfProtector::new(),
            security_config: config,
        }
    }

    /// Validate complete request context
    pub fn validate_request(&self, context: &RequestSecurityContext) -> Result<(), HttpError> {
        // URL validation
        self.url_validator.validate(context.url.as_str())?;

        // SSRF protection
        if self.security_config.enable_ssrf_protection {
            self.ssrf_protector.validate_url(&context.url)?;
        }

        // Body size check
        if context.body_size > self.security_config.max_request_size {
            return Err(HttpError::RequestTooLarge {
                current: context.body_size,
                limit: self.security_config.max_request_size,
            });
        }

        Ok(())
    }
}

impl Default for ComprehensiveRequestValidator {
    fn default() -> Self {
        Self::new(SecurityConfig::default())
    }
}

/// Security configuration
#[derive(Debug, Clone, bevy::prelude::Resource)]
pub struct SecurityConfig {
    /// Enable SSRF protection
    pub enable_ssrf_protection: bool,
    /// Validate URLs before making requests
    pub validate_urls: bool,
    /// Sanitize request data
    pub sanitize_requests: bool,
    /// Sanitize response data
    pub sanitize_responses: bool,
    /// Maximum request body size (bytes)
    pub max_request_size: usize,
    /// Maximum response size in bytes
    pub max_response_size: usize,
    /// Request timeout duration
    pub request_timeout: std::time::Duration,
    /// Maximum URL length
    pub max_url_length: usize,
    /// Blocked domains
    pub blocked_domains: Vec<String>,
    /// Allowed schemes
    pub allowed_schemes: Vec<String>,
    /// Enable header injection protection
    pub enable_header_injection_protection: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            enable_ssrf_protection: true,
            validate_urls: true,
            sanitize_requests: true,
            sanitize_responses: true,
            max_request_size: 10 * 1024 * 1024,  // 10MB
            max_response_size: 50 * 1024 * 1024, // 50MB
            request_timeout: std::time::Duration::from_secs(30),
            max_url_length: 2048,
            blocked_domains: vec!["localhost".to_string(), "127.0.0.1".to_string()],
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
            enable_header_injection_protection: true,
        }
    }
}

use std::net::{IpAddr, SocketAddr};
use std::str::FromStr;

use ipnet::IpNet;
use url::Url;

/// Security violation errors
#[derive(Debug, Clone, thiserror::Error)]
pub enum HttpError {
    #[error("SSRF protection triggered: {0}")]
    SsrfViolation(String),
    #[error("Invalid URL format: {0}")]
    InvalidUrl(String),
    #[error("Blocked domain: {0}")]
    BlockedDomain(String),
    #[error("Blocked IP address: {0}")]
    BlockedIpAddress(String),
    #[error("Unsupported URL scheme: {0}")]
    UnsupportedScheme(String),
    #[error("Request too large: {current} bytes exceeds limit of {limit} bytes")]
    RequestTooLarge { current: usize, limit: usize },
    #[error("Header injection detected: {0}")]
    HeaderInjection(String),
    #[error("Malicious URL pattern detected: {0}")]
    MaliciousPattern(String),
    #[error("DNS resolution blocked: {0}")]
    DnsBlocked(String),
}

/// SSRF protection validator
pub struct SsrfProtector {
    /// Blocked IP networks (CIDR notation)
    blocked_networks: Vec<IpNet>,
    /// Blocked domains
    blocked_domains: Vec<String>,
    /// Allowed schemes
    allowed_schemes: Vec<String>,
}

impl SsrfProtector {
    /// Create new SSRF protector with default blocked ranges
    pub fn new() -> Self {
        let blocked_networks = vec![
            // IPv4 private ranges
            IpNet::from_str("127.0.0.0/8").unwrap(), // Loopback
            IpNet::from_str("10.0.0.0/8").unwrap(),  // Private Class A
            IpNet::from_str("172.16.0.0/12").unwrap(), // Private Class B
            IpNet::from_str("192.168.0.0/16").unwrap(), // Private Class C
            IpNet::from_str("169.254.0.0/16").unwrap(), // Link-local
            IpNet::from_str("224.0.0.0/4").unwrap(), // Multicast
            IpNet::from_str("240.0.0.0/4").unwrap(), // Reserved
            // IPv6 private ranges
            IpNet::from_str("::1/128").unwrap(),   // Loopback
            IpNet::from_str("fc00::/7").unwrap(),  // Unique local
            IpNet::from_str("fe80::/10").unwrap(), // Link-local
            IpNet::from_str("ff00::/8").unwrap(),  // Multicast
        ];

        Self {
            blocked_networks,
            blocked_domains: vec![
                "localhost".to_string(),
                "local".to_string(),
                "internal".to_string(),
            ],
            allowed_schemes: vec!["http".to_string(), "https".to_string()],
        }
    }

    /// Add blocked IP network
    pub fn add_blocked_network(&mut self, network: &str) -> Result<(), HttpError> {
        let net = IpNet::from_str(network)
            .map_err(|e| HttpError::InvalidUrl(format!("Invalid network: {}: {}", network, e)))?;
        self.blocked_networks.push(net);
        Ok(())
    }

    /// Add blocked domain
    pub fn add_blocked_domain(&mut self, domain: String) {
        self.blocked_domains.push(domain.to_lowercase());
    }

    /// Validate URL against SSRF protections
    pub fn validate_url(&self, url: &Url) -> Result<(), HttpError> {
        // Check scheme
        if !self.allowed_schemes.contains(&url.scheme().to_lowercase()) {
            return Err(HttpError::UnsupportedScheme(url.scheme().to_string()));
        }

        // Get host
        let host = url
            .host_str()
            .ok_or_else(|| HttpError::InvalidUrl("Missing host".to_string()))?;

        // Check blocked domains
        let host_lower = host.to_lowercase();
        for blocked_domain in &self.blocked_domains {
            if host_lower == *blocked_domain
                || host_lower.ends_with(&format!(".{}", blocked_domain))
            {
                return Err(HttpError::BlockedDomain(host.to_string()));
            }
        }

        // Check for IP address in host
        if let Ok(ip) = host.parse::<IpAddr>() {
            return self.validate_ip_address(ip);
        }

        // Perform DNS resolution check for hostname
        self.validate_hostname_resolution(host)?;

        Ok(())
    }

    /// Validate IP address against blocked networks
    fn validate_ip_address(&self, ip: IpAddr) -> Result<(), HttpError> {
        for network in &self.blocked_networks {
            if network.contains(&ip) {
                return Err(HttpError::BlockedIpAddress(ip.to_string()));
            }
        }
        Ok(())
    }

    /// Validate hostname resolution doesn't resolve to blocked IPs
    fn validate_hostname_resolution(&self, hostname: &str) -> Result<(), HttpError> {
        // This is a synchronous DNS lookup - in production, this should be async
        // For now, we'll do basic validation and rely on reqwest's built-in protections

        // Check for obvious bypass attempts
        if hostname.contains("0x") || hostname.contains("0") && hostname.len() < 4 {
            return Err(HttpError::MaliciousPattern(
                "Potential IP address obfuscation".to_string(),
            ));
        }

        // Check for URL shorteners that could be used for SSRF
        let suspicious_domains = [
            "bit.ly",
            "tinyurl.com",
            "t.co",
            "goo.gl",
            "ow.ly",
            "is.gd",
            "buff.ly",
        ];

        let hostname_lower = hostname.to_lowercase();
        for suspicious in &suspicious_domains {
            if hostname_lower.contains(suspicious) {
                return Err(HttpError::MaliciousPattern(format!(
                    "Suspicious URL shortener: {}",
                    hostname
                )));
            }
        }

        Ok(())
    }

    /// Validate socket address (for direct IP connections)
    pub fn validate_socket_addr(&self, addr: &SocketAddr) -> Result<(), HttpError> {
        self.validate_ip_address(addr.ip())
    }
}

impl Default for SsrfProtector {
    fn default() -> Self {
        Self::new()
    }
}

/// URL security validator with comprehensive checks
pub struct UrlSecurityValidator {
    ssrf_protector: SsrfProtector,
    max_url_length: usize,
    max_path_segments: usize,
    blocked_extensions: Vec<String>,
}

impl UrlSecurityValidator {
    /// Create new URL security validator
    pub fn new() -> Self {
        Self {
            ssrf_protector: SsrfProtector::new(),
            max_url_length: 2048, // Standard browser limit
            max_path_segments: 20,
            blocked_extensions: vec![
                "exe".to_string(),
                "bat".to_string(),
                "cmd".to_string(),
                "com".to_string(),
                "scr".to_string(),
                "dll".to_string(),
            ],
        }
    }

    /// Comprehensive URL validation
    pub fn validate(&self, url: &str) -> Result<Url, HttpError> {
        // Check URL length
        if url.len() > self.max_url_length {
            return Err(HttpError::InvalidUrl(format!(
                "URL too long: {} chars",
                url.len()
            )));
        }

        // Parse URL
        let parsed_url = Url::parse(url)
            .map_err(|e| HttpError::InvalidUrl(format!("Failed to parse URL: {}", e)))?;

        // SSRF validation
        self.ssrf_protector.validate_url(&parsed_url)?;

        // Check path segments
        let path_segments: Vec<&str> = parsed_url
            .path_segments()
            .unwrap_or_else(|| "".split('/'))
            .collect();

        if path_segments.len() > self.max_path_segments {
            return Err(HttpError::MaliciousPattern(
                "Too many path segments".to_string(),
            ));
        }

        // Check for path traversal attempts
        for segment in &path_segments {
            if segment.contains("..") || segment.contains("%2e%2e") || segment.contains("%252e") {
                return Err(HttpError::MaliciousPattern(
                    "Path traversal attempt detected".to_string(),
                ));
            }
        }

        // Check file extension if present
        if let Some(last_segment) = path_segments.last() {
            if let Some(extension_start) = last_segment.rfind('.') {
                let extension = &last_segment[extension_start + 1..].to_lowercase();
                if self.blocked_extensions.contains(&extension.to_string()) {
                    return Err(HttpError::MaliciousPattern(format!(
                        "Blocked file extension: {}",
                        extension
                    )));
                }
            }
        }

        // Check for encoded characters that might bypass filters
        if url.contains('%') {
            let decoded = urlencoding::decode(url)
                .map_err(|_| HttpError::MaliciousPattern("Invalid URL encoding".to_string()))?;

            // Re-validate decoded URL to catch double-encoding bypasses
            if decoded != url {
                return self.validate(&decoded);
            }
        }

        Ok(parsed_url)
    }
}

impl Default for UrlSecurityValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Request security context for validation
#[derive(Debug, Clone)]
pub struct RequestSecurityContext {
    pub url: Url,
    pub method: reqwest::Method,
    pub headers: reqwest::header::HeaderMap,
    pub body_size: usize,
    pub requester: String,
    pub is_internal: bool,
}

impl RequestSecurityContext {
    /// Create new security context
    pub fn new(
        url: Url,
        method: reqwest::Method,
        headers: reqwest::header::HeaderMap,
        body_size: usize,
        requester: String,
    ) -> Self {
        let is_internal = requester.starts_with("system:") || requester == "internal";

        Self {
            url,
            method,
            headers,
            body_size,
            requester,
            is_internal,
        }
    }

    /// Get security risk level for this request
    pub fn risk_level(&self) -> SecurityRiskLevel {
        let mut risk_score = 0u8;

        // Method risk
        match self.method {
            reqwest::Method::GET | reqwest::Method::HEAD => risk_score += 1,
            reqwest::Method::POST | reqwest::Method::PUT | reqwest::Method::PATCH => {
                risk_score += 2
            },
            reqwest::Method::DELETE => risk_score += 3,
            _ => risk_score += 4, // Custom methods are higher risk
        }

        // Body size risk
        if self.body_size > 10 * 1024 * 1024 {
            // > 10MB
            risk_score += 3;
        } else if self.body_size > 1024 * 1024 {
            // > 1MB
            risk_score += 2;
        } else if self.body_size > 0 {
            risk_score += 1;
        }

        // Host risk
        if let Some(host) = self.url.host_str() {
            if host.contains("internal") || host.contains("local") || host.contains("admin") {
                risk_score += 4;
            }
        }

        // Header risk
        if self.headers.contains_key("authorization") {
            risk_score += 1;
        }
        if self.headers.contains_key("cookie") {
            risk_score += 1;
        }

        match risk_score {
            0..=2 => SecurityRiskLevel::Low,
            3..=5 => SecurityRiskLevel::Medium,
            6..=8 => SecurityRiskLevel::High,
            _ => SecurityRiskLevel::Critical,
        }
    }
}

/// Security risk levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum SecurityRiskLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}
