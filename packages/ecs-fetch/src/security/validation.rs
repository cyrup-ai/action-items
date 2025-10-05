//! URL Validation and Security Checks
//!
//! Advanced URL validation with comprehensive security pattern detection
//! and malicious request filtering.

use std::collections::HashSet;

use reqwest::header::HeaderMap;
use url::Url;

use super::{HttpError, RequestSecurityContext, SecurityRiskLevel};

/// Advanced URL validator with pattern recognition
#[derive(bevy::prelude::Resource)]
pub struct AdvancedUrlValidator {
    /// Blocked URL patterns (regex-like)
    blocked_patterns: Vec<String>,
    /// Suspicious file extensions
    suspicious_extensions: HashSet<String>,
    /// Blocked query parameters
    blocked_query_params: HashSet<String>,
    /// Maximum query string length
    max_query_length: usize,
    /// Known malicious domains
    malicious_domains: HashSet<String>,
}

impl AdvancedUrlValidator {
    /// Create new advanced validator
    pub fn new() -> Self {
        let mut suspicious_extensions = HashSet::new();
        suspicious_extensions.extend(
            [
                "exe", "bat", "cmd", "com", "scr", "dll", "msi", "vbs", "js", "jar", "app", "deb",
                "rpm", "dmg", "pkg",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut blocked_query_params = HashSet::new();
        blocked_query_params.extend(
            [
                "eval", "exec", "system", "shell", "cmd", "command", "include", "require",
                "import", "load", "file",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut malicious_domains = HashSet::new();
        malicious_domains.extend(
            [
                "malware.com",
                "phishing.net",
                "suspicious.org",
                "localhost",
                "local",
                "internal.local",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        Self {
            blocked_patterns: vec![
                r"javascript:".to_string(),
                r"data:text/html".to_string(),
                r"file://".to_string(),
                r"ftp://".to_string(),
                r"\.(exe|bat|cmd|scr)($|\?|#)".to_string(),
                r"[\x00-\x1f]".to_string(), // Control characters
            ],
            suspicious_extensions,
            blocked_query_params,
            max_query_length: 2048,
            malicious_domains,
        }
    }

    /// Validate URL against advanced security patterns
    pub fn validate_advanced(&self, url: &Url) -> Result<(), HttpError> {
        let url_string = url.as_str();

        // Check blocked patterns
        for pattern in &self.blocked_patterns {
            if self.matches_pattern(url_string, pattern) {
                return Err(HttpError::MaliciousPattern(format!(
                    "Blocked pattern: {}",
                    pattern
                )));
            }
        }

        // Validate domain
        self.validate_domain(url)?;

        // Validate path
        self.validate_path(url)?;

        // Validate query parameters
        self.validate_query_params(url)?;

        // Validate fragment
        self.validate_fragment(url)?;

        Ok(())
    }

    /// Simple pattern matching (could be enhanced with regex)
    fn matches_pattern(&self, text: &str, pattern: &str) -> bool {
        // Simple implementation - in production, use a proper regex engine
        if pattern.starts_with("r\"") && pattern.ends_with('"') {
            // Simple regex-like patterns
            let pattern = &pattern[2..pattern.len() - 1];
            if pattern == "javascript:" {
                return text.to_lowercase().contains("javascript:");
            }
            if pattern == "data:text/html" {
                return text.to_lowercase().contains("data:text/html");
            }
            if pattern == "file://" {
                return text.to_lowercase().starts_with("file://");
            }
            if pattern == "ftp://" {
                return text.to_lowercase().starts_with("ftp://");
            }
        }

        text.contains(pattern)
    }

    /// Validate domain against known malicious domains
    fn validate_domain(&self, url: &Url) -> Result<(), HttpError> {
        if let Some(domain) = url.domain() {
            let domain_lower = domain.to_lowercase();

            // Check exact matches
            if self.malicious_domains.contains(&domain_lower) {
                return Err(HttpError::BlockedDomain(domain.to_string()));
            }

            // Check for suspicious subdomains
            for malicious in &self.malicious_domains {
                if domain_lower.ends_with(&format!(".{}", malicious)) {
                    return Err(HttpError::BlockedDomain(domain.to_string()));
                }
            }

            // Check for suspicious patterns in domain
            if domain_lower.contains("xn--") {
                // Internationalized domain - requires additional scrutiny
                return Err(HttpError::MaliciousPattern(
                    "Internationalized domain detected".to_string(),
                ));
            }

            if domain_lower.chars().filter(|c| *c == '.').count() > 5 {
                return Err(HttpError::MaliciousPattern(
                    "Excessive subdomain levels".to_string(),
                ));
            }
        }

        Ok(())
    }

    /// Validate URL path for suspicious patterns
    fn validate_path(&self, url: &Url) -> Result<(), HttpError> {
        let path = url.path();

        // Check for suspicious file extensions
        if let Some(extension_start) = path.rfind('.') {
            let extension = path[extension_start + 1..].to_lowercase();
            if self.suspicious_extensions.contains(&extension) {
                return Err(HttpError::MaliciousPattern(format!(
                    "Suspicious extension: {}",
                    extension
                )));
            }
        }

        // Check for path traversal patterns
        if path.contains("../")
            || path.contains("..\\")
            || path.contains("%2e%2e%2f")
            || path.contains("%2e%2e%5c")
        {
            return Err(HttpError::MaliciousPattern(
                "Path traversal detected".to_string(),
            ));
        }

        // Check for null bytes and control characters
        if path.chars().any(|c| c.is_control()) {
            return Err(HttpError::MaliciousPattern(
                "Control characters in path".to_string(),
            ));
        }

        // Check path length
        if path.len() > 1024 {
            return Err(HttpError::MaliciousPattern("Path too long".to_string()));
        }

        Ok(())
    }

    /// Validate query parameters
    fn validate_query_params(&self, url: &Url) -> Result<(), HttpError> {
        if let Some(query) = url.query() {
            // Check query length
            if query.len() > self.max_query_length {
                return Err(HttpError::MaliciousPattern(
                    "Query string too long".to_string(),
                ));
            }

            // Parse and validate individual parameters
            for (key, value) in url.query_pairs() {
                let key_lower = key.to_lowercase();

                // Check for blocked parameter names
                if self.blocked_query_params.contains(&key_lower) {
                    return Err(HttpError::MaliciousPattern(format!(
                        "Blocked parameter: {}",
                        key
                    )));
                }

                // Check for script injection in values
                let value_lower = value.to_lowercase();
                if value_lower.contains("<script")
                    || value_lower.contains("javascript:")
                    || value_lower.contains("data:text/html")
                    || value_lower.contains("eval(")
                {
                    return Err(HttpError::MaliciousPattern(
                        "Script injection detected in query".to_string(),
                    ));
                }

                // Check for SQL injection patterns
                if value_lower.contains("union select")
                    || value_lower.contains("drop table")
                    || value_lower.contains("' or 1=1")
                    || value_lower.contains("-- ")
                {
                    return Err(HttpError::MaliciousPattern(
                        "SQL injection detected in query".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Validate URL fragment
    fn validate_fragment(&self, url: &Url) -> Result<(), HttpError> {
        if let Some(fragment) = url.fragment() {
            // Check for script injection in fragment
            let fragment_lower = fragment.to_lowercase();
            if fragment_lower.contains("<script") || fragment_lower.contains("javascript:") {
                return Err(HttpError::MaliciousPattern(
                    "Script injection in fragment".to_string(),
                ));
            }

            // Check fragment length
            if fragment.len() > 512 {
                return Err(HttpError::MaliciousPattern("Fragment too long".to_string()));
            }
        }

        Ok(())
    }
}

impl Default for AdvancedUrlValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// HTTP header validator
pub struct HeaderValidator {
    /// Maximum header value length
    max_header_value_length: usize,
    /// Maximum number of headers
    max_header_count: usize,
    /// Blocked header names
    blocked_headers: HashSet<String>,
    /// Sensitive headers that need special handling
    sensitive_headers: HashSet<String>,
}

impl HeaderValidator {
    /// Create new header validator
    pub fn new() -> Self {
        let mut blocked_headers = HashSet::new();
        blocked_headers.extend(
            [
                "x-forwarded-for",
                "x-real-ip",
                "x-original-ip",
                "x-forwarded-proto",
                "x-forwarded-host",
            ]
            .iter()
            .map(|s| s.to_string()),
        );

        let mut sensitive_headers = HashSet::new();
        sensitive_headers.extend(
            ["authorization", "cookie", "x-api-key", "x-auth-token"]
                .iter()
                .map(|s| s.to_string()),
        );

        Self {
            max_header_value_length: 8192,
            max_header_count: 50,
            blocked_headers,
            sensitive_headers,
        }
    }

    /// Validate HTTP headers
    pub fn validate_headers(&self, headers: &HeaderMap) -> Result<(), HttpError> {
        // Check header count
        if headers.len() > self.max_header_count {
            return Err(HttpError::HeaderInjection("Too many headers".to_string()));
        }

        for (name, value) in headers {
            let name_str = name.as_str().to_lowercase();
            let value_str = value.to_str().map_err(|_| {
                HttpError::HeaderInjection("Invalid header value encoding".to_string())
            })?;

            // Check for blocked headers
            if self.blocked_headers.contains(&name_str) {
                return Err(HttpError::HeaderInjection(format!(
                    "Blocked header: {}",
                    name_str
                )));
            }

            // Check header value length
            if value_str.len() > self.max_header_value_length {
                return Err(HttpError::HeaderInjection(format!(
                    "Header value too long: {}",
                    name_str
                )));
            }

            // Check for header injection patterns
            if value_str.contains('\n') || value_str.contains('\r') {
                return Err(HttpError::HeaderInjection(format!(
                    "CRLF injection in header: {}",
                    name_str
                )));
            }

            // Check for null bytes
            if value_str.contains('\0') {
                return Err(HttpError::HeaderInjection(format!(
                    "Null byte in header: {}",
                    name_str
                )));
            }

            // Validate specific header types
            self.validate_specific_header(&name_str, value_str)?;
        }

        Ok(())
    }

    /// Validate specific header types
    fn validate_specific_header(&self, name: &str, value: &str) -> Result<(), HttpError> {
        match name {
            "content-length" => {
                if value.parse::<u64>().is_err() {
                    return Err(HttpError::HeaderInjection(
                        "Invalid Content-Length".to_string(),
                    ));
                }
            },
            "content-type" => {
                // Basic content-type validation
                if !value.contains('/') {
                    return Err(HttpError::HeaderInjection(
                        "Invalid Content-Type format".to_string(),
                    ));
                }
            },
            "host" => {
                // Host header should not contain suspicious patterns
                if value.contains(' ') || value.contains('\t') {
                    return Err(HttpError::HeaderInjection(
                        "Invalid Host header".to_string(),
                    ));
                }
            },
            "user-agent" => {
                // User-Agent should have reasonable length
                if value.len() > 512 {
                    return Err(HttpError::HeaderInjection(
                        "User-Agent too long".to_string(),
                    ));
                }
            },
            _ => {},
        }

        Ok(())
    }

    /// Check if header contains sensitive information
    pub fn is_sensitive_header(&self, name: &str) -> bool {
        self.sensitive_headers.contains(&name.to_lowercase())
    }

    /// Sanitize header value for logging
    pub fn sanitize_for_logging(&self, name: &str, value: &str) -> String {
        if self.is_sensitive_header(name) {
            "[REDACTED]".to_string()
        } else if value.len() > 100 {
            format!("{}...[truncated]", &value[..100])
        } else {
            value.to_string()
        }
    }
}

impl Default for HeaderValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Comprehensive request validator combining all validation logic
pub struct ComprehensiveRequestValidator {
    url_validator: AdvancedUrlValidator,
    header_validator: HeaderValidator,
    max_request_size: usize,
}

impl ComprehensiveRequestValidator {
    /// Create new comprehensive validator
    pub fn new(max_request_size: usize) -> Self {
        Self {
            url_validator: AdvancedUrlValidator::new(),
            header_validator: HeaderValidator::new(),
            max_request_size,
        }
    }

    /// Validate entire request context
    pub fn validate_request(
        &self,
        context: &RequestSecurityContext,
    ) -> Result<SecurityRiskLevel, HttpError> {
        // URL validation
        self.url_validator.validate_advanced(&context.url)?;

        // Header validation
        self.header_validator.validate_headers(&context.headers)?;

        // Body size validation
        if context.body_size > self.max_request_size {
            return Err(HttpError::RequestTooLarge {
                current: context.body_size,
                limit: self.max_request_size,
            });
        }

        // Calculate and return risk level
        let risk_level = context.risk_level();
        Ok(risk_level)
    }

    /// Get sanitized headers for logging
    pub fn sanitize_headers_for_logging(
        &self,
        headers: &HeaderMap,
    ) -> std::collections::HashMap<String, String> {
        headers
            .iter()
            .map(|(name, value)| {
                let name_str = name.as_str();
                let value_str = value.to_str().unwrap_or("[INVALID_ENCODING]");
                (
                    name_str.to_string(),
                    self.header_validator
                        .sanitize_for_logging(name_str, value_str),
                )
            })
            .collect()
    }
}

impl Default for ComprehensiveRequestValidator {
    fn default() -> Self {
        Self::new(10 * 1024 * 1024) // 10MB default
    }
}
