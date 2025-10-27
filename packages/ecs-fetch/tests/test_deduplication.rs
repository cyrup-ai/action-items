//! Tests for deduplication.rs

use action_items_ecs_fetch::deduplication::*;
use bytes::Bytes;
use reqwest::Method;

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
        &DeduplicationRequest {
            method: &Method::GET,
            url: "https://api.example.com/users",
            headers: None,
            body: None,
            operation_id: op_id_1,
            correlation_id: corr_id_1,
            requester: "requester1",
        },
        &config,
    );

    assert!(matches!(result1, DeduplicationResult::NotDuplicate));
    assert_eq!(manager.stats.unique_requests, 1);

    // Second identical request should be a duplicate
    let result2 = manager.check_and_handle_duplicate(
        &DeduplicationRequest {
            method: &Method::GET,
            url: "https://api.example.com/users",
            headers: None,
            body: None,
            operation_id: op_id_2,
            correlation_id: corr_id_2,
            requester: "requester2",
        },
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
        &DeduplicationRequest {
            method: &Method::GET,
            url: "https://api.example.com/users",
            headers: None,
            body: None,
            operation_id: op_id,
            correlation_id: corr_id,
            requester: "requester",
        },
        &config,
    );

    // Add duplicate
    manager.check_and_handle_duplicate(
        &DeduplicationRequest {
            method: &Method::GET,
            url: "https://api.example.com/users",
            headers: None,
            body: None,
            operation_id: uuid::Uuid::new_v4(),
            correlation_id: uuid::Uuid::new_v4(),
            requester: "requester2",
        },
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
