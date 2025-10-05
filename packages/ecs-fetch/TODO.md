# ECS Fetch Package TODO

## Overview
Bevy ECS-based HTTP client service wrapping reqwest with connection pooling, rate limiting, caching integration, security, and observability.

## Package Bootstrap and Structure
- [ ] Create Cargo.toml with dependencies: reqwest, bevy, tokio, serde, tracing, governor, uuid, thiserror
- [ ] Create lib.rs with module declarations and public API exports

## Core Resources Implementation  
- [ ] Implement HttpClientPool resource with reqwest::Client pool management and connection limits in resources.rs
- [ ] Implement HttpConfig resource with timeout settings, headers, TLS config, security policies in resources.rs 
- [ ] Implement RateLimitManager resource with token bucket algorithm and per-domain limits in resources.rs
- [ ] Implement RequestMetrics resource with latency histograms and bandwidth tracking in resources.rs

## Events System Implementation
- [ ] Implement HttpRequestSubmitted event with URL, method, headers, body, timeout, retry policy in events.rs
- [ ] Implement HttpResponseReceived event with status, headers, body, response time, cache metadata in events.rs
- [ ] Implement HttpRequestFailed event with error taxonomy and retry eligibility in events.rs
- [ ] Implement HttpRequestRetryRequested, RateLimitExceeded, HttpRequestCancelled events in events.rs

## Components Implementation
- [ ] Implement HttpRequest component with request ID, URL, method, timing, retry count in components.rs
- [ ] Implement RetryPolicy component with exponential backoff, jitter, maximum attempts in components.rs
- [ ] Implement RequestTimeout component with deadline tracking and cleanup coordination in components.rs

## Core Systems Implementation
- [ ] Implement process_http_requests_system with SSRF protection, rate limiting, async task spawning in systems.rs
- [ ] Implement process_http_responses_system with metrics updates, cache integration, response events in systems.rs
- [ ] Implement request_retry_system with exponential backoff, jitter, retry limit enforcement in systems.rs
- [ ] Implement rate_limiting_system with token bucket management and domain limiting in systems.rs
- [ ] Implement request_timeout_system and connection_pool_management_system in systems.rs

## Security Implementation
- [ ] Create security/mod.rs with SSRF protection: IP range validation, localhost detection, private network blocking
- [ ] Implement URL validation in security/validation.rs with scheme restrictions and malicious pattern detection
- [ ] Implement request sanitization in security/sanitization.rs with header injection prevention and size limits

## Authentication and Middleware
- [ ] Create auth/mod.rs with Bearer token support, API key handling, OAuth integration
- [ ] Create middleware/mod.rs with compression, content negotiation, custom middleware support

## Cache Integration
- [ ] Create cache_integration.rs with intelligent response caching, TTL calculation, conditional requests
- [ ] Integrate cache checking into process_http_requests_system for cache hits/misses

## Observability and Metrics
- [ ] Create metrics/mod.rs with request latency histograms, success/failure tracking, bandwidth monitoring
- [ ] Implement tracing.rs with request correlation IDs, span creation, trace propagation

## Plugin Integration
- [ ] Create plugin.rs implementing HttpPlugin with system registration and resource initialization

## Migration of Existing HTTP Code
- [ ] Extract and migrate HTTP service from packages/core/src/plugins/services/http.rs lines 1-300
- [ ] Update HTTP bridge handler in packages/core/src/plugins/bridge/handlers/http.rs lines 1-150
- [ ] Migrate extism host functions in packages/core/src/plugins/extism/host_functions/http.rs lines 1-200
- [ ] Update core dependencies in packages/core/Cargo.toml and lib.rs plugin registration

## Advanced Features Implementation
- [ ] Implement request deduplication in deduplication.rs with fingerprinting and response sharing
- [ ] Implement request prioritization in prioritization.rs with priority queues and starvation prevention
- [ ] Implement response streaming in streaming.rs with chunked handling and backpressure management
- [ ] Implement circuit breaker in circuit_breaker.rs with failure thresholds and automatic recovery

## Documentation and Architecture Updates
- [ ] Add ECS Fetch service documentation to ARCHITECTURE.md with complete API reference
- [ ] Update Service Overview table in ARCHITECTURE.md with HTTP service entry and usage examples

## Final Integration and Validation
- [ ] Update workspace Cargo.toml to include ecs-fetch package
- [ ] Verify compilation across entire workspace and run comprehensive test suite

## Performance Requirements
- Zero allocation optimization where possible
- Lock-free concurrent access patterns  
- Blazing-fast request processing
- Efficient memory usage
- No unsafe code blocks
- Elegant ergonomic APIs

## Security Requirements
- SSRF protection for all requests
- Header injection prevention
- Response size limits
- TLS certificate validation
- Request/response sanitization
- Sensitive data redaction in logs

## Integration Points
- Plugin HTTP requests (API calls, webhooks)
- Asset downloading (icons, images)  
- External service health monitoring
- Cloud service synchronization
- Authentication and authorization flows
- Cache integration with ecs-cache service

## Dependencies
- reqwest - HTTP client implementation
- bevy - ECS framework
- tokio - Async runtime  
- serde - JSON request/response handling
- tracing - Request tracing and logging
- governor - Rate limiting implementation
- uuid - Request correlation IDs
- thiserror - Error handling