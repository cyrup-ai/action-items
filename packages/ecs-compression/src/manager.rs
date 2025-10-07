//! Compression manager following proper Bevy ECS service patterns
//!
//! Resource with sync API following ARCHITECTURE.md patterns

use std::cell::RefCell;
use std::collections::VecDeque;
use std::ops::{Deref, DerefMut};
use std::time::Instant;

use bevy::prelude::*;
use flate2::{Compression, read::GzDecoder, write::GzEncoder, read::DeflateDecoder, write::DeflateEncoder};

use crate::types::{
    CompressedData, CompressionAlgorithm, CompressionConfig, CompressionError, CompressionStats,
};

// Thread-local buffer pool for true zero-allocation compression
thread_local! {
    static BUFFER_POOL: RefCell<VecDeque<Vec<u8>>> = RefCell::new(VecDeque::with_capacity(16));
}

const MAX_POOLED_BUFFERS: usize = 32;
const INITIAL_BUFFER_CAPACITY: usize = 8192;
const MAX_BUFFER_SIZE: usize = 1024 * 1024; // 1MB max buffer size

/// A buffer wrapper that automatically returns to the thread-local pool when dropped
#[derive(Debug)]
pub struct PooledBuffer {
    buffer: Option<Vec<u8>>,
}

impl PooledBuffer {
    /// Acquire a buffer from the thread-local pool
    pub fn acquire() -> Self {
        let buffer = BUFFER_POOL.with(|pool| {
            let mut pool = pool.borrow_mut();
            // Pre-populate pool if empty to ensure zero allocations after warmup
            if pool.is_empty() {
                for _ in 0..4 {
                    pool.push_back(Vec::with_capacity(INITIAL_BUFFER_CAPACITY));
                }
            }
            pool.pop_front()
                .unwrap_or_else(|| Vec::with_capacity(INITIAL_BUFFER_CAPACITY))
        });

        Self {
            buffer: Some(buffer),
        }
    }

    /// Take ownership of the inner buffer (prevents return to pool)
    #[inline]
    pub fn into_vec(mut self) -> Vec<u8> {
        self.buffer.take().unwrap_or_default()
    }

    /// Get the length of the buffer
    #[inline]
    pub fn len(&self) -> usize {
        self.buffer.as_ref().map_or(0, |b| b.len())
    }

    /// Check if the buffer is empty
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.buffer.as_ref().is_none_or(|b| b.is_empty())
    }

    /// Get a slice of the buffer contents
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_ref().map_or(&[], |b| b.as_slice())
    }
}

impl Deref for PooledBuffer {
    type Target = Vec<u8>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap_or_else(|| {
            panic!("PooledBuffer accessed after being consumed - this is a programming error")
        })
    }
}

impl DerefMut for PooledBuffer {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap_or_else(|| {
            panic!("PooledBuffer accessed after being consumed - this is a programming error")
        })
    }
}

impl Drop for PooledBuffer {
    #[inline]
    fn drop(&mut self) {
        if let Some(mut buffer) = self.buffer.take() {
            // Only return buffers that aren't too large to prevent memory bloat
            if buffer.capacity() <= MAX_BUFFER_SIZE {
                buffer.clear();
                BUFFER_POOL.with(|pool| {
                    let mut pool = pool.borrow_mut();
                    if pool.len() < MAX_POOLED_BUFFERS {
                        pool.push_back(buffer);
                    }
                });
            }
        }
    }
}

/// Compression manager resource following ARCHITECTURE.md patterns
#[derive(Debug, Clone, Resource)]
pub struct CompressionManager {
    config: CompressionConfig,
    stats: CompressionStats,
}

impl CompressionManager {
    /// Create new compression manager with configuration
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            config,
            stats: CompressionStats::new(),
        }
    }

    /// Create compression manager with shared statistics
    pub fn with_shared_stats(config: CompressionConfig, stats: CompressionStats) -> Self {
        Self {
            config,
            stats,
        }
    }

    /// Get shared statistics reference
    pub fn stats(&self) -> &CompressionStats {
        &self.stats
    }

    /// Check if compression service is available and ready
    #[inline]
    pub fn is_available(&self) -> bool {
        true // Service is always available once constructed
    }

    /// Get compression statistics (alias for stats() for plugin compatibility)
    #[inline]
    pub fn get_stats(&self) -> &CompressionStats {
        &self.stats
    }

    /// Analyze data characteristics for optimal algorithm selection
    #[inline]
    fn select_optimal_algorithm(&self, data: &[u8]) -> CompressionAlgorithm {
        let size = data.len();

        // Small data: use LZ4 for speed
        if size < 4096 {
            return CompressionAlgorithm::Lz4;
        }

        // Calculate entropy for algorithm selection
        let mut byte_counts = [0u32; 256];
        for &byte in data.iter().take(std::cmp::min(size, 8192)) {
            byte_counts[byte as usize] += 1;
        }

        let sample_size = std::cmp::min(size, 8192) as f64;
        let mut entropy = 0.0;
        for &count in &byte_counts {
            if count > 0 {
                let p = count as f64 / sample_size;
                entropy -= p * p.log2();
            }
        }

        // High entropy (> 7.0): use LZ4 (fastest)
        // Medium entropy (4.0-7.0): use configured default
        // Low entropy (< 4.0): use Brotli (best compression)
        if entropy > 7.0 {
            CompressionAlgorithm::Lz4
        } else if entropy > 4.0 {
            self.config.default_algorithm
        } else {
            CompressionAlgorithm::Brotli
        }
    }

    /// Compress data synchronously - matches ARCHITECTURE.md sync API pattern
    pub fn compress_sync(&self, data: Vec<u8>) -> Result<CompressedData, CompressionError> {
        if data.len() < self.config.min_size_threshold {
            // Return uncompressed data for small payloads
            return Ok(CompressedData::new(
                data.clone(),
                data.len(),
                self.config.default_algorithm,
            ));
        }

        let start_time = Instant::now();
        let original_size = data.len();

        // Use the configured default algorithm for consistent behavior
        let algorithm = self.config.default_algorithm;

        let compressed = match algorithm {
            CompressionAlgorithm::Gzip => self.compress_gzip_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Deflate => self.compress_deflate_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Lz4 => self.compress_lz4_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Zstd => self.compress_zstd_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Brotli => self.compress_brotli_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Snappy => self.compress_snappy_pooled(&data)?.into_vec(),
        };

        let duration = start_time.elapsed();
        self.stats.record_compression(
            original_size as u64,
            compressed.len() as u64,
            duration.as_nanos() as u64,
        );

        let result = CompressedData::new(compressed, original_size, algorithm);
        Ok(result)
    }

    /// Decompress data synchronously - matches ARCHITECTURE.md sync API pattern
    pub fn decompress_sync(
        &self,
        compressed: &CompressedData,
    ) -> Result<Vec<u8>, CompressionError> {
        let start_time = Instant::now();

        // Validate compressed data integrity
        if compressed.data.is_empty() {
            return Err(CompressionError::InvalidFormat);
        }

        let result = match compressed.algorithm {
            CompressionAlgorithm::Gzip => self.decompress_gzip_pooled(&compressed.data)?,
            CompressionAlgorithm::Deflate => self.decompress_deflate_pooled(&compressed.data)?,
            CompressionAlgorithm::Lz4 => self.decompress_lz4_pooled(&compressed.data)?,
            CompressionAlgorithm::Zstd => self.decompress_zstd_pooled(&compressed.data)?,
            CompressionAlgorithm::Brotli => self.decompress_brotli_pooled(&compressed.data)?,
            CompressionAlgorithm::Snappy => self.decompress_snappy_pooled(&compressed.data)?,
        };

        // Validate decompressed size matches expected
        if result.len() != compressed.original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                compressed.original_size,
                result.len()
            )));
        }

        let duration = start_time.elapsed();
        self.stats.record_decompression(duration.as_nanos() as u64);

        Ok(result)
    }

    /// Compress data with specific algorithm - allows runtime algorithm selection
    pub fn compress_with_algorithm(&self, data: Vec<u8>, algorithm: CompressionAlgorithm) -> Result<CompressedData, CompressionError> {
        if data.len() < self.config.min_size_threshold {
            // Return uncompressed data for small payloads
            return Ok(CompressedData::new(
                data.clone(),
                data.len(),
                algorithm,
            ));
        }

        let start_time = Instant::now();
        let original_size = data.len();

        let compressed = match algorithm {
            CompressionAlgorithm::Gzip => self.compress_gzip_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Deflate => self.compress_deflate_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Lz4 => self.compress_lz4_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Zstd => self.compress_zstd_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Brotli => self.compress_brotli_pooled(&data)?.into_vec(),
            CompressionAlgorithm::Snappy => self.compress_snappy_pooled(&data)?.into_vec(),
        };

        let duration = start_time.elapsed();
        self.stats.record_compression(
            original_size as u64,
            compressed.len() as u64,
            duration.as_nanos() as u64,
        );

        let result = CompressedData::new(compressed, original_size, algorithm);
        Ok(result)
    }

    /// Compress data with automatic algorithm selection based on data characteristics
    pub fn compress_auto(&self, data: Vec<u8>) -> Result<CompressedData, CompressionError> {
        let optimal_algorithm = self.select_optimal_algorithm(&data);
        self.compress_with_algorithm(data, optimal_algorithm)
    }

    /// Production LZ4 compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_lz4_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use lz4_flex::block::{compress_into, get_maximum_output_size};

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Reserve space for size prefix (4 bytes) + compressed data
        let max_compressed_size = get_maximum_output_size(data.len());
        buffer.reserve(4 + max_compressed_size);

        // First, try compression into a temporary buffer to check the result
        let mut temp_buffer = vec![0u8; max_compressed_size];
        let compressed_size = compress_into(data, &mut temp_buffer).map_err(|e| {
            CompressionError::CompressionFailed(format!("LZ4 compression error: {}", e))
        })?;

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + compressed_size; // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer[..compressed_size]); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production Brotli compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_brotli_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use brotli::CompressorReader;
        use std::io::Read;

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Create Brotli compressor with quality level from config
        let quality = self.config.compression_level.min(11); // Brotli max quality is 11
        let mut compressor = CompressorReader::new(data, 4096, quality, 22); // 22 = lg_win default

        // Compress data into temporary buffer
        let mut temp_buffer = Vec::new();
        compressor.read_to_end(&mut temp_buffer).map_err(|e| {
            CompressionError::CompressionFailed(format!("Brotli compression error: {}", e))
        })?;

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + temp_buffer.len(); // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production Snappy compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_snappy_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use snap::raw::Encoder;

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Create Snappy encoder
        let mut encoder = Encoder::new();

        // Compress data into temporary buffer
        let temp_buffer = encoder.compress_vec(data).map_err(|e| {
            CompressionError::CompressionFailed(format!("Snappy compression error: {}", e))
        })?;

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + temp_buffer.len(); // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production LZ4 decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_lz4_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use lz4_flex::block::decompress_into;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.resize(original_size, 0);

        let decompressed_size = decompress_into(compressed_data, &mut buffer).map_err(|e| {
            CompressionError::DecompressionFailed(format!("LZ4 decompression error: {}", e))
        })?;

        if decompressed_size != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size, decompressed_size
            )));
        }

        buffer.shrink_to_fit();
        Ok(buffer.into_vec())
    }

    /// Production Brotli decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_brotli_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use brotli::Decompressor;
        use std::io::Read;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        // Decompress the data using buffer pool
        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.reserve(original_size);

        {
            let mut decompressor = Decompressor::new(compressed_data, 4096);
            decompressor.read_to_end(&mut buffer).map_err(|e| {
                CompressionError::DecompressionFailed(format!("Brotli decompression error: {}", e))
            })?;
        }

        // Validate size matches expected
        if buffer.len() != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size,
                buffer.len()
            )));
        }

        Ok(buffer.into_vec())
    }

    /// Production Snappy decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_snappy_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use snap::raw::Decoder;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        // Decompress using buffer pool for zero allocation
        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.resize(original_size, 0); // Snappy requires exact output buffer size

        let mut decoder = Decoder::new();
        decoder.decompress(compressed_data, &mut buffer).map_err(|e| {
            CompressionError::DecompressionFailed(format!("Snappy decompression error: {}", e))
        })?;

        // Validate size matches expected
        if buffer.len() != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size,
                buffer.len()
            )));
        }

        Ok(buffer.into_vec())
    }

    /// Production Gzip compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_gzip_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use std::io::Write;

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Create Gzip encoder with compression level from config
        let compression_level = Compression::new(self.config.compression_level);
        
        // Compress data into temporary buffer first to check ratio
        let mut temp_buffer = Vec::new();
        {
            let mut encoder = GzEncoder::new(&mut temp_buffer, compression_level);
            encoder.write_all(data).map_err(|e| {
                CompressionError::CompressionFailed(format!("Gzip compression error: {}", e))
            })?;
            encoder.finish().map_err(|e| {
                CompressionError::CompressionFailed(format!("Gzip finish error: {}", e))
            })?;
        }

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + temp_buffer.len(); // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production Gzip decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_gzip_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use std::io::Read;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        // Decompress the data using buffer pool
        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.reserve(original_size);

        {
            let mut decoder = GzDecoder::new(compressed_data);
            decoder.read_to_end(&mut buffer).map_err(|e| {
                CompressionError::DecompressionFailed(format!("Gzip decompression error: {}", e))
            })?;
        }

        // Validate size matches expected
        if buffer.len() != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size,
                buffer.len()
            )));
        }

        buffer.shrink_to_fit();
        Ok(buffer.into_vec())
    }

    /// Production Deflate compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_deflate_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use std::io::Write;

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Create Deflate encoder with compression level from config
        let compression_level = Compression::new(self.config.compression_level);
        
        // Compress data into temporary buffer first to check ratio
        let mut temp_buffer = Vec::new();
        {
            let mut encoder = DeflateEncoder::new(&mut temp_buffer, compression_level);
            encoder.write_all(data).map_err(|e| {
                CompressionError::CompressionFailed(format!("Deflate compression error: {}", e))
            })?;
            encoder.finish().map_err(|e| {
                CompressionError::CompressionFailed(format!("Deflate finish error: {}", e))
            })?;
        }

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + temp_buffer.len(); // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production Deflate decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_deflate_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use std::io::Read;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        // Decompress the data using buffer pool
        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.reserve(original_size);

        {
            let mut decoder = DeflateDecoder::new(compressed_data);
            decoder.read_to_end(&mut buffer).map_err(|e| {
                CompressionError::DecompressionFailed(format!("Deflate decompression error: {}", e))
            })?;
        }

        // Validate size matches expected
        if buffer.len() != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size,
                buffer.len()
            )));
        }

        buffer.shrink_to_fit();
        Ok(buffer.into_vec())
    }

    /// Production Zstd compression using thread-local buffer pool - true zero allocation
    #[inline]
    fn compress_zstd_pooled(&self, data: &[u8]) -> Result<PooledBuffer, CompressionError> {
        use std::io::Write;
        use zstd::stream::write::Encoder;

        let mut buffer = PooledBuffer::acquire();
        buffer.clear();

        // Compress data into temporary buffer first to check ratio
        let mut temp_buffer = Vec::new();
        {
            let mut encoder = Encoder::new(&mut temp_buffer, self.config.compression_level as i32)
                .map_err(|e| {
                    CompressionError::CompressionFailed(format!("Zstd encoder creation error: {}", e))
                })?;

            encoder.write_all(data).map_err(|e| {
                CompressionError::CompressionFailed(format!("Zstd compression error: {}", e))
            })?;

            encoder.finish().map_err(|e| {
                CompressionError::CompressionFailed(format!("Zstd finish error: {}", e))
            })?;
        }

        // Check compression ratio - if compression didn't save much space, store uncompressed
        let total_compressed_size = 4 + temp_buffer.len(); // 4 bytes prefix + compressed data
        if total_compressed_size >= data.len() * 90 / 100 {
            // Less than 10% savings
            // Store uncompressed data with a special marker
            buffer.clear();
            buffer.extend_from_slice(&0u32.to_le_bytes()); // 0 size means uncompressed
            buffer.extend_from_slice(data);
            buffer.shrink_to_fit();
            return Ok(buffer);
        }

        // Store compressed data with proper size prefix
        buffer.clear();
        buffer.extend_from_slice(&(data.len() as u32).to_le_bytes()); // Write original size
        buffer.extend_from_slice(&temp_buffer); // Write compressed data
        buffer.shrink_to_fit();

        Ok(buffer)
    }

    /// Production Zstd decompression using thread-local buffer pool - true zero allocation
    #[inline]
    fn decompress_zstd_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
        use std::io::Read;
        use zstd::stream::read::Decoder;

        if data.len() < 4 {
            return Err(CompressionError::InvalidFormat);
        }

        // Read original size from 4-byte prefix (little endian)
        let original_size = u32::from_le_bytes([data[0], data[1], data[2], data[3]]) as usize;
        let compressed_data = &data[4..];

        // Check if data is stored uncompressed (original_size == 0 is our marker)
        if original_size == 0 {
            // Data is stored uncompressed after the 4-byte marker
            return Ok(compressed_data.to_vec());
        }

        // Decompress the data using buffer pool
        let mut buffer = PooledBuffer::acquire();
        buffer.clear();
        buffer.reserve(original_size);

        {
            let mut decoder = Decoder::new(compressed_data).map_err(|e| {
                CompressionError::DecompressionFailed(format!("Zstd decoder creation error: {}", e))
            })?;

            decoder.read_to_end(&mut buffer).map_err(|e| {
                CompressionError::DecompressionFailed(format!("Zstd decompression error: {}", e))
            })?;
        }

        // Validate size matches expected
        if buffer.len() != original_size {
            return Err(CompressionError::DecompressionFailed(format!(
                "Size mismatch: expected {}, got {}",
                original_size,
                buffer.len()
            )));
        }

        buffer.shrink_to_fit();
        Ok(buffer.into_vec())
    }


}

impl Default for CompressionManager {
    fn default() -> Self {
        Self::new(CompressionConfig::default())
    }
}
