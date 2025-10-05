use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bytes::Bytes;
use reqwest::{Method, StatusCode};
use tracing::{debug, info, warn};

use crate::events::{CorrelationId, HttpOperationId};

/// Request deduplication configuration
#[derive(Debug, Clone, Resource)]
pub struct DeduplicationConfig {
    /// Enable request deduplication
    pub enabled: bool,
    /// Time window for deduplication
    pub deduplication_window: Duration,
    /// Maximum number of pending duplicate requests
    pub max_pending_duplicates: usize,
    /// Deduplication strategy
    pub strategy: DeduplicationStrategy,
    /// Include headers in fingerprinting
    pub include_headers_in_fingerprint: bool,
    /// Headers to include in fingerprinting (if enabled)
    pub fingerprint_headers: Vec<String>,
}

impl Default for DeduplicationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            deduplication_window: Duration::from_secs(30),
            max_pending_duplicates: 100,
            strategy: DeduplicationStrategy::ContentBased,
            include_headers_in_fingerprint: false,
            fingerprint_headers: vec!["authorization".to_string(), "content-type".to_string()],
        }
    }
}

/// Deduplication strategies
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeduplicationStrategy {
    /// Deduplicate based on URL and method only
    UrlBased,
    /// Deduplicate based on full request content
    ContentBased,
    /// Deduplicate based on custom hash function
    HashBased,
}

/// Request fingerprint for deduplication
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RequestFingerprint {
    /// Request method
    pub method: String,
    /// Request URL
    pub url: String,
    /// Content hash (optional)
    pub content_hash: Option<u64>,
    /// Headers hash (optional)
    pub headers_hash: Option<u64>,
}

impl RequestFingerprint {
    /// Create fingerprint from request data
    pub fn from_request(
        method: &Method,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
        body: Option<&Bytes>,
        config: &DeduplicationConfig,
    ) -> Self {
        let mut fingerprint = RequestFingerprint {
            method: method.to_string(),
            url: url.to_string(),
            content_hash: None,
            headers_hash: None,
        };

        match config.strategy {
            DeduplicationStrategy::UrlBased => {
                // URL and method only - already set
            },
            DeduplicationStrategy::ContentBased => {
                // Include body content in fingerprint
                if let Some(body) = body {
                    fingerprint.content_hash = Some(Self::hash_bytes(body));
                }

                // Include headers if configured
                if config.include_headers_in_fingerprint {
                    if let Some(headers) = headers {
                        fingerprint.headers_hash = Some(Self::hash_headers(headers, config));
                    }
                }
            },
            DeduplicationStrategy::HashBased => {
                // Custom hash combining all elements
                let mut hasher = DefaultHasher::new();
                method.hash(&mut hasher);
                url.hash(&mut hasher);

                if let Some(body) = body {
                    body.hash(&mut hasher);
                }

                if let Some(headers) = headers {
                    Self::hash_headers_into(headers, &mut hasher, config);
                }

                fingerprint.content_hash = Some(hasher.finish());
            },
        }

        fingerprint
    }

    /// Hash byte content
    fn hash_bytes(bytes: &Bytes) -> u64 {
        let mut hasher = DefaultHasher::new();
        bytes.hash(&mut hasher);
        hasher.finish()
    }

    /// Hash headers based on configuration
    fn hash_headers(headers: &reqwest::header::HeaderMap, config: &DeduplicationConfig) -> u64 {
        let mut hasher = DefaultHasher::new();
        Self::hash_headers_into(headers, &mut hasher, config);
        hasher.finish()
    }

    /// Hash headers into existing hasher
    fn hash_headers_into(
        headers: &reqwest::header::HeaderMap,
        hasher: &mut DefaultHasher,
        config: &DeduplicationConfig,
    ) {
        // Sort headers for consistent hashing
        let mut header_pairs: Vec<_> = headers
            .iter()
            .filter_map(|(name, value)| {
                let name_str = name.as_str().to_lowercase();
                if config.fingerprint_headers.contains(&name_str) {
                    value.to_str().ok().map(|v| (name_str, v))
                } else {
                    None
                }
            })
            .collect();

        header_pairs.sort_by_key(|(name, _)| name.clone());

        for (name, value) in header_pairs {
            name.hash(hasher);
            value.hash(hasher);
        }
    }
}

/// Pending duplicate request information
#[derive(Debug, Clone)]
pub struct PendingDuplicateRequest {
    /// Operation ID of the duplicate request
    pub operation_id: HttpOperationId,
    /// Correlation ID for tracking
    pub correlation_id: CorrelationId,
    /// When the duplicate was detected
    pub detected_at: Instant,
    /// Requester identifier
    pub requester: String,
}

/// Active request tracking for deduplication
#[derive(Debug)]
pub struct ActiveRequest {
    /// Original operation ID
    pub operation_id: HttpOperationId,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// When request was started
    pub started_at: Instant,
    /// Pending duplicate requests
    pub pending_duplicates: Vec<PendingDuplicateRequest>,
    /// Original requester
    pub requester: String,
}

impl ActiveRequest {
    pub fn new(
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        requester: String,
    ) -> Self {
        Self {
            operation_id,
            correlation_id,
            started_at: Instant::now(),
            pending_duplicates: Vec::new(),
            requester,
        }
    }
}

/// Request deduplication manager
#[derive(Debug, Resource)]
pub struct DeduplicationManager {
    /// Active requests indexed by fingerprint
    pub active_requests: HashMap<RequestFingerprint, ActiveRequest>,
    /// Deduplication statistics
    pub stats: DeduplicationStats,
}

impl Default for DeduplicationManager {
    fn default() -> Self {
        Self {
            active_requests: HashMap::new(),
            stats: DeduplicationStats::default(),
        }
    }
}

impl DeduplicationManager {
    /// Check if request is a duplicate and handle accordingly
    pub fn check_and_handle_duplicate(
        &mut self,
        method: &Method,
        url: &str,
        headers: Option<&reqwest::header::HeaderMap>,
        body: Option<&Bytes>,
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        requester: &str,
        config: &DeduplicationConfig,
    ) -> DeduplicationResult {
        if !config.enabled {
            return DeduplicationResult::NotDuplicate;
        }

        // Clean up expired requests first
        self.cleanup_expired_requests(config);

        // Generate fingerprint for the request
        let fingerprint = RequestFingerprint::from_request(method, url, headers, body, config);

        // Check if this request is already active
        if let Some(active_request) = self.active_requests.get_mut(&fingerprint) {
            // This is a duplicate request
            self.stats.duplicates_detected += 1;

            // Check if we can add another duplicate
            if active_request.pending_duplicates.len() >= config.max_pending_duplicates {
                self.stats.duplicates_rejected += 1;
                warn!(
                    "Too many pending duplicates for request fingerprint, rejecting: {} {}",
                    method, url
                );
                return DeduplicationResult::TooManyDuplicates;
            }

            // Add to pending duplicates
            active_request
                .pending_duplicates
                .push(PendingDuplicateRequest {
                    operation_id,
                    correlation_id,
                    detected_at: Instant::now(),
                    requester: requester.to_string(),
                });

            debug!(
                "Duplicate request detected: {} {} (original: {:?}, duplicates: {})",
                method,
                url,
                active_request.operation_id,
                active_request.pending_duplicates.len()
            );

            DeduplicationResult::Duplicate {
                original_operation_id: active_request.operation_id,
                original_correlation_id: active_request.correlation_id,
                duplicate_count: active_request.pending_duplicates.len(),
            }
        } else {
            // This is a new unique request
            self.active_requests.insert(
                fingerprint.clone(),
                ActiveRequest::new(operation_id, correlation_id, requester.to_string()),
            );

            self.stats.unique_requests += 1;
            debug!("New unique request: {} {}", method, url);

            DeduplicationResult::NotDuplicate
        }
    }

    /// Complete a request and notify all duplicates
    pub fn complete_request(
        &mut self,
        fingerprint: &RequestFingerprint,
        response_data: Option<&ResponseData>,
    ) -> Vec<PendingDuplicateRequest> {
        if let Some(active_request) = self.active_requests.remove(fingerprint) {
            self.stats.requests_completed += 1;

            if !active_request.pending_duplicates.is_empty() {
                // Log response details for debugging duplicate resolution
                match response_data {
                    Some(resp_data) => {
                        info!(
                            "Completing request with {} pending duplicates - Status: {}, Body \
                             size: {}",
                            active_request.pending_duplicates.len(),
                            resp_data.status,
                            resp_data.body.as_ref().map(|b| b.len()).unwrap_or(0)
                        );

                        // Track successful response sharing
                        if resp_data.status.is_success() {
                            self.stats.duplicates_resolved +=
                                active_request.pending_duplicates.len() as u64;
                        }
                    },
                    None => {
                        info!(
                            "Completing request with {} pending duplicates - No response data \
                             available",
                            active_request.pending_duplicates.len()
                        );
                        self.stats.duplicates_resolved +=
                            active_request.pending_duplicates.len() as u64;
                    },
                }
            }

            active_request.pending_duplicates
        } else {
            Vec::new()
        }
    }

    /// Find and remove request by operation ID
    pub fn find_and_remove_request(
        &mut self,
        operation_id: HttpOperationId,
    ) -> Option<Vec<PendingDuplicateRequest>> {
        // Find the fingerprint for this operation ID
        let fingerprint = self
            .active_requests
            .iter()
            .find(|(_, active_request)| active_request.operation_id == operation_id)
            .map(|(fingerprint, _)| fingerprint.clone());

        if let Some(fingerprint) = fingerprint {
            Some(self.complete_request(&fingerprint, None))
        } else {
            None
        }
    }

    /// Clean up expired requests
    pub fn cleanup_expired_requests(&mut self, config: &DeduplicationConfig) {
        let now = Instant::now();
        let expired_fingerprints: Vec<_> = self
            .active_requests
            .iter()
            .filter_map(|(fingerprint, active_request)| {
                if now.duration_since(active_request.started_at) > config.deduplication_window {
                    Some(fingerprint.clone())
                } else {
                    None
                }
            })
            .collect();

        for fingerprint in expired_fingerprints {
            if let Some(active_request) = self.active_requests.remove(&fingerprint) {
                self.stats.requests_expired += 1;
                if !active_request.pending_duplicates.is_empty() {
                    warn!(
                        "Expired request had {} pending duplicates",
                        active_request.pending_duplicates.len()
                    );
                    self.stats.duplicates_expired += active_request.pending_duplicates.len() as u64;
                }
            }
        }
    }

    /// Get current statistics
    pub fn get_stats(&self) -> &DeduplicationStats {
        &self.stats
    }

    /// Get active request count
    pub fn active_request_count(&self) -> usize {
        self.active_requests.len()
    }

    /// Get total pending duplicates
    pub fn total_pending_duplicates(&self) -> usize {
        self.active_requests
            .values()
            .map(|req| req.pending_duplicates.len())
            .sum()
    }
}

/// Result of deduplication check
#[derive(Debug, Clone)]
pub enum DeduplicationResult {
    /// Request is not a duplicate
    NotDuplicate,
    /// Request is a duplicate of an active request
    Duplicate {
        original_operation_id: HttpOperationId,
        original_correlation_id: CorrelationId,
        duplicate_count: usize,
    },
    /// Too many duplicates for this request pattern
    TooManyDuplicates,
}

/// Response data for sharing among duplicates
#[derive(Debug, Clone)]
pub struct ResponseData {
    pub status: StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub body: Option<Bytes>,
}

/// Deduplication statistics
#[derive(Debug, Default)]
pub struct DeduplicationStats {
    /// Unique requests processed
    pub unique_requests: u64,
    /// Duplicate requests detected
    pub duplicates_detected: u64,
    /// Duplicate requests resolved (shared response)
    pub duplicates_resolved: u64,
    /// Duplicate requests rejected (too many)
    pub duplicates_rejected: u64,
    /// Requests that expired before completion
    pub requests_expired: u64,
    /// Duplicates that expired before resolution
    pub duplicates_expired: u64,
    /// Total requests completed
    pub requests_completed: u64,
}

impl DeduplicationStats {
    /// Get total requests processed
    #[inline]
    pub fn total_requests(&self) -> u64 {
        self.unique_requests + self.duplicates_detected
    }

    /// Get deduplication ratio
    #[inline]
    pub fn deduplication_ratio(&self) -> f64 {
        let total = self.total_requests();
        if total > 0 {
            self.duplicates_detected as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get bandwidth savings estimate
    #[inline]
    pub fn estimated_bandwidth_savings(&self) -> u64 {
        // Estimate: each deduplicated request saves 100% of the bandwidth
        self.duplicates_resolved
    }
}

/// Deduplication events
#[derive(Debug, Clone, Event)]
pub struct DuplicateRequestDetected {
    pub duplicate_operation_id: HttpOperationId,
    pub duplicate_correlation_id: CorrelationId,
    pub original_operation_id: HttpOperationId,
    pub original_correlation_id: CorrelationId,
    pub method: String,
    pub url: String,
    pub duplicate_count: usize,
    pub detected_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct DuplicateRequestResolved {
    pub original_operation_id: HttpOperationId,
    pub duplicate_operation_ids: Vec<HttpOperationId>,
    pub method: String,
    pub url: String,
    pub resolved_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct TooManyDuplicatesRejected {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub method: String,
    pub url: String,
    pub rejected_at: Instant,
}

#[cfg(test)]
mod tests {
    use reqwest::Method;

    use super::*;

    #[test]
    fn test_request_fingerprint_url_based() {
        let config = DeduplicationConfig {
            strategy: DeduplicationStrategy::UrlBased,
            ..Default::default()
        };

        let fp1 = RequestFingerprint::from_request(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            &config,
        );

        let fp2 = RequestFingerprint::from_request(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            &config,
        );

        assert_eq!(fp1, fp2);
        assert_eq!(fp1.content_hash, None);
    }

    #[test]
    fn test_request_fingerprint_content_based() {
        let config = DeduplicationConfig {
            strategy: DeduplicationStrategy::ContentBased,
            ..Default::default()
        };

        let body1 = Bytes::from("test body");
        let body2 = Bytes::from("different body");

        let fp1 = RequestFingerprint::from_request(
            &Method::POST,
            "https://api.example.com/users",
            None,
            Some(&body1),
            &config,
        );

        let fp2 = RequestFingerprint::from_request(
            &Method::POST,
            "https://api.example.com/users",
            None,
            Some(&body2),
            &config,
        );

        assert_ne!(fp1, fp2);
        assert!(fp1.content_hash.is_some());
        assert!(fp2.content_hash.is_some());
        assert_ne!(fp1.content_hash, fp2.content_hash);
    }

    #[test]
    fn test_deduplication_manager() {
        let mut manager = DeduplicationManager::default();
        let config = DeduplicationConfig::default();
        let op_id_1 = uuid::Uuid::new_v4();
        let op_id_2 = uuid::Uuid::new_v4();
        let corr_id_1 = uuid::Uuid::new_v4();
        let corr_id_2 = uuid::Uuid::new_v4();

        // First request should not be a duplicate
        let result1 = manager.check_and_handle_duplicate(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            op_id_1,
            corr_id_1,
            "requester1",
            &config,
        );

        assert!(matches!(result1, DeduplicationResult::NotDuplicate));
        assert_eq!(manager.stats.unique_requests, 1);

        // Second identical request should be a duplicate
        let result2 = manager.check_and_handle_duplicate(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            op_id_2,
            corr_id_2,
            "requester2",
            &config,
        );

        assert!(matches!(result2, DeduplicationResult::Duplicate { .. }));
        assert_eq!(manager.stats.duplicates_detected, 1);
        assert_eq!(manager.active_request_count(), 1);
        assert_eq!(manager.total_pending_duplicates(), 1);
    }

    #[test]
    fn test_request_completion() {
        let mut manager = DeduplicationManager::default();
        let config = DeduplicationConfig::default();
        let op_id = uuid::Uuid::new_v4();
        let corr_id = uuid::Uuid::new_v4();

        // Add a request with duplicates
        manager.check_and_handle_duplicate(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            op_id,
            corr_id,
            "requester",
            &config,
        );

        // Add duplicate
        manager.check_and_handle_duplicate(
            &Method::GET,
            "https://api.example.com/users",
            None,
            None,
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            "requester2",
            &config,
        );

        assert_eq!(manager.total_pending_duplicates(), 1);

        // Complete the request
        let duplicates = manager.find_and_remove_request(op_id);
        assert!(duplicates.is_some());
        assert_eq!(duplicates.unwrap().len(), 1);
        assert_eq!(manager.active_request_count(), 0);
        assert_eq!(manager.stats.duplicates_resolved, 1);
    }

    #[test]
    fn test_deduplication_stats() {
        let stats = DeduplicationStats {
            unique_requests: 70,
            duplicates_detected: 30,
            duplicates_resolved: 25,
            ..Default::default()
        };

        assert_eq!(stats.total_requests(), 100);
        assert_eq!(stats.deduplication_ratio(), 0.3);
        assert_eq!(stats.estimated_bandwidth_savings(), 25);
    }
}
