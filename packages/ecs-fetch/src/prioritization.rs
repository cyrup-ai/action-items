use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bevy::prelude::*;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tracing::debug;

use crate::events::{CorrelationId, HttpOperationId, RequestPriority};

/// Serialize Option<Instant> as Option<u64> (seconds since UNIX_EPOCH)
fn serialize_instant_option<S>(instant: &Option<Instant>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match instant {
        Some(instant) => {
            // Convert Instant to SystemTime for serialization
            let duration_since_start = instant.elapsed();
            let now = SystemTime::now();
            let instant_as_systemtime = now - duration_since_start;
            let secs = instant_as_systemtime
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            Some(secs).serialize(serializer)
        },
        None => serializer.serialize_none(),
    }
}

/// Deserialize Option<u64> as Option<Instant>
fn deserialize_instant_option<'de, D>(deserializer: D) -> Result<Option<Instant>, D::Error>
where
    D: Deserializer<'de>,
{
    let opt_secs: Option<u64> = Option::deserialize(deserializer)?;
    match opt_secs {
        Some(secs) => {
            let system_time = UNIX_EPOCH + Duration::from_secs(secs);
            // Convert SystemTime back to Instant (approximate)
            let now_system = SystemTime::now();
            let now_instant = Instant::now();
            if let Ok(duration) = now_system.duration_since(system_time) {
                Ok(Some(now_instant - duration))
            } else if let Ok(duration) = system_time.duration_since(now_system) {
                Ok(Some(now_instant + duration))
            } else {
                Ok(Some(now_instant))
            }
        },
        None => Ok(None),
    }
}

/// Request prioritization configuration
#[derive(Debug, Clone, Resource)]
pub struct PrioritizationConfig {
    /// Enable request prioritization
    pub enabled: bool,
    /// Maximum queue size per priority level
    pub max_queue_size_per_priority: usize,
    /// Starvation prevention timeout
    pub starvation_prevention_timeout: Duration,
    /// High priority request threshold (requests per second)
    pub high_priority_rate_limit: f64,
    /// Aging strategy for preventing starvation
    pub aging_strategy: AgingStrategy,
    /// Priority boost for aged requests
    pub age_priority_boost: u8,
}

impl Default for PrioritizationConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_queue_size_per_priority: 1000,
            starvation_prevention_timeout: Duration::from_secs(30),
            high_priority_rate_limit: 100.0, // 100 requests per second max
            aging_strategy: AgingStrategy::Exponential,
            age_priority_boost: 1,
        }
    }
}

/// Aging strategies to prevent starvation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgingStrategy {
    /// No aging - requests maintain original priority
    None,
    /// Linear aging - priority increases linearly with time
    Linear,
    /// Exponential aging - priority increases exponentially with time
    Exponential,
    /// Threshold aging - priority jumps after threshold time
    Threshold,
}

/// Prioritized HTTP request
#[derive(Debug, Clone)]
pub struct PrioritizedRequest {
    /// Operation ID
    pub operation_id: HttpOperationId,
    /// Correlation ID
    pub correlation_id: CorrelationId,
    /// Request priority (higher number = higher priority)
    pub priority: RequestPriority,
    /// Effective priority (after aging adjustments)
    pub effective_priority: u8,
    /// When request was queued
    pub queued_at: Instant,
    /// Original priority (before aging)
    pub original_priority: RequestPriority,
    /// Request metadata
    pub metadata: RequestMetadata,
}

impl PrioritizedRequest {
    /// Create new prioritized request
    pub fn new(
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        priority: RequestPriority,
        metadata: RequestMetadata,
    ) -> Self {
        let priority_value = Self::priority_to_value(priority);
        Self {
            operation_id,
            correlation_id,
            priority,
            effective_priority: priority_value,
            queued_at: Instant::now(),
            original_priority: priority,
            metadata,
        }
    }

    /// Convert RequestPriority to numeric value
    fn priority_to_value(priority: RequestPriority) -> u8 {
        match priority {
            RequestPriority::Critical => 100,
            RequestPriority::High => 80,
            RequestPriority::Normal => 50,
            RequestPriority::Low => 20,
            RequestPriority::Background => 1,
        }
    }

    /// Update effective priority based on aging
    pub fn update_effective_priority(&mut self, config: &PrioritizationConfig) {
        if matches!(config.aging_strategy, AgingStrategy::None) {
            return;
        }

        let age = self.queued_at.elapsed();
        let boost = self.calculate_age_boost(age, config);

        self.effective_priority =
            Self::priority_to_value(self.original_priority).saturating_add(boost);
    }

    /// Calculate priority boost based on age
    fn calculate_age_boost(&self, age: Duration, config: &PrioritizationConfig) -> u8 {
        let age_seconds = age.as_secs_f64();
        let threshold_seconds = config.starvation_prevention_timeout.as_secs_f64();

        match config.aging_strategy {
            AgingStrategy::None => 0,
            AgingStrategy::Linear => {
                if age_seconds > threshold_seconds {
                    let ratio = (age_seconds / threshold_seconds) as u8;
                    ratio.saturating_mul(config.age_priority_boost)
                } else {
                    0
                }
            },
            AgingStrategy::Exponential => {
                if age_seconds > threshold_seconds {
                    let ratio = (age_seconds / threshold_seconds).ln() as u8;
                    ratio.saturating_mul(config.age_priority_boost)
                } else {
                    0
                }
            },
            AgingStrategy::Threshold => {
                if age_seconds > threshold_seconds {
                    config.age_priority_boost.saturating_mul(10) // Significant boost
                } else {
                    0
                }
            },
        }
    }

    /// Get queue time duration
    #[inline]
    pub fn queue_time(&self) -> Duration {
        self.queued_at.elapsed()
    }

    /// Check if request is at risk of starvation
    #[inline]
    pub fn is_at_starvation_risk(&self, config: &PrioritizationConfig) -> bool {
        self.queue_time() > config.starvation_prevention_timeout
    }
}

/// Request metadata for prioritization decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetadata {
    /// Request type/category
    pub request_type: String,
    /// User/client identifier
    pub client_id: Option<String>,
    /// Estimated response size
    pub estimated_size: Option<usize>,
    /// Request deadline (if any)
    #[serde(
        serialize_with = "serialize_instant_option",
        deserialize_with = "deserialize_instant_option",
        skip_serializing_if = "Option::is_none",
        default
    )]
    pub deadline: Option<Instant>,
    /// Custom priority factors
    pub custom_factors: Vec<PriorityFactor>,
}

impl Default for RequestMetadata {
    fn default() -> Self {
        Self {
            request_type: "unknown".to_string(),
            client_id: None,
            estimated_size: None,
            deadline: None,
            custom_factors: Vec::new(),
        }
    }
}

/// Custom priority factors for advanced prioritization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityFactor {
    pub name: String,
    pub weight: f64,
    pub value: f64,
}

/// Priority queue implementation for HTTP requests
#[derive(Debug, Default)]
pub struct PriorityQueue {
    /// High-priority heap (max-heap by effective priority)
    high_priority_heap: BinaryHeap<PrioritizedRequestWrapper>,
    /// Normal priority queue (FIFO with aging)
    normal_priority_queue: VecDeque<PrioritizedRequest>,
    /// Background priority queue (FIFO)
    background_priority_queue: VecDeque<PrioritizedRequest>,
    /// Total requests queued
    total_queued: usize,
}

impl PriorityQueue {
    /// Add request to appropriate queue
    pub fn enqueue(
        &mut self,
        request: PrioritizedRequest,
        config: &PrioritizationConfig,
    ) -> Result<(), PrioritizationError> {
        // Check queue capacity
        if self.total_queued >= config.max_queue_size_per_priority * 3 {
            return Err(PrioritizationError::QueueFull);
        }

        match request.priority {
            RequestPriority::Critical | RequestPriority::High => {
                if self.high_priority_heap.len() >= config.max_queue_size_per_priority {
                    return Err(PrioritizationError::HighPriorityQueueFull);
                }
                self.high_priority_heap
                    .push(PrioritizedRequestWrapper(request));
            },
            RequestPriority::Normal | RequestPriority::Low => {
                if self.normal_priority_queue.len() >= config.max_queue_size_per_priority {
                    return Err(PrioritizationError::NormalPriorityQueueFull);
                }
                self.normal_priority_queue.push_back(request);
            },
            RequestPriority::Background => {
                if self.background_priority_queue.len() >= config.max_queue_size_per_priority {
                    return Err(PrioritizationError::BackgroundPriorityQueueFull);
                }
                self.background_priority_queue.push_back(request);
            },
        }

        self.total_queued += 1;
        Ok(())
    }

    /// Get next request to process
    pub fn dequeue(&mut self, config: &PrioritizationConfig) -> Option<PrioritizedRequest> {
        // Update priorities for aging
        self.update_priorities_for_aging(config);

        // Priority order: High -> Normal (with starvation prevention) -> Background

        // Check for high priority requests
        if let Some(wrapper) = self.high_priority_heap.pop() {
            self.total_queued -= 1;
            return Some(wrapper.0);
        }

        // Check for aged normal priority requests (starvation prevention)
        if let Some(pos) = self.find_aged_normal_request(config) {
            if pos < self.normal_priority_queue.len() {
                let request = self.normal_priority_queue.remove(pos).unwrap_or_else(|| {
                    warn!("Failed to remove aged request at position {}, queue may have been modified concurrently", pos);
                    // Return a fallback request if available
                    self.normal_priority_queue.pop_front().unwrap_or_else(|| {
                        panic!("Priority queue corrupted: no requests available despite finding aged request")
                    })
                });
                self.total_queued -= 1;
                return Some(request);
            }
        }

        // Regular normal priority requests
        if let Some(request) = self.normal_priority_queue.pop_front() {
            self.total_queued -= 1;
            return Some(request);
        }

        // Background requests (lowest priority)
        if let Some(request) = self.background_priority_queue.pop_front() {
            self.total_queued -= 1;
            return Some(request);
        }

        None
    }

    /// Find aged normal priority request that needs immediate processing
    fn find_aged_normal_request(&self, config: &PrioritizationConfig) -> Option<usize> {
        self.normal_priority_queue
            .iter()
            .enumerate()
            .find(|(_, request)| request.is_at_starvation_risk(config))
            .map(|(index, _)| index)
    }

    /// Update priorities for aging across all queues
    fn update_priorities_for_aging(&mut self, config: &PrioritizationConfig) {
        // Update normal priority queue
        for request in &mut self.normal_priority_queue {
            request.update_effective_priority(config);
        }

        // Update background priority queue
        for request in &mut self.background_priority_queue {
            request.update_effective_priority(config);
        }

        // Note: High priority heap requests don't need aging as they're already prioritized
    }

    /// Get queue size information
    pub fn queue_sizes(&self) -> QueueSizes {
        QueueSizes {
            high_priority: self.high_priority_heap.len(),
            normal_priority: self.normal_priority_queue.len(),
            background_priority: self.background_priority_queue.len(),
            total: self.total_queued,
        }
    }

    /// Check if any queue is empty
    pub fn is_empty(&self) -> bool {
        self.total_queued == 0
    }

    /// Get oldest request time (for starvation detection)
    pub fn oldest_request_age(&self) -> Option<Duration> {
        let mut oldest: Option<Instant> = None;

        // Check normal priority queue
        if let Some(front) = self.normal_priority_queue.front() {
            oldest = Some(oldest.map_or(front.queued_at, |o| o.min(front.queued_at)));
        }

        // Check background priority queue
        if let Some(front) = self.background_priority_queue.front() {
            oldest = Some(oldest.map_or(front.queued_at, |o| o.min(front.queued_at)));
        }

        oldest.map(|instant| instant.elapsed())
    }
}

/// Wrapper for heap ordering (max-heap by effective priority)
#[derive(Debug)]
struct PrioritizedRequestWrapper(PrioritizedRequest);

impl PartialEq for PrioritizedRequestWrapper {
    fn eq(&self, other: &Self) -> bool {
        self.0.effective_priority == other.0.effective_priority
    }
}

impl Eq for PrioritizedRequestWrapper {}

impl PartialOrd for PrioritizedRequestWrapper {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PrioritizedRequestWrapper {
    fn cmp(&self, other: &Self) -> Ordering {
        // Higher priority first, then FIFO for same priority
        self.0
            .effective_priority
            .cmp(&other.0.effective_priority)
            .then_with(|| other.0.queued_at.cmp(&self.0.queued_at)) // Reverse time order for FIFO
    }
}

/// Queue size information
#[derive(Debug, Clone)]
pub struct QueueSizes {
    pub high_priority: usize,
    pub normal_priority: usize,
    pub background_priority: usize,
    pub total: usize,
}

/// Request prioritization manager
#[derive(Debug, Resource)]
pub struct PrioritizationManager {
    /// Priority queue for requests
    pub queue: PriorityQueue,
    /// Prioritization statistics
    pub stats: PrioritizationStats,
    /// Rate limiting for high priority requests
    pub high_priority_rate_limiter: RateLimiter,
}

impl Default for PrioritizationManager {
    fn default() -> Self {
        Self {
            queue: PriorityQueue::default(),
            stats: PrioritizationStats::default(),
            high_priority_rate_limiter: RateLimiter::new(100.0), // 100 requests per second
        }
    }
}

impl PrioritizationManager {
    /// Add request to priority queue
    pub fn enqueue_request(
        &mut self,
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        priority: RequestPriority,
        metadata: RequestMetadata,
        config: &PrioritizationConfig,
    ) -> Result<(), PrioritizationError> {
        let request = PrioritizedRequest::new(operation_id, correlation_id, priority, metadata);

        // Check high priority rate limiting
        if matches!(priority, RequestPriority::Critical | RequestPriority::High)
            && !self.high_priority_rate_limiter.try_acquire()
        {
            self.stats.high_priority_rate_limited += 1;
            return Err(PrioritizationError::HighPriorityRateLimited);
        }

        self.queue.enqueue(request, config)?;

        // Update statistics
        match priority {
            RequestPriority::Critical => self.stats.critical_queued += 1,
            RequestPriority::High => self.stats.high_queued += 1,
            RequestPriority::Normal => self.stats.normal_queued += 1,
            RequestPriority::Low => self.stats.low_queued += 1,
            RequestPriority::Background => self.stats.background_queued += 1,
        }
        self.stats.total_queued += 1;

        debug!(
            "Request queued with priority {:?}: {:?}",
            priority, operation_id
        );
        Ok(())
    }

    /// Get next request to process
    pub fn dequeue_request(&mut self, config: &PrioritizationConfig) -> Option<PrioritizedRequest> {
        let request = self.queue.dequeue(config)?;

        // Update statistics
        self.stats.total_dequeued += 1;
        let queue_time = request.queue_time();
        self.stats.total_queue_time += queue_time;

        if queue_time > config.starvation_prevention_timeout {
            self.stats.starvation_prevented += 1;
        }

        debug!(
            "Request dequeued: {:?} (queued for {}ms)",
            request.operation_id,
            queue_time.as_millis()
        );

        Some(request)
    }

    /// Get current queue sizes
    pub fn get_queue_sizes(&self) -> QueueSizes {
        self.queue.queue_sizes()
    }

    /// Get prioritization statistics
    pub fn get_stats(&self) -> &PrioritizationStats {
        &self.stats
    }

    /// Check for potential starvation issues
    pub fn check_starvation_risk(&self) -> Option<Duration> {
        self.queue.oldest_request_age()
    }
}

/// Simple rate limiter for high priority requests
#[derive(Debug)]
pub struct RateLimiter {
    /// Requests per second limit
    rate_limit: f64,
    /// Last request time
    last_request: Option<Instant>,
    /// Token bucket counter
    tokens: f64,
}

impl RateLimiter {
    pub fn new(rate_limit: f64) -> Self {
        Self {
            rate_limit,
            last_request: None,
            tokens: rate_limit,
        }
    }

    /// Try to acquire a token
    pub fn try_acquire(&mut self) -> bool {
        let now = Instant::now();

        // Refill tokens based on elapsed time
        if let Some(last) = self.last_request {
            let elapsed = now.duration_since(last).as_secs_f64();
            self.tokens = (self.tokens + elapsed * self.rate_limit).min(self.rate_limit);
        }

        self.last_request = Some(now);

        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }
}

/// Prioritization statistics
#[derive(Debug, Default)]
pub struct PrioritizationStats {
    /// Requests queued by priority
    pub critical_queued: u64,
    pub high_queued: u64,
    pub normal_queued: u64,
    pub low_queued: u64,
    pub background_queued: u64,

    /// Total requests processed
    pub total_queued: u64,
    pub total_dequeued: u64,

    /// Queue performance
    pub total_queue_time: Duration,
    pub starvation_prevented: u64,
    pub high_priority_rate_limited: u64,
}

impl PrioritizationStats {
    /// Get average queue time
    #[inline]
    pub fn average_queue_time(&self) -> Duration {
        if self.total_dequeued > 0 {
            self.total_queue_time / self.total_dequeued as u32
        } else {
            Duration::ZERO
        }
    }

    /// Get queue utilization
    #[inline]
    pub fn queue_utilization(&self) -> f64 {
        if self.total_queued > 0 {
            self.total_dequeued as f64 / self.total_queued as f64
        } else {
            0.0
        }
    }
}

/// Prioritization errors
#[derive(Debug, thiserror::Error)]
pub enum PrioritizationError {
    #[error("Queue is full")]
    QueueFull,

    #[error("High priority queue is full")]
    HighPriorityQueueFull,

    #[error("Normal priority queue is full")]
    NormalPriorityQueueFull,

    #[error("Background priority queue is full")]
    BackgroundPriorityQueueFull,

    #[error("High priority request rate limited")]
    HighPriorityRateLimited,
}

/// Prioritization events
#[derive(Debug, Clone, Event)]
pub struct RequestQueued {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub priority: RequestPriority,
    pub queued_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct RequestDequeued {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub priority: RequestPriority,
    pub queue_time: Duration,
    pub dequeued_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct StarvationPrevented {
    pub operation_id: HttpOperationId,
    pub original_priority: RequestPriority,
    pub queue_time: Duration,
    pub prevented_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct PriorityQueueFull {
    pub priority: RequestPriority,
    pub queue_size: usize,
    pub rejected_at: Instant,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prioritized_request_creation() {
        let op_id = uuid::Uuid::new_v4();
        let corr_id = uuid::Uuid::new_v4();
        let metadata = RequestMetadata::default();

        let request = PrioritizedRequest::new(op_id, corr_id, RequestPriority::High, metadata);

        assert_eq!(request.operation_id, op_id);
        assert_eq!(request.priority, RequestPriority::High);
        assert_eq!(request.effective_priority, 80);
    }

    #[test]
    fn test_priority_queue_ordering() {
        let mut queue = PriorityQueue::default();
        let config = PrioritizationConfig::default();

        // Add requests with different priorities
        let normal_req = PrioritizedRequest::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            RequestPriority::Normal,
            RequestMetadata::default(),
        );

        let high_req = PrioritizedRequest::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            RequestPriority::High,
            RequestMetadata::default(),
        );

        queue.enqueue(normal_req, &config).unwrap();
        queue.enqueue(high_req, &config).unwrap();

        // High priority should come out first
        let first = queue.dequeue(&config).unwrap();
        assert_eq!(first.priority, RequestPriority::High);

        let second = queue.dequeue(&config).unwrap();
        assert_eq!(second.priority, RequestPriority::Normal);
    }

    #[test]
    fn test_aging_priority_boost() {
        let config = PrioritizationConfig {
            aging_strategy: AgingStrategy::Threshold,
            starvation_prevention_timeout: Duration::from_millis(100),
            age_priority_boost: 5,
            ..Default::default()
        };

        let mut request = PrioritizedRequest::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            RequestPriority::Low,
            RequestMetadata::default(),
        );

        // Simulate aging
        request.queued_at = Instant::now() - Duration::from_millis(200);
        request.update_effective_priority(&config);

        assert!(
            request.effective_priority
                > PrioritizedRequest::priority_to_value(RequestPriority::Low)
        );
    }

    #[test]
    fn test_rate_limiter() {
        let mut limiter = RateLimiter::new(2.0); // 2 requests per second

        // Should allow first request
        assert!(limiter.try_acquire());

        // Should allow second request
        assert!(limiter.try_acquire());

        // Should reject third request immediately
        assert!(!limiter.try_acquire());
    }

    #[test]
    fn test_prioritization_manager() {
        let mut manager = PrioritizationManager::default();
        let config = PrioritizationConfig::default();

        let op_id = uuid::Uuid::new_v4();
        let corr_id = uuid::Uuid::new_v4();

        // Enqueue request
        manager
            .enqueue_request(
                op_id,
                corr_id,
                RequestPriority::Normal,
                RequestMetadata::default(),
                &config,
            )
            .unwrap();

        assert_eq!(manager.stats.normal_queued, 1);
        assert_eq!(manager.stats.total_queued, 1);

        // Dequeue request
        let request = manager.dequeue_request(&config).unwrap();
        assert_eq!(request.operation_id, op_id);
        assert_eq!(manager.stats.total_dequeued, 1);
    }

    #[test]
    fn test_queue_full_error() {
        let mut queue = PriorityQueue::default();
        let config = PrioritizationConfig {
            max_queue_size_per_priority: 2,
            ..Default::default()
        };

        // Fill normal priority queue
        for _ in 0..2 {
            let request = PrioritizedRequest::new(
                uuid::Uuid::new_v4(),
                uuid::Uuid::new_v4(),
                RequestPriority::Normal,
                RequestMetadata::default(),
            );
            queue.enqueue(request, &config).unwrap();
        }

        // Third request should fail
        let request = PrioritizedRequest::new(
            uuid::Uuid::new_v4(),
            uuid::Uuid::new_v4(),
            RequestPriority::Normal,
            RequestMetadata::default(),
        );

        let result = queue.enqueue(request, &config);
        assert!(matches!(
            result,
            Err(PrioritizationError::NormalPriorityQueueFull)
        ));
    }
}
