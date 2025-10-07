use std::collections::HashMap;
use std::fmt;

use bevy::prelude::*;
use bytes::Bytes;
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE, HeaderMap, HeaderName, HeaderValue,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tracing::{debug, error, warn};

/// Middleware configuration resource
#[derive(Debug, Clone, Resource)]
pub struct MiddlewareConfig {
    /// Compression settings
    pub compression: CompressionConfig,
    /// Content negotiation settings
    pub content_negotiation: ContentNegotiationConfig,
    /// Request transformation middleware
    pub request_middleware: Vec<RequestMiddleware>,
    /// Response transformation middleware
    pub response_middleware: Vec<ResponseMiddleware>,
    /// Custom middleware enabled
    pub custom_middleware_enabled: bool,
}

impl Default for MiddlewareConfig {
    fn default() -> Self {
        Self {
            compression: CompressionConfig::default(),
            content_negotiation: ContentNegotiationConfig::default(),
            request_middleware: Vec::new(),
            response_middleware: Vec::new(),
            custom_middleware_enabled: true,
        }
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Supported compression algorithms in preference order
    pub algorithms: Vec<CompressionAlgorithm>,
    /// Minimum content length to compress
    pub min_content_length: usize,
    /// Content types to compress
    pub compressible_types: Vec<String>,
    /// Automatic decompression enabled
    pub auto_decompress: bool,
    /// Compression quality (1-9 for deflate/gzip)
    pub compression_level: u32,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            algorithms: vec![
                CompressionAlgorithm::Brotli,
                CompressionAlgorithm::Gzip,
                CompressionAlgorithm::Deflate,
            ],
            min_content_length: 1024, // 1KB minimum
            compressible_types: vec![
                "text/".to_string(),
                "application/json".to_string(),
                "application/javascript".to_string(),
                "application/xml".to_string(),
                "image/svg+xml".to_string(),
            ],
            auto_decompress: true,
            compression_level: 6, // Balanced compression
        }
    }
}

/// Content negotiation configuration
#[derive(Debug, Clone)]
pub struct ContentNegotiationConfig {
    /// Preferred content types
    pub preferred_types: Vec<ContentType>,
    /// Preferred languages
    pub preferred_languages: Vec<String>,
    /// Preferred encodings (character sets)
    pub preferred_encodings: Vec<String>,
    /// Quality factor adjustment
    pub quality_factor_adjustment: f32,
    /// Strict content negotiation
    pub strict_negotiation: bool,
}

impl Default for ContentNegotiationConfig {
    fn default() -> Self {
        Self {
            preferred_types: vec![ContentType::Json, ContentType::Plain, ContentType::Html],
            preferred_languages: vec!["en".to_string(), "en-US".to_string()],
            preferred_encodings: vec!["utf-8".to_string()],
            quality_factor_adjustment: 0.1,
            strict_negotiation: false,
        }
    }
}

/// Supported compression algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// Brotli compression (preferred)
    Brotli,
    /// Gzip compression
    Gzip,
    /// Deflate compression  
    Deflate,
    /// Identity (no compression)
    Identity,
}

impl CompressionAlgorithm {
    /// Get HTTP header value for compression algorithm
    #[inline]
    pub fn as_header_value(&self) -> &'static str {
        match self {
            CompressionAlgorithm::Brotli => "br",
            CompressionAlgorithm::Gzip => "gzip",
            CompressionAlgorithm::Deflate => "deflate",
            CompressionAlgorithm::Identity => "identity",
        }
    }

    /// Parse compression algorithm from header value
    pub fn from_header_value(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "br" => Some(CompressionAlgorithm::Brotli),
            "gzip" => Some(CompressionAlgorithm::Gzip),
            "deflate" => Some(CompressionAlgorithm::Deflate),
            "identity" => Some(CompressionAlgorithm::Identity),
            _ => None,
        }
    }

    /// Check if content type is compressible with this algorithm
    pub fn is_content_compressible(&self, content_type: &str, config: &CompressionConfig) -> bool {
        if *self == CompressionAlgorithm::Identity {
            return false;
        }

        config
            .compressible_types
            .iter()
            .any(|compressible| content_type.starts_with(compressible))
    }
}

/// Content types for negotiation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentType {
    Json,
    Xml,
    Html,
    Plain,
    Binary,
    FormData,
    Custom(&'static str),
}

impl ContentType {
    /// Get MIME type string
    #[inline]
    pub fn as_mime_type(&self) -> &'static str {
        match self {
            ContentType::Json => "application/json",
            ContentType::Xml => "application/xml",
            ContentType::Html => "text/html",
            ContentType::Plain => "text/plain",
            ContentType::Binary => "application/octet-stream",
            ContentType::FormData => "application/x-www-form-urlencoded",
            ContentType::Custom(mime) => mime,
        }
    }

    /// Parse content type from MIME string
    pub fn from_mime_type(mime: &str) -> Self {
        let mime = mime.split(';').next().unwrap_or(mime).trim();
        match mime {
            "application/json" => ContentType::Json,
            "application/xml" | "text/xml" => ContentType::Xml,
            "text/html" => ContentType::Html,
            "text/plain" => ContentType::Plain,
            "application/octet-stream" => ContentType::Binary,
            "application/x-www-form-urlencoded" => ContentType::FormData,
            _ => ContentType::Binary,
        }
    }

    /// Build Accept header with quality factors
    pub fn build_accept_header(types: &[ContentType], quality_adjustment: f32) -> String {
        let mut accept_parts = Vec::new();
        let mut quality = 1.0_f32;

        for content_type in types {
            let mime_type = content_type.as_mime_type();
            if quality >= 1.0 {
                accept_parts.push(mime_type.to_string());
            } else {
                accept_parts.push(format!("{};q={:.1}", mime_type, quality));
            }
            quality = (quality - quality_adjustment).max(0.1);
        }

        accept_parts.join(", ")
    }
}

/// Request middleware types
#[derive(Debug, Clone)]
pub enum RequestMiddleware {
    /// Add custom headers
    AddHeaders(HashMap<String, String>),
    /// User agent setting
    UserAgent(String),
    /// Request logging
    RequestLogging { log_headers: bool, log_body: bool },
    /// Request transformation
    Transform(RequestTransform),
}

/// Response middleware types
#[derive(Debug, Clone)]
pub enum ResponseMiddleware {
    /// Response logging
    ResponseLogging {
        log_headers: bool,
        log_body: bool,
        max_body_length: usize,
    },
    /// Response transformation
    Transform(ResponseTransform),
    /// Cache headers processing
    CacheHeaders,
    /// Security headers validation
    SecurityHeaders,
}

/// Request transformation functions
#[derive(Debug, Clone)]
pub enum RequestTransform {
    /// JSON body transformation
    JsonTransform(String), // JSONPath or transformation script
    /// Header manipulation
    HeaderTransform(String),
    /// URL modification
    UrlTransform(String),
}

/// Response transformation functions
#[derive(Debug, Clone)]
pub enum ResponseTransform {
    /// JSON response transformation
    JsonTransform(String),
    /// HTML content extraction
    HtmlExtract(String),
    /// Binary data processing
    BinaryTransform(String),
}

/// Middleware processor for applying transformations
#[derive(Debug, Default, Resource)]
pub struct MiddlewareProcessor {
    /// Processing statistics
    pub stats: MiddlewareStats,
}

impl MiddlewareProcessor {
    /// Apply request middleware
    pub fn process_request(
        &mut self,
        headers: &mut HeaderMap,
        url: &str,
        body: &Option<Bytes>,
        config: &MiddlewareConfig,
    ) -> Result<(), MiddlewareError> {
        // Apply compression headers
        self.apply_compression_request_headers(headers, config)?;

        // Apply content negotiation headers
        self.apply_content_negotiation_headers(headers, config)?;

        // Process request middleware
        for middleware in &config.request_middleware {
            self.apply_request_middleware(headers, url, body, middleware)?;
        }

        self.stats.requests_processed += 1;
        Ok(())
    }

    /// Apply response middleware
    pub fn process_response(
        &mut self,
        response_headers: &HeaderMap,
        response_body: &Option<Bytes>,
        config: &MiddlewareConfig,
    ) -> Result<ProcessedResponse, MiddlewareError> {
        let mut processed_body = response_body.clone();
        let mut metadata = ResponseMetadata::default();

        // Extract compression information
        if let Some(encoding) = response_headers.get(CONTENT_ENCODING) {
            if let Ok(encoding_str) = encoding.to_str() {
                metadata.compression = CompressionAlgorithm::from_header_value(encoding_str);

                // Auto-decompress if enabled and body is compressed
                if config.compression.auto_decompress && processed_body.is_some() {
                    processed_body =
                        self.decompress_response(processed_body, metadata.compression)?;
                    metadata.decompressed = true;
                }
            }
        }

        // Extract content type
        if let Some(content_type) = response_headers.get(CONTENT_TYPE) {
            if let Ok(ct_str) = content_type.to_str() {
                metadata.content_type = Some(ContentType::from_mime_type(ct_str));
            }
        }

        // Process response middleware
        for middleware in &config.response_middleware {
            processed_body = self.apply_response_middleware(
                response_headers,
                processed_body,
                &mut metadata,
                middleware,
            )?;
        }

        self.stats.responses_processed += 1;

        Ok(ProcessedResponse {
            body: processed_body,
            metadata,
        })
    }

    /// Apply compression request headers
    fn apply_compression_request_headers(
        &self,
        headers: &mut HeaderMap,
        config: &MiddlewareConfig,
    ) -> Result<(), MiddlewareError> {
        if !config.compression.algorithms.is_empty() {
            let accept_encoding = config
                .compression
                .algorithms
                .iter()
                .map(|alg| alg.as_header_value())
                .collect::<Vec<_>>()
                .join(", ");

            let header_value = HeaderValue::from_str(&accept_encoding)
                .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;

            headers.insert(ACCEPT_ENCODING, header_value);
            debug!("Added compression headers: {}", accept_encoding);
        }
        Ok(())
    }

    /// Apply content negotiation headers
    fn apply_content_negotiation_headers(
        &self,
        headers: &mut HeaderMap,
        config: &MiddlewareConfig,
    ) -> Result<(), MiddlewareError> {
        // Build Accept header
        let accept_value = ContentType::build_accept_header(
            &config.content_negotiation.preferred_types,
            config.content_negotiation.quality_factor_adjustment,
        );

        let header_value = HeaderValue::from_str(&accept_value)
            .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;

        headers.insert(ACCEPT, header_value);

        // Add Accept-Language if configured
        if !config.content_negotiation.preferred_languages.is_empty() {
            let accept_lang = config.content_negotiation.preferred_languages.join(", ");
            let lang_header = HeaderValue::from_str(&accept_lang)
                .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;
            headers.insert("accept-language", lang_header);
        }

        debug!("Applied content negotiation headers");
        Ok(())
    }

    /// Apply individual request middleware
    fn apply_request_middleware(
        &mut self,
        headers: &mut HeaderMap,
        _url: &str,
        _body: &Option<Bytes>,
        middleware: &RequestMiddleware,
    ) -> Result<(), MiddlewareError> {
        match middleware {
            RequestMiddleware::AddHeaders(custom_headers) => {
                for (name, value) in custom_headers {
                    let header_name = HeaderName::from_bytes(name.as_bytes())
                        .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;
                    let header_value = HeaderValue::from_str(value)
                        .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;
                    headers.insert(header_name, header_value);
                }
            },
            RequestMiddleware::UserAgent(user_agent) => {
                let header_value = HeaderValue::from_str(user_agent)
                    .map_err(|e| MiddlewareError::InvalidHeader(e.to_string()))?;
                headers.insert("user-agent", header_value);
            },
            RequestMiddleware::RequestLogging {
                log_headers,
                log_body: _,
            } => {
                if *log_headers {
                    debug!("Request headers: {:?}", headers);
                }
                // Body logging would be implemented here
            },
            RequestMiddleware::Transform(_transform) => {
                // Transform middleware would be implemented here
                warn!("Request transformation not yet implemented");
            },
        }
        Ok(())
    }

    /// Apply individual response middleware
    fn apply_response_middleware(
        &mut self,
        response_headers: &HeaderMap,
        response_body: Option<Bytes>,
        metadata: &mut ResponseMetadata,
        middleware: &ResponseMiddleware,
    ) -> Result<Option<Bytes>, MiddlewareError> {
        match middleware {
            ResponseMiddleware::ResponseLogging {
                log_headers,
                log_body,
                max_body_length,
            } => {
                if *log_headers {
                    debug!("Response headers: {:?}", response_headers);
                }
                if *log_body {
                    if let Some(body) = response_body.as_ref() {
                        let log_length = (*max_body_length).min(body.len());
                        let body_preview = String::from_utf8_lossy(&body[..log_length]);
                        debug!("Response body preview: {}", body_preview);
                    }
                }
            },
            ResponseMiddleware::Transform(_transform) => {
                warn!("Response transformation not yet implemented");
            },
            ResponseMiddleware::CacheHeaders => {
                // Extract cache-related headers
                if let Some(cache_control) = response_headers.get("cache-control") {
                    metadata.cache_control = cache_control.to_str().ok().map(|s| s.to_string());
                }
                if let Some(etag) = response_headers.get("etag") {
                    metadata.etag = etag.to_str().ok().map(|s| s.to_string());
                }
            },
            ResponseMiddleware::SecurityHeaders => {
                // Validate security headers
                metadata.has_security_headers = response_headers
                    .contains_key("strict-transport-security")
                    || response_headers.contains_key("content-security-policy");
            },
        }
        Ok(response_body)
    }

    /// Decompress response body
    fn decompress_response(
        &self,
        body: Option<Bytes>,
        compression: Option<CompressionAlgorithm>,
    ) -> Result<Option<Bytes>, MiddlewareError> {
        match (body, compression) {
            (Some(compressed_body), Some(algorithm)) => {
                match algorithm {
                    CompressionAlgorithm::Gzip => {
                        // Gzip decompression would be implemented here
                        warn!("Gzip decompression not yet implemented");
                        Ok(Some(compressed_body))
                    },
                    CompressionAlgorithm::Deflate => {
                        // Deflate decompression would be implemented here
                        warn!("Deflate decompression not yet implemented");
                        Ok(Some(compressed_body))
                    },
                    CompressionAlgorithm::Brotli => {
                        // Brotli decompression would be implemented here
                        warn!("Brotli decompression not yet implemented");
                        Ok(Some(compressed_body))
                    },
                    CompressionAlgorithm::Identity => Ok(Some(compressed_body)),
                }
            },
            (body, _) => Ok(body),
        }
    }
}

/// Processed response with metadata
#[derive(Debug)]
pub struct ProcessedResponse {
    pub body: Option<Bytes>,
    pub metadata: ResponseMetadata,
}

/// Response processing metadata
#[derive(Debug, Default)]
pub struct ResponseMetadata {
    pub compression: Option<CompressionAlgorithm>,
    pub decompressed: bool,
    pub content_type: Option<ContentType>,
    pub cache_control: Option<String>,
    pub etag: Option<String>,
    pub has_security_headers: bool,
}

/// Middleware processing statistics
#[derive(Debug, Default)]
pub struct MiddlewareStats {
    pub requests_processed: u64,
    pub responses_processed: u64,
    pub compression_applied: u64,
    pub decompression_performed: u64,
    pub content_negotiation_applied: u64,
    pub transformation_applied: u64,
}

/// Middleware errors
#[derive(Debug, Error)]
pub enum MiddlewareError {
    #[error("Invalid header: {0}")]
    InvalidHeader(String),

    #[error("Compression error: {0}")]
    CompressionError(String),

    #[error("Decompression error: {0}")]
    DecompressionError(String),

    #[error("Content negotiation failed: {0}")]
    ContentNegotiationFailed(String),

    #[error("Transformation error: {0}")]
    TransformationError(String),

    #[error("Middleware configuration error: {0}")]
    ConfigurationError(String),
}

impl fmt::Display for CompressionAlgorithm {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_header_value())
    }
}

impl fmt::Display for ContentType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_mime_type())
    }
}

/// Builder pattern for middleware configuration
pub struct MiddlewareConfigBuilder {
    config: MiddlewareConfig,
}

impl MiddlewareConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: MiddlewareConfig::default(),
        }
    }

    pub fn with_compression(mut self, algorithms: Vec<CompressionAlgorithm>) -> Self {
        self.config.compression.algorithms = algorithms;
        self
    }

    pub fn with_content_types(mut self, types: Vec<ContentType>) -> Self {
        self.config.content_negotiation.preferred_types = types;
        self
    }

    pub fn add_request_middleware(mut self, middleware: RequestMiddleware) -> Self {
        self.config.request_middleware.push(middleware);
        self
    }

    pub fn add_response_middleware(mut self, middleware: ResponseMiddleware) -> Self {
        self.config.response_middleware.push(middleware);
        self
    }

    pub fn build(self) -> MiddlewareConfig {
        self.config
    }
}

impl Default for MiddlewareConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compression_algorithm_header_values() {
        assert_eq!(CompressionAlgorithm::Brotli.as_header_value(), "br");
        assert_eq!(CompressionAlgorithm::Gzip.as_header_value(), "gzip");
        assert_eq!(CompressionAlgorithm::Deflate.as_header_value(), "deflate");
        assert_eq!(CompressionAlgorithm::Identity.as_header_value(), "identity");
    }

    #[test]
    fn test_compression_algorithm_parsing() {
        assert_eq!(
            CompressionAlgorithm::from_header_value("br"),
            Some(CompressionAlgorithm::Brotli)
        );
        assert_eq!(
            CompressionAlgorithm::from_header_value("gzip"),
            Some(CompressionAlgorithm::Gzip)
        );
        assert_eq!(CompressionAlgorithm::from_header_value("unknown"), None);
    }

    #[test]
    fn test_content_type_mime_types() {
        assert_eq!(ContentType::Json.as_mime_type(), "application/json");
        assert_eq!(ContentType::Html.as_mime_type(), "text/html");
        assert_eq!(ContentType::Plain.as_mime_type(), "text/plain");
    }

    #[test]
    fn test_content_type_parsing() {
        assert_eq!(
            ContentType::from_mime_type("application/json"),
            ContentType::Json
        );
        assert_eq!(
            ContentType::from_mime_type("text/html; charset=utf-8"),
            ContentType::Html
        );
        assert_eq!(
            ContentType::from_mime_type("unknown/type"),
            ContentType::Binary
        );
    }

    #[test]
    fn test_accept_header_building() {
        let types = vec![ContentType::Json, ContentType::Html];
        let header = ContentType::build_accept_header(&types, 0.1);
        assert!(header.contains("application/json"));
        assert!(header.contains("text/html"));
    }

    #[test]
    fn test_middleware_config_builder() {
        let config = MiddlewareConfigBuilder::new()
            .with_compression(vec![CompressionAlgorithm::Gzip])
            .with_content_types(vec![ContentType::Json])
            .add_request_middleware(RequestMiddleware::UserAgent("test-agent".to_string()))
            .build();

        assert_eq!(config.compression.algorithms.len(), 1);
        assert_eq!(config.content_negotiation.preferred_types.len(), 1);
        assert_eq!(config.request_middleware.len(), 1);
    }

    #[test]
    fn test_middleware_processor_basic() {
        let mut processor = MiddlewareProcessor::default();
        let config = MiddlewareConfig::default();
        let mut headers = HeaderMap::new();

        let result = processor.process_request(&mut headers, "https://example.com", &None, &config);
        assert!(result.is_ok());
        assert_eq!(processor.stats.requests_processed, 1);

        // Should have compression headers
        assert!(headers.contains_key(ACCEPT_ENCODING));
        assert!(headers.contains_key(ACCEPT));
    }
}
