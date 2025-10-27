//! Tests for streaming.rs

use action_items_ecs_fetch::streaming::*;
use bytes::Bytes;
use std::time::{Duration, Instant};

#[test]
fn test_streaming_config_defaults() {
    let config = StreamingConfig::default();
    assert!(config.enable_streaming);
    assert_eq!(config.chunk_size, 64 * 1024);
    assert_eq!(config.buffer_size, 100);
}

#[test]
fn test_stream_chunk_creation() {
    let chunk = StreamChunk {
        sequence: 1,
        data: Bytes::from("test data"),
        timestamp: Instant::now(),
        is_final: false,
        metadata: ChunkMetadata::default(),
    };

    assert_eq!(chunk.sequence, 1);
    assert_eq!(chunk.data, Bytes::from("test data"));
    assert!(!chunk.is_final);
}

#[test]
fn test_streaming_stats() {
    let stats = StreamingStats {
        streams_started: 100,
        streams_completed: 90,
        streams_cancelled: 5,
        streams_errored: 3,
        total_bytes_streamed: 1000000,
        total_stream_duration: Duration::from_secs(100),
        ..Default::default()
    };

    assert_eq!(stats.success_rate(), 0.9);
    assert_eq!(stats.average_streaming_rate(), 10000.0);
    assert_eq!(stats.active_streams(), 2);
}

#[test]
fn test_stream_progress() {
    let progress = StreamProgress {
        bytes_received: 500,
        chunks_received: 10,
        content_length: Some(1000),
        progress_ratio: 0.5,
    };

    assert_eq!(progress.bytes_received, 500);
    assert_eq!(progress.progress_ratio, 0.5);
}

#[test]
fn test_chunk_metadata() {
    let metadata = ChunkMetadata {
        original_size: 1024,
        compressed_size: Some(512),
        hash: Some(12345),
        encoding: Some("gzip".to_string()),
        decompressed: false,
    };

    assert_eq!(metadata.original_size, 1024);
    assert_eq!(metadata.compressed_size, Some(512));
    assert_eq!(metadata.hash, Some(12345));
    assert_eq!(metadata.encoding, Some("gzip".to_string()));
}
