//! Compression types and data structures
//!
//! Following proper Bevy ECS service patterns from ARCHITECTURE.md

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};

use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Compression algorithm variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CompressionAlgorithm {
    /// Gzip compression (widely compatible)
    Gzip,
    /// Deflate compression (raw deflate)
    Deflate,
    /// LZ4 fast compression
    Lz4,
    /// Zstd compression (excellent ratio and speed)
    Zstd,
    /// Brotli compression (high quality)
    Brotli,
    /// Snappy compression (very fast)
    Snappy,
}

impl Default for CompressionAlgorithm {
    fn default() -> Self {
        Self::Lz4
    }
}

/// Compression configuration
#[derive(Debug, Clone)]
pub struct CompressionConfig {
    /// Minimum size threshold for compression (bytes)
    pub min_size_threshold: usize,
    /// Compression level (algorithm-specific)
    pub compression_level: u32,
    /// Default algorithm to use
    pub default_algorithm: CompressionAlgorithm,
    /// Maximum buffer pool size
    pub max_pool_size: usize,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            min_size_threshold: 1024, // 1KB
            compression_level: 6,     // Balanced compression
            default_algorithm: CompressionAlgorithm::Lz4,
            max_pool_size: 64, // 64 buffers max
        }
    }
}

/// Optimized shared compression statistics using relaxed atomic operations
#[derive(Debug, Clone, Resource)]
pub struct CompressionStats {
    inner: Arc<CompressionStatsInner>,
}

#[derive(Debug)]
struct CompressionStatsInner {
    total_compressed: AtomicU64,
    total_decompressed: AtomicU64,
    bytes_saved: AtomicU64,
    compression_time_ns: AtomicU64,
    decompression_time_ns: AtomicU64,
    total_input_bytes: AtomicU64,
    total_output_bytes: AtomicU64,
    compression_ratio_sum: AtomicU64, // Sum of ratios * 1000 for precision
}

impl CompressionStats {
    /// Create new shared statistics
    pub fn new() -> Self {
        Self {
            inner: Arc::new(CompressionStatsInner {
                total_compressed: AtomicU64::new(0),
                total_decompressed: AtomicU64::new(0),
                bytes_saved: AtomicU64::new(0),
                compression_time_ns: AtomicU64::new(0),
                decompression_time_ns: AtomicU64::new(0),
                total_input_bytes: AtomicU64::new(0),
                total_output_bytes: AtomicU64::new(0),
                compression_ratio_sum: AtomicU64::new(0),
            }),
        }
    }

    /// Record compression operation with optimized atomic operations
    #[inline]
    pub fn record_compression(&self, original_size: u64, compressed_size: u64, duration_ns: u64) {
        // Use relaxed ordering for performance - statistics don't need strict consistency
        self.inner.total_compressed.fetch_add(1, Ordering::Relaxed);
        self.inner
            .total_input_bytes
            .fetch_add(original_size, Ordering::Relaxed);
        self.inner
            .total_output_bytes
            .fetch_add(compressed_size, Ordering::Relaxed);
        self.inner
            .compression_time_ns
            .fetch_add(duration_ns, Ordering::Relaxed);

        if original_size > compressed_size {
            self.inner
                .bytes_saved
                .fetch_add(original_size - compressed_size, Ordering::Relaxed);
        }

        // Store compression ratio * 1000 for precision without floating point
        if original_size > 0 {
            let ratio_scaled = (compressed_size * 1000) / original_size;
            self.inner
                .compression_ratio_sum
                .fetch_add(ratio_scaled, Ordering::Relaxed);
        }
    }

    /// Record decompression operation with optimized atomic operations
    #[inline]
    pub fn record_decompression(&self, duration_ns: u64) {
        self.inner
            .total_decompressed
            .fetch_add(1, Ordering::Relaxed);
        self.inner
            .decompression_time_ns
            .fetch_add(duration_ns, Ordering::Relaxed);
    }

    /// Get total compressions performed
    #[inline]
    pub fn total_compressed(&self) -> u64 {
        self.inner.total_compressed.load(Ordering::Relaxed)
    }

    /// Get total decompressions performed
    #[inline]
    pub fn total_decompressed(&self) -> u64 {
        self.inner.total_decompressed.load(Ordering::Relaxed)
    }

    /// Get total bytes saved through compression
    #[inline]
    pub fn bytes_saved(&self) -> u64 {
        self.inner.bytes_saved.load(Ordering::Relaxed)
    }

    /// Get total input bytes processed
    #[inline]
    pub fn total_input_bytes(&self) -> u64 {
        self.inner.total_input_bytes.load(Ordering::Relaxed)
    }

    /// Get total output bytes produced
    #[inline]
    pub fn total_output_bytes(&self) -> u64 {
        self.inner.total_output_bytes.load(Ordering::Relaxed)
    }

    /// Get average compression time in nanoseconds
    #[inline]
    pub fn avg_compression_time_ns(&self) -> u64 {
        let total = self.inner.total_compressed.load(Ordering::Relaxed);
        if total > 0 {
            self.inner.compression_time_ns.load(Ordering::Relaxed) / total
        } else {
            0
        }
    }

    /// Get average decompression time in nanoseconds
    #[inline]
    pub fn avg_decompression_time_ns(&self) -> u64 {
        let total = self.inner.total_decompressed.load(Ordering::Relaxed);
        if total > 0 {
            self.inner.decompression_time_ns.load(Ordering::Relaxed) / total
        } else {
            0
        }
    }

    /// Get average compression ratio (0.0 to 1.0)
    #[inline]
    pub fn avg_compression_ratio(&self) -> f64 {
        let total = self.inner.total_compressed.load(Ordering::Relaxed);
        if total > 0 {
            let ratio_sum = self.inner.compression_ratio_sum.load(Ordering::Relaxed);
            (ratio_sum as f64) / (total as f64 * 1000.0)
        } else {
            0.0
        }
    }

    /// Get overall compression efficiency (bytes saved / total input)
    #[inline]
    pub fn compression_efficiency(&self) -> f64 {
        let total_input = self.inner.total_input_bytes.load(Ordering::Relaxed);
        if total_input > 0 {
            let bytes_saved = self.inner.bytes_saved.load(Ordering::Relaxed);
            (bytes_saved as f64) / (total_input as f64)
        } else {
            0.0
        }
    }

    /// Get compression throughput in bytes per second
    #[inline]
    pub fn compression_throughput_bps(&self) -> f64 {
        let total_time_ns = self.inner.compression_time_ns.load(Ordering::Relaxed);
        if total_time_ns > 0 {
            let total_bytes = self.inner.total_input_bytes.load(Ordering::Relaxed);
            (total_bytes as f64 * 1_000_000_000.0) / (total_time_ns as f64)
        } else {
            0.0
        }
    }

    /// Get decompression throughput in bytes per second
    #[inline]
    pub fn decompression_throughput_bps(&self) -> f64 {
        let total_time_ns = self.inner.decompression_time_ns.load(Ordering::Relaxed);
        if total_time_ns > 0 {
            let total_bytes = self.inner.total_output_bytes.load(Ordering::Relaxed);
            (total_bytes as f64 * 1_000_000_000.0) / (total_time_ns as f64)
        } else {
            0.0
        }
    }
}

impl Default for CompressionStats {
    fn default() -> Self {
        Self::new()
    }
}

/// Compressed data container with metadata
#[derive(Debug, Clone)]
pub struct CompressedData {
    /// Compressed bytes
    pub data: Vec<u8>,
    /// Original uncompressed size
    pub original_size: usize,
    /// Compression algorithm used
    pub algorithm: CompressionAlgorithm,
    /// Compression ratio (compressed_size / original_size)
    pub compression_ratio: f64,
}

impl CompressedData {
    /// Create new compressed data container
    pub fn new(data: Vec<u8>, original_size: usize, algorithm: CompressionAlgorithm) -> Self {
        let compression_ratio = if original_size > 0 {
            data.len() as f64 / original_size as f64
        } else {
            1.0
        };

        Self {
            data,
            original_size,
            algorithm,
            compression_ratio,
        }
    }

    /// Get compressed size
    #[inline]
    pub fn compressed_size(&self) -> usize {
        self.data.len()
    }

    /// Check if compression was beneficial (ratio < 0.95)
    #[inline]
    pub fn is_compressed(&self) -> bool {
        self.compression_ratio < 0.95
    }

    /// Get bytes saved through compression
    #[inline]
    pub fn bytes_saved(&self) -> usize {
        if self.original_size > self.data.len() {
            self.original_size - self.data.len()
        } else {
            0
        }
    }
}

/// Enhanced compression-related errors
#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("Serialization error: {0}")]
    SerializationError(String),
    #[error("Invalid compression format")]
    InvalidFormat,
    #[error("Buffer pool exhausted")]
    PoolExhausted,
    #[error("Internal error: {0}")]
    InternalError(String),
    #[error("Size limit exceeded")]
    SizeLimitExceeded,
}

/// Bevy events following ARCHITECTURE.md request/response pattern
#[derive(Event, Debug)]
pub enum CompressionRequest {
    Compress {
        data: Vec<u8>,
        requester: Entity,
    },
    Decompress {
        compressed_data: CompressedData,
        requester: Entity,
    },
}

/// Compression response events
#[derive(Event, Debug)]
pub enum CompressionResponse {
    CompressResult {
        requester: Entity,
        result: Result<CompressedData, CompressionError>,
    },
    DecompressResult {
        requester: Entity,
        result: Result<Vec<u8>, CompressionError>,
    },
}
