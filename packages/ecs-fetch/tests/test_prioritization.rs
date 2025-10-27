//! Tests for prioritization.rs

use action_items_ecs_fetch::prioritization::*;
use std::time::{Duration, Instant};

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
