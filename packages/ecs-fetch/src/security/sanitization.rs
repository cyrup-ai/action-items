//! Request Sanitization
//!
//! Comprehensive request and response sanitization for security and privacy.

use std::collections::HashMap;

use bytes::Bytes;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};

use super::HttpError;

/// Request sanitizer for security and privacy
pub struct RequestSanitizer {
    /// Maximum request body size
    max_body_size: usize,
    /// Sensitive headers to redact in logs
    sensitive_headers: Vec<String>,
    /// Headers to remove from requests
    blocked_headers: Vec<String>,
    /// Content types requiring special handling
    restricted_content_types: Vec<String>,
}

impl RequestSanitizer {
    /// Create new request sanitizer
    pub fn new(max_body_size: usize) -> Self {
        Self {
            max_body_size,
            sensitive_headers: vec![
                "authorization".to_string(),
                "cookie".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
                "x-access-token".to_string(),
                "bearer".to_string(),
                "proxy-authorization".to_string(),
            ],
            blocked_headers: vec![
                "x-forwarded-for".to_string(),
                "x-real-ip".to_string(),
                "x-original-ip".to_string(),
                "x-client-ip".to_string(),
            ],
            restricted_content_types: vec![
                "application/octet-stream".to_string(),
                "application/x-executable".to_string(),
                "application/x-msdownload".to_string(),
            ],
        }
    }

    /// Sanitize request headers
    pub fn sanitize_headers(&self, headers: &mut HeaderMap) -> Result<(), HttpError> {
        // Remove blocked headers
        for blocked_header in &self.blocked_headers {
            if let Ok(header_name) = HeaderName::from_bytes(blocked_header.as_bytes()) {
                headers.remove(&header_name);
            }
        }

        // Validate remaining headers
        let mut headers_to_remove = Vec::new();
        for (name, value) in headers.iter() {
            // Check for invalid characters
            let value_str = value
                .to_str()
                .map_err(|_| HttpError::HeaderInjection("Invalid header encoding".to_string()))?;

            // Check for header injection
            if value_str.contains('\r') || value_str.contains('\n') {
                headers_to_remove.push(name.clone());
                continue;
            }

            // Check for null bytes
            if value_str.contains('\0') {
                headers_to_remove.push(name.clone());
                continue;
            }

            // Limit header value length
            if value_str.len() > 8192 {
                headers_to_remove.push(name.clone());
                continue;
            }
        }

        // Remove invalid headers
        for header_name in headers_to_remove {
            headers.remove(&header_name);
        }

        // Add security headers
        self.add_security_headers(headers);

        Ok(())
    }

    /// Add security headers to request
    fn add_security_headers(&self, headers: &mut HeaderMap) {
        // Add user agent if not present
        if !headers.contains_key("user-agent") {
            if let Ok(value) = HeaderValue::from_str("ActionItems-ECS-Fetch/1.0") {
                headers.insert("user-agent", value);
            }
        }

        // Add accept encoding if not present
        if !headers.contains_key("accept-encoding") {
            if let Ok(value) = HeaderValue::from_str("gzip, br") {
                headers.insert("accept-encoding", value);
            }
        }

        // Add connection header for HTTP/1.1
        if !headers.contains_key("connection") {
            if let Ok(value) = HeaderValue::from_str("keep-alive") {
                headers.insert("connection", value);
            }
        }
    }

    /// Sanitize request body
    pub fn sanitize_body(
        &self,
        body: &Bytes,
        content_type: Option<&str>,
    ) -> Result<Bytes, HttpError> {
        // Check body size
        if body.len() > self.max_body_size {
            return Err(HttpError::RequestTooLarge {
                current: body.len(),
                limit: self.max_body_size,
            });
        }

        // Check for restricted content types
        if let Some(ct) = content_type {
            let ct_lower = ct.to_lowercase();
            for restricted in &self.restricted_content_types {
                if ct_lower.contains(restricted) {
                    return Err(HttpError::MaliciousPattern(format!(
                        "Restricted content type: {}",
                        ct
                    )));
                }
            }
        }

        // Check for binary executable signatures
        if body.len() >= 4 {
            let magic = &body[0..4];

            // Common executable signatures
            if magic == b"MZ\x90\x00" || // PE executable
               magic == b"\x7fELF" ||   // ELF executable
               magic == b"\xca\xfe\xba\xbe" || // Java class file
               magic == b"PK\x03\x04"
            {
                // ZIP/JAR (could be executable)
                return Err(HttpError::MaliciousPattern(
                    "Executable content detected".to_string(),
                ));
            }
        }

        // For text content, check for script injection
        if let Some(ct) = content_type {
            if ct.contains("text/")
                || ct.contains("application/json")
                || ct.contains("application/xml")
            {
                if let Ok(text) = std::str::from_utf8(body) {
                    self.check_text_content(text)?;
                }
            }
        }

        Ok(body.clone())
    }

    /// Check text content for malicious patterns
    fn check_text_content(&self, text: &str) -> Result<(), HttpError> {
        let text_lower = text.to_lowercase();

        // Check for script injection
        let script_patterns = [
            "<script",
            "javascript:",
            "data:text/html",
            "eval(",
            "setTimeout(",
            "setInterval(",
            "function(",
            "=eval",
            "document.cookie",
            "document.write",
        ];

        for pattern in &script_patterns {
            if text_lower.contains(pattern) {
                return Err(HttpError::MaliciousPattern(format!(
                    "Script injection pattern: {}",
                    pattern
                )));
            }
        }

        // Check for SQL injection patterns
        let sql_patterns = [
            "union select",
            "drop table",
            "delete from",
            "insert into",
            "' or 1=1",
            "' or '1'='1",
            "admin'--",
            "'; drop table",
        ];

        for pattern in &sql_patterns {
            if text_lower.contains(pattern) {
                return Err(HttpError::MaliciousPattern(format!(
                    "SQL injection pattern: {}",
                    pattern
                )));
            }
        }

        Ok(())
    }

    /// Get sanitized headers for logging
    pub fn sanitize_headers_for_logging(&self, headers: &HeaderMap) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(name, value)| {
                let name_str = name.as_str().to_lowercase();
                let value_str = value.to_str().unwrap_or("[INVALID_ENCODING]");

                let sanitized_value = if self.sensitive_headers.contains(&name_str) {
                    "[REDACTED]".to_string()
                } else if value_str.len() > 200 {
                    format!(
                        "{}...[truncated {} chars]",
                        &value_str[..200],
                        value_str.len() - 200
                    )
                } else {
                    value_str.to_string()
                };

                (name.as_str().to_string(), sanitized_value)
            })
            .collect()
    }

    /// Sanitize URL for logging (remove sensitive query parameters)
    pub fn sanitize_url_for_logging(&self, url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            let mut sanitized = url::Url::parse(&format!(
                "{}://{}",
                parsed.scheme(),
                parsed.host_str().unwrap_or("unknown")
            ))
            .unwrap();

            if let Some(port) = parsed.port() {
                sanitized.set_port(Some(port)).ok();
            }

            sanitized.set_path(parsed.path());

            // Sanitize query parameters
            if let Some(query) = parsed.query() {
                // Log raw query for debugging if needed
                tracing::trace!("Processing query string: {}", query);
                let sensitive_params = ["key", "token", "password", "secret", "auth", "api_key"];
                let mut clean_pairs = Vec::new();

                for (key, value) in parsed.query_pairs() {
                    let key_lower = key.to_lowercase();
                    let is_sensitive = sensitive_params
                        .iter()
                        .any(|&param| key_lower.contains(param));

                    if is_sensitive {
                        clean_pairs.push(format!("{}=[REDACTED]", key));
                    } else if value.len() > 50 {
                        clean_pairs.push(format!("{}={}...[truncated]", key, &value[..50]));
                    } else {
                        clean_pairs.push(format!("{}={}", key, value));
                    }
                }

                if !clean_pairs.is_empty() {
                    sanitized.set_query(Some(&clean_pairs.join("&")));
                }
            }

            sanitized.to_string()
        } else {
            "[INVALID_URL]".to_string()
        }
    }
}

impl Default for RequestSanitizer {
    fn default() -> Self {
        Self::new(10 * 1024 * 1024) // 10MB default
    }
}

/// Response sanitizer for logging and security
pub struct ResponseSanitizer {
    /// Maximum response body size for logging
    max_log_body_size: usize,
    /// Sensitive response headers
    sensitive_response_headers: Vec<String>,
}

impl ResponseSanitizer {
    /// Create new response sanitizer
    pub fn new() -> Self {
        Self {
            max_log_body_size: 1024, // Only log first 1KB of response
            sensitive_response_headers: vec![
                "set-cookie".to_string(),
                "www-authenticate".to_string(),
                "proxy-authenticate".to_string(),
                "x-api-key".to_string(),
                "x-auth-token".to_string(),
            ],
        }
    }

    /// Sanitize response headers for logging
    pub fn sanitize_response_headers_for_logging(
        &self,
        headers: &HeaderMap,
    ) -> HashMap<String, String> {
        headers
            .iter()
            .map(|(name, value)| {
                let name_str = name.as_str().to_lowercase();
                let value_str = value.to_str().unwrap_or("[INVALID_ENCODING]");

                let sanitized_value = if self.sensitive_response_headers.contains(&name_str) {
                    "[REDACTED]".to_string()
                } else if value_str.len() > 200 {
                    format!("{}...[truncated]", &value_str[..200])
                } else {
                    value_str.to_string()
                };

                (name.as_str().to_string(), sanitized_value)
            })
            .collect()
    }

    /// Sanitize response body for logging
    pub fn sanitize_response_body_for_logging(
        &self,
        body: &Bytes,
        content_type: Option<&str>,
    ) -> String {
        if body.is_empty() {
            return "[EMPTY]".to_string();
        }

        // Don't log binary content
        if let Some(ct) = content_type {
            let ct_lower = ct.to_lowercase();
            if ct_lower.contains("image/")
                || ct_lower.contains("video/")
                || ct_lower.contains("audio/")
                || ct_lower.contains("application/octet-stream")
            {
                return format!("[BINARY_DATA {} bytes]", body.len());
            }
        }

        // For text content, sanitize and truncate
        if let Ok(text) = std::str::from_utf8(body) {
            let sanitized = if text.len() > self.max_log_body_size {
                format!(
                    "{}...[truncated {} more bytes]",
                    &text[..self.max_log_body_size],
                    text.len() - self.max_log_body_size
                )
            } else {
                text.to_string()
            };

            // Remove potential credentials from JSON responses
            self.sanitize_json_credentials(&sanitized)
        } else {
            format!("[NON_UTF8_DATA {} bytes]", body.len())
        }
    }

    /// Sanitize JSON credentials in response body
    fn sanitize_json_credentials(&self, json_text: &str) -> String {
        let sensitive_fields = [
            "password",
            "secret",
            "token",
            "key",
            "auth",
            "credential",
            "api_key",
            "access_token",
            "refresh_token",
            "bearer",
            "jwt",
        ];

        let mut sanitized = json_text.to_string();

        for field in &sensitive_fields {
            // Simple pattern matching for different JSON value formats
            let patterns = [
                (
                    format!("\"{}\":", field),
                    format!("\"{}\":\"[REDACTED]\"", field),
                ),
                (format!("'{}':", field), format!("'{}':'[REDACTED]'", field)),
                (
                    format!("\"{}\" :", field),
                    format!("\"{}\" : \"[REDACTED]\"", field),
                ),
            ];

            for (search_pattern, replace_pattern) in &patterns {
                // Use the actual pattern for case-insensitive search and replacement
                let field_lower = field.to_lowercase();
                let sanitized_lower = sanitized.to_lowercase();
                let pattern_lower = search_pattern.to_lowercase();

                if sanitized_lower.contains(&pattern_lower) && pattern_lower.contains(&field_lower)
                {
                    // Perform case-insensitive replacement using the pattern
                    tracing::trace!(
                        "Sanitizing field '{}' using pattern '{}'",
                        field,
                        search_pattern
                    );

                    // Simple case-insensitive replacement
                    let mut result = String::new();
                    let mut remaining = sanitized.as_str();

                    while let Some(pos) = remaining
                        .to_lowercase()
                        .find(&search_pattern.to_lowercase())
                    {
                        result.push_str(&remaining[..pos]);
                        result.push_str(replace_pattern);
                        remaining = &remaining[pos + search_pattern.len()..];

                        // Skip to next value to avoid infinite replacement
                        if let Some(next_pos) = remaining.find(',') {
                            remaining = &remaining[next_pos..];
                        } else if let Some(next_pos) = remaining.find('}') {
                            remaining = &remaining[next_pos..];
                        } else {
                            break;
                        }
                    }
                    result.push_str(remaining);
                    sanitized = result;
                }
            }
        }

        sanitized
    }

    /// Check if response should be cached
    pub fn should_cache_response(&self, status: reqwest::StatusCode, headers: &HeaderMap) -> bool {
        // Don't cache error responses
        if !status.is_success() {
            return false;
        }

        // Check cache-control headers
        if let Some(cache_control) = headers.get("cache-control") {
            if let Ok(cache_control_str) = cache_control.to_str() {
                let cache_control_lower = cache_control_str.to_lowercase();
                if cache_control_lower.contains("no-cache")
                    || cache_control_lower.contains("no-store")
                    || cache_control_lower.contains("private")
                {
                    return false;
                }
            }
        }

        // Check for sensitive content types
        if let Some(content_type) = headers.get("content-type") {
            if let Ok(ct_str) = content_type.to_str() {
                let ct_lower = ct_str.to_lowercase();
                if ct_lower.contains("text/html") && ct_lower.contains("login") {
                    return false; // Don't cache login pages
                }
            }
        }

        true
    }
}

impl Default for ResponseSanitizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Content scanner for malicious patterns
pub struct ContentScanner {
    /// Known malware signatures
    malware_signatures: Vec<&'static [u8]>,
    /// Suspicious text patterns
    suspicious_patterns: Vec<&'static str>,
}

impl ContentScanner {
    /// Create new content scanner
    pub fn new() -> Self {
        Self {
            malware_signatures: vec![
                b"MZ\x90\x00",       // PE executable
                b"\x7fELF",          // ELF executable
                b"\xca\xfe\xba\xbe", // Java class file
                b"\xfe\xed\xfa\xce", // Mach-O 32-bit
                b"\xfe\xed\xfa\xcf", // Mach-O 64-bit
            ],
            suspicious_patterns: vec![
                "eval(",
                "exec(",
                "system(",
                "shell_exec(",
                "javascript:",
                "data:text/html",
                "<script",
                "document.write",
                "document.cookie",
                "localStorage",
                "sessionStorage",
                "XMLHttpRequest",
                "fetch(",
            ],
        }
    }

    /// Scan content for malicious patterns
    pub fn scan_content(&self, content: &Bytes) -> Result<(), HttpError> {
        // Check for malware signatures
        for signature in &self.malware_signatures {
            if self.contains_signature(content, signature) {
                return Err(HttpError::MaliciousPattern(
                    "Malware signature detected".to_string(),
                ));
            }
        }

        // Check text content for suspicious patterns
        if let Ok(text) = std::str::from_utf8(content) {
            let text_lower = text.to_lowercase();
            for pattern in &self.suspicious_patterns {
                if text_lower.contains(pattern) {
                    return Err(HttpError::MaliciousPattern(format!(
                        "Suspicious pattern detected: {}",
                        pattern
                    )));
                }
            }
        }

        Ok(())
    }

    /// Check if content contains signature
    fn contains_signature(&self, content: &Bytes, signature: &[u8]) -> bool {
        if content.len() < signature.len() {
            return false;
        }

        content
            .windows(signature.len())
            .any(|window| window == signature)
    }
}

impl Default for ContentScanner {
    fn default() -> Self {
        Self::new()
    }
}
