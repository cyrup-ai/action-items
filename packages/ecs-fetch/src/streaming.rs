use std::collections::HashMap;
use std::time::{Duration, Instant};

use bevy::prelude::*;
use bytes::{Bytes, BytesMut};
use futures_util::StreamExt;
use reqwest::Response;
use thiserror::Error;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::events::{CorrelationId, HttpOperationId};

/// Streaming configuration
#[derive(Debug, Clone, Resource)]
pub struct StreamingConfig {
    /// Enable response streaming
    pub enable_streaming: bool,
    /// Chunk size for streaming (bytes)
    pub chunk_size: usize,
    /// Buffer size for stream channels
    pub buffer_size: usize,
    /// Timeout for individual chunks
    pub chunk_timeout: Duration,
    /// Maximum stream duration
    pub max_stream_duration: Duration,
    /// Backpressure threshold (bytes)
    pub backpressure_threshold: usize,
    /// Enable chunk compression
    pub enable_chunk_compression: bool,
}

impl Default for StreamingConfig {
    fn default() -> Self {
        Self {
            enable_streaming: true,
            chunk_size: 64 * 1024, // 64KB chunks
            buffer_size: 100,      // Buffer up to 100 chunks
            chunk_timeout: Duration::from_secs(30),
            max_stream_duration: Duration::from_secs(300), // 5 minutes max
            backpressure_threshold: 10 * 1024 * 1024,      // 10MB backpressure limit
            enable_chunk_compression: false,
        }
    }
}

/// Streaming response manager
#[derive(Debug, Default, Resource)]
pub struct StreamingManager {
    /// Active streams
    pub active_streams: HashMap<HttpOperationId, ActiveStream>,
    /// Streaming statistics
    pub stats: StreamingStats,
}

impl StreamingManager {
    /// Start streaming a response
    pub async fn start_stream(
        &mut self,
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        response: Response,
        config: &StreamingConfig,
    ) -> Result<StreamReceiver, StreamingError> {
        if !config.enable_streaming {
            return Err(StreamingError::StreamingDisabled);
        }

        // Create stream channel
        let (chunk_sender, chunk_receiver) = mpsc::channel(config.buffer_size);
        let (control_sender, control_receiver) = mpsc::channel(10);

        // Get response metadata
        let content_length = response.content_length();
        let headers = response.headers().clone();
        let status = response.status();

        // Create stream handler
        let stream_handler = StreamHandler::new(
            operation_id,
            correlation_id,
            response,
            chunk_sender,
            control_receiver,
            config.clone(),
        );

        // Spawn stream processing task
        let stream_task = tokio::spawn(stream_handler.process());

        // Create active stream record
        let active_stream = ActiveStream {
            operation_id,
            correlation_id,
            started_at: Instant::now(),
            content_length,
            bytes_streamed: 0,
            chunks_sent: 0,
            status,
            headers: headers.clone(),
            stream_task,
            control_sender,
            backpressure_active: false,
        };

        self.active_streams.insert(operation_id, active_stream);
        self.stats.streams_started += 1;

        debug!("Started streaming response: {:?}", operation_id);

        Ok(StreamReceiver {
            operation_id,
            correlation_id,
            chunk_receiver,
            content_length,
            headers,
            status,
            bytes_received: 0,
            chunks_received: 0,
        })
    }

    /// Cancel a stream
    pub async fn cancel_stream(
        &mut self,
        operation_id: HttpOperationId,
    ) -> Result<(), StreamingError> {
        if let Some(active_stream) = self.active_streams.remove(&operation_id) {
            // Send cancel signal
            let _ = active_stream
                .control_sender
                .send(StreamControl::Cancel)
                .await;

            // Abort the task
            active_stream.stream_task.abort();

            self.stats.streams_cancelled += 1;
            info!("Cancelled stream: {:?}", operation_id);
            Ok(())
        } else {
            Err(StreamingError::StreamNotFound)
        }
    }

    /// Update stream progress
    pub fn update_stream_progress(
        &mut self,
        operation_id: HttpOperationId,
        bytes_sent: u64,
        chunks_sent: u64,
    ) {
        if let Some(stream) = self.active_streams.get_mut(&operation_id) {
            stream.bytes_streamed = bytes_sent;
            stream.chunks_sent = chunks_sent;

            // Check for backpressure
            if let Some(content_length) = stream.content_length {
                let remaining_bytes = content_length - bytes_sent;
                stream.backpressure_active = remaining_bytes as usize > 0; // Simplified check
            }
        }
    }

    /// Complete a stream
    pub fn complete_stream(&mut self, operation_id: HttpOperationId) {
        if let Some(stream) = self.active_streams.remove(&operation_id) {
            let duration = stream.started_at.elapsed();
            self.stats.streams_completed += 1;
            self.stats.total_stream_duration += duration;
            self.stats.total_bytes_streamed += stream.bytes_streamed;

            info!(
                "Completed stream: {:?} ({} bytes in {}ms)",
                operation_id,
                stream.bytes_streamed,
                duration.as_millis()
            );
        }
    }

    /// Get stream statistics
    pub fn get_stats(&self) -> &StreamingStats {
        &self.stats
    }

    /// Get active stream count
    pub fn active_stream_count(&self) -> usize {
        self.active_streams.len()
    }

    /// Clean up expired streams
    pub fn cleanup_expired_streams(&mut self, config: &StreamingConfig) {
        let now = Instant::now();
        let mut expired_streams = Vec::new();

        for (operation_id, stream) in &self.active_streams {
            if now.duration_since(stream.started_at) > config.max_stream_duration {
                expired_streams.push(*operation_id);
            }
        }

        for operation_id in expired_streams {
            if let Some(stream) = self.active_streams.remove(&operation_id) {
                stream.stream_task.abort();
                self.stats.streams_expired += 1;
                warn!("Expired stream: {:?}", operation_id);
            }
        }
    }
}

/// Active stream information
#[derive(Debug)]
pub struct ActiveStream {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub started_at: Instant,
    pub content_length: Option<u64>,
    pub bytes_streamed: u64,
    pub chunks_sent: u64,
    pub status: reqwest::StatusCode,
    pub headers: reqwest::header::HeaderMap,
    pub stream_task: tokio::task::JoinHandle<Result<(), StreamingError>>,
    pub control_sender: mpsc::Sender<StreamControl>,
    pub backpressure_active: bool,
}

/// Stream control messages
#[derive(Debug, Clone)]
pub enum StreamControl {
    /// Pause streaming
    Pause,
    /// Resume streaming
    Resume,
    /// Cancel streaming
    Cancel,
    /// Adjust chunk size
    AdjustChunkSize(usize),
}

/// Stream chunk data
#[derive(Debug, Clone)]
pub struct StreamChunk {
    /// Chunk sequence number
    pub sequence: u64,
    /// Chunk data
    pub data: Bytes,
    /// Chunk timestamp
    pub timestamp: Instant,
    /// Is this the final chunk?
    pub is_final: bool,
    /// Chunk metadata
    pub metadata: ChunkMetadata,
}

/// Chunk metadata
#[derive(Debug, Clone, Default)]
pub struct ChunkMetadata {
    /// Chunk size before compression
    pub original_size: usize,
    /// Chunk size after compression (if applicable)
    pub compressed_size: Option<usize>,
    /// Chunk hash for integrity verification
    pub hash: Option<u64>,
    /// Chunk encoding
    pub encoding: Option<String>,
}

/// Stream receiver for consuming chunks
pub struct StreamReceiver {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub chunk_receiver: mpsc::Receiver<Result<StreamChunk, StreamingError>>,
    pub content_length: Option<u64>,
    pub headers: reqwest::header::HeaderMap,
    pub status: reqwest::StatusCode,
    pub bytes_received: u64,
    pub chunks_received: u64,
}

impl StreamReceiver {
    /// Receive next chunk
    pub async fn recv_chunk(&mut self) -> Option<Result<StreamChunk, StreamingError>> {
        match self.chunk_receiver.recv().await {
            Some(chunk_result) => {
                if let Ok(chunk) = &chunk_result {
                    self.bytes_received += chunk.data.len() as u64;
                    self.chunks_received += 1;
                }
                Some(chunk_result)
            },
            None => None,
        }
    }

    /// Collect all chunks into a single buffer
    pub async fn collect_all(mut self) -> Result<Bytes, StreamingError> {
        let mut buffer = BytesMut::new();

        while let Some(chunk_result) = self.recv_chunk().await {
            let chunk = chunk_result?;
            buffer.extend_from_slice(&chunk.data);

            if chunk.is_final {
                break;
            }
        }

        Ok(buffer.freeze())
    }

    /// Get progress information
    pub fn progress(&self) -> StreamProgress {
        StreamProgress {
            bytes_received: self.bytes_received,
            chunks_received: self.chunks_received,
            content_length: self.content_length,
            progress_ratio: self
                .content_length
                .map(|total| self.bytes_received as f64 / total as f64)
                .unwrap_or(0.0),
        }
    }
}

/// Stream progress information
#[derive(Debug, Clone)]
pub struct StreamProgress {
    pub bytes_received: u64,
    pub chunks_received: u64,
    pub content_length: Option<u64>,
    pub progress_ratio: f64,
}

/// Stream handler for processing response streams
struct StreamHandler {
    operation_id: HttpOperationId,
    correlation_id: CorrelationId,
    response: Response,
    chunk_sender: mpsc::Sender<Result<StreamChunk, StreamingError>>,
    control_receiver: mpsc::Receiver<StreamControl>,
    config: StreamingConfig,
    sequence: u64,
    paused: bool,
}

impl StreamHandler {
    fn new(
        operation_id: HttpOperationId,
        correlation_id: CorrelationId,
        response: Response,
        chunk_sender: mpsc::Sender<Result<StreamChunk, StreamingError>>,
        control_receiver: mpsc::Receiver<StreamControl>,
        config: StreamingConfig,
    ) -> Self {
        Self {
            operation_id,
            correlation_id,
            response,
            chunk_sender,
            control_receiver,
            config,
            sequence: 0,
            paused: false,
        }
    }

    /// Process the response stream
    async fn process(self) -> Result<(), StreamingError> {
        // Extract needed fields to avoid partial move
        let StreamHandler {
            operation_id,
            correlation_id,
            response,
            chunk_sender,
            mut control_receiver,
            config,
            mut sequence,
            mut paused,
        } = self;

        let mut stream = response.bytes_stream();
        let mut current_chunk_size = config.chunk_size;
        let mut chunk_buffer = BytesMut::new();

        debug!(
            "Starting stream processing with chunk size {}: {:?}",
            current_chunk_size, operation_id
        );

        loop {
            tokio::select! {
                // Handle control messages
                control_msg = control_receiver.recv() => {
                    match control_msg {
                        Some(StreamControl::Pause) => {
                            paused = true;
                            debug!("Stream paused: {:?}", operation_id);
                        }
                        Some(StreamControl::Resume) => {
                            paused = false;
                            debug!("Stream resumed: {:?}", operation_id);
                        }
                        Some(StreamControl::Cancel) => {
                            debug!("Stream cancelled: {:?}", operation_id);
                            return Ok(());
                        }
                        Some(StreamControl::AdjustChunkSize(new_size)) => {
                            current_chunk_size = new_size;
                            debug!("Stream chunk size adjusted to {}: {:?}", new_size, operation_id);

                            // Process any buffered data with new chunk size if buffer exceeds new size
                            if chunk_buffer.len() >= current_chunk_size {
                                let chunk_data = chunk_buffer.split_to(current_chunk_size).freeze();
                                if let Err(e) = Self::process_chunk_data(&chunk_sender, &operation_id, &correlation_id, &mut sequence, chunk_data).await {
                                    let _ = chunk_sender.send(Err(e)).await;
                                    return Err(StreamingError::ChunkProcessingError("Failed to process resized chunk".to_string()));
                                }
                            }
                        }
                        None => {
                            // Control channel closed
                            break;
                        }
                    }
                }

                // Process stream chunks
                chunk_result = stream.next(), if !paused => {
                    match chunk_result {
                        Some(Ok(raw_chunk)) => {
                            // Add data to buffer
                            chunk_buffer.extend_from_slice(&raw_chunk);

                            // Process buffer when it reaches the target chunk size
                            while chunk_buffer.len() >= current_chunk_size {
                                let chunk_data = chunk_buffer.split_to(current_chunk_size).freeze();
                                if let Err(e) = Self::process_chunk_data(&chunk_sender, &operation_id, &correlation_id, &mut sequence, chunk_data).await {
                                    let _ = chunk_sender.send(Err(e)).await;
                                    return Err(StreamingError::ChunkProcessingError("Failed to process buffered chunk".to_string()));
                                }
                            }
                        }
                        Some(Err(e)) => {
                            let error = StreamingError::NetworkError(e.to_string());
                            let _ = chunk_sender.send(Err(error.clone())).await;
                            return Err(error);
                        }
                        None => {
                            // Stream completed - process any remaining data in buffer
                            if !chunk_buffer.is_empty() {
                                let final_chunk_data = chunk_buffer.freeze();
                                if let Err(e) = Self::process_chunk_data(&chunk_sender, &operation_id, &correlation_id, &mut sequence, final_chunk_data).await {
                                    let _ = chunk_sender.send(Err(e)).await;
                                    return Err(StreamingError::ChunkProcessingError("Failed to process final buffered chunk".to_string()));
                                }
                            }
                            break;
                        }
                    }
                }

                // Timeout handling
                _ = tokio::time::sleep(config.chunk_timeout) => {
                    let error = StreamingError::ChunkTimeout;
                    let _ = chunk_sender.send(Err(error.clone())).await;
                    return Err(error);
                }
            }
        }

        // Send final chunk indicator
        let final_chunk = StreamChunk {
            sequence,
            data: Bytes::new(),
            timestamp: Instant::now(),
            is_final: true,
            metadata: ChunkMetadata::default(),
        };

        let _ = chunk_sender.send(Ok(final_chunk)).await;
        debug!("Stream processing completed: {:?}", operation_id);
        Ok(())
    }

    /// Process individual chunk
    async fn process_chunk_data(
        chunk_sender: &mpsc::Sender<Result<StreamChunk, StreamingError>>,
        operation_id: &HttpOperationId,
        correlation_id: &CorrelationId,
        sequence: &mut u64,
        chunk: Bytes,
    ) -> Result<(), StreamingError> {
        let timestamp = Instant::now();
        let original_size = chunk.len();

        debug!(
            "Processing chunk {} for operation {:?} (correlation: {:?}), size: {} bytes",
            *sequence, operation_id, correlation_id, original_size
        );

        // Apply chunk compression if enabled (simplified for now)
        let processed_chunk = chunk;

        let metadata = ChunkMetadata {
            original_size,
            compressed_size: None,
            hash: None, // Could add hash calculation here
            encoding: None,
        };

        let stream_chunk = StreamChunk {
            sequence: *sequence,
            data: processed_chunk,
            timestamp,
            is_final: false,
            metadata,
        };

        *sequence += 1;

        // Send chunk with backpressure handling
        match tokio::time::timeout(
            Duration::from_secs(30), // Default timeout
            chunk_sender.send(Ok(stream_chunk)),
        )
        .await
        {
            Ok(Ok(())) => {
                debug!(
                    "Successfully sent chunk {} for operation {:?} (correlation: {:?})",
                    *sequence - 1,
                    operation_id,
                    correlation_id
                );
                Ok(())
            },
            Ok(Err(_)) => {
                error!(
                    "Channel closed while sending chunk {} for operation {:?} (correlation: {:?})",
                    *sequence - 1,
                    operation_id,
                    correlation_id
                );
                Err(StreamingError::ChannelClosed)
            },
            Err(_) => {
                error!(
                    "Timeout sending chunk {} for operation {:?} (correlation: {:?})",
                    *sequence - 1,
                    operation_id,
                    correlation_id
                );
                Err(StreamingError::ChunkTimeout)
            },
        }
    }
}

/// Streaming statistics
#[derive(Debug, Default)]
pub struct StreamingStats {
    pub streams_started: u64,
    pub streams_completed: u64,
    pub streams_cancelled: u64,
    pub streams_expired: u64,
    pub streams_errored: u64,
    pub total_bytes_streamed: u64,
    pub total_stream_duration: Duration,
}

impl StreamingStats {
    /// Get average streaming rate (bytes per second)
    #[inline]
    pub fn average_streaming_rate(&self) -> f64 {
        if self.total_stream_duration.as_secs() > 0 {
            self.total_bytes_streamed as f64 / self.total_stream_duration.as_secs() as f64
        } else {
            0.0
        }
    }

    /// Get stream success rate
    #[inline]
    pub fn success_rate(&self) -> f64 {
        if self.streams_started > 0 {
            self.streams_completed as f64 / self.streams_started as f64
        } else {
            0.0
        }
    }

    /// Get active stream count
    #[inline]
    pub fn active_streams(&self) -> u64 {
        self.streams_started
            - self.streams_completed
            - self.streams_cancelled
            - self.streams_expired
            - self.streams_errored
    }
}

/// Streaming errors
#[derive(Debug, Error, Clone)]
pub enum StreamingError {
    #[error("Streaming is disabled")]
    StreamingDisabled,

    #[error("Stream not found")]
    StreamNotFound,

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Chunk processing error: {0}")]
    ChunkProcessingError(String),

    #[error("Chunk timeout")]
    ChunkTimeout,

    #[error("Channel closed")]
    ChannelClosed,

    #[error("Backpressure limit exceeded")]
    BackpressureLimitExceeded,

    #[error("Stream expired")]
    StreamExpired,

    #[error("Invalid chunk sequence")]
    InvalidChunkSequence,
}

/// Streaming events
#[derive(Debug, Clone, Event)]
pub struct StreamStarted {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub content_length: Option<u64>,
    pub started_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct StreamChunkReceived {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub sequence: u64,
    pub chunk_size: usize,
    pub bytes_received_total: u64,
    pub received_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct StreamCompleted {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub total_bytes: u64,
    pub total_chunks: u64,
    pub duration: Duration,
    pub completed_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct StreamErrorOccurred {
    pub operation_id: HttpOperationId,
    pub correlation_id: CorrelationId,
    pub error: String,
    pub occurred_at: Instant,
}

#[derive(Debug, Clone, Event)]
pub struct StreamBackpressureDetected {
    pub operation_id: HttpOperationId,
    pub buffer_size: usize,
    pub threshold: usize,
    pub detected_at: Instant,
}

/// System for cleaning up expired streams
pub fn stream_cleanup_system(
    mut streaming_manager: ResMut<StreamingManager>,
    config: Res<StreamingConfig>,
) {
    streaming_manager.cleanup_expired_streams(&config);
}

#[cfg(test)]
mod tests {
    use super::*;

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
        };

        assert_eq!(metadata.original_size, 1024);
        assert_eq!(metadata.compressed_size, Some(512));
        assert_eq!(metadata.hash, Some(12345));
        assert_eq!(metadata.encoding, Some("gzip".to_string()));
    }
}
