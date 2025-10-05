//! ECS Fetch - Bevy ECS HTTP Client Service
//!
//! A production-grade HTTP client service that wraps reqwest within the Bevy ECS architecture,
//! providing event-driven HTTP operations with connection pooling, rate limiting, caching
//! integration, security, and comprehensive observability.

pub mod auth;
pub mod cache_integration;
pub mod circuit_breaker;
pub mod components;
pub mod deduplication;
pub mod events;
pub mod metrics;
pub mod middleware;
pub mod plugin;
pub mod prioritization;
pub mod resources;
pub mod security;
pub mod streaming;
pub mod systems;
pub mod tracing;

// Re-export commonly used types
pub use components::{HttpRequest, RequestTimeout, RetryPolicy};
pub use events::{
    HttpRequestCancelled, HttpRequestFailed, HttpRequestRetryRequested, HttpRequestSubmitted,
    HttpResponseReceived, RateLimitExceeded,
};
// Cache integration types are already re-exported by cache_integration.rs via pub use cache_integration::*;
pub use http::HeaderMap;
pub use plugin::HttpPlugin;
// Re-export HTTP types for convenience
pub use reqwest::{Method, StatusCode, Version};
pub use resources::{HttpClientPool, HttpConfig, RateLimitManager, RequestMetrics};
/// HTTP client error types
pub use security::HttpError;
pub use systems::HttpRequestTask;
pub use url::Url;
