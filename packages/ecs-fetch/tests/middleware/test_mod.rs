//! Tests for middleware/mod.rs

use action_items_ecs_fetch::middleware::*;
use reqwest::header::{HeaderMap, ACCEPT, ACCEPT_ENCODING};

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
