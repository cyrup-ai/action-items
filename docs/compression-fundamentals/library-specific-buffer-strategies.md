# Library-Specific Buffer Management Strategies

## Strategy Design Based on Research Findings

### 1. Flate2 (Gzip/Deflate) Strategy
**Memory Pattern:** Uses Write trait - can write to pre-allocated Vec

**Correct Implementation:**
```rust
fn compress_gzip_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut buffer = self.buffer_pool.get_buffer();
    buffer.clear(); // Ensure buffer is empty
    
    {
        let mut encoder = GzEncoder::new(&mut buffer, Compression::new(self.config.compression_level));
        encoder.write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
        encoder.finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
    }
    
    // Return the ACTUAL buffer used, not a new one
    Ok(buffer)
}

// CRITICAL: Update return_buffer call in compress_sync
fn compress_sync(&self, data: Vec<u8>) -> Result<CompressedData, CompressionError> {
    // ... algorithm selection ...
    
    let compressed = match algorithm {
        CompressionAlgorithm::Gzip => {
            let result = self.compress_gzip_pooled(&data)?;
            // Return buffer to pool AFTER getting result
            // Note: Need to redesign to avoid this ownership issue
            result
        },
        // ... other algorithms ...
    };
    
    // ... rest of method ...
}
```

**Issue Identified:** Current design has ownership problem - can't return buffer to pool if we return it as result.

**Solution:** Redesign to separate buffer management from result:
```rust
fn compress_gzip_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut buffer = self.buffer_pool.get_buffer();
    buffer.clear();
    
    {
        let mut encoder = GzEncoder::new(&mut buffer, Compression::new(self.config.compression_level));
        encoder.write_all(data)?;
        encoder.finish()?;
    }
    
    // Clone result and return buffer to pool
    let result = buffer.clone();
    self.buffer_pool.return_buffer(buffer);
    Ok(result)
}
```

### 2. Deflate Strategy
**Current Issue:** Not using buffer pool at all

**Correct Implementation:**
```rust
fn compress_deflate_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut buffer = self.buffer_pool.get_buffer();
    buffer.clear();
    
    {
        let mut encoder = DeflateEncoder::new(&mut buffer, Compression::new(self.config.compression_level));
        encoder.write_all(data)
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
        encoder.finish()
            .map_err(|e| CompressionError::CompressionFailed(e.to_string()))?;
    }
    
    let result = buffer.clone();
    self.buffer_pool.return_buffer(buffer);
    Ok(result)
}
```

### 3. LZ4 Strategy
**Current Issue:** `compress_prepend_size()` always allocates internally

**Research Needed:** Find streaming API or implement custom wrapper
**Temporary Solution:** Use existing API but document allocation
**Future Solution:** Implement custom LZ4 wrapper with buffer pool

```rust
fn compress_lz4_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    // TODO: Research LZ4 streaming API for zero allocation
    // Current implementation allocates internally
    let compressed = compress_prepend_size(data);
    Ok(compressed)
}
```

### 4. Zstd Strategy
**Current Issues:** 
- Hardcoded 16MB limit
- Bulk API allocates internally

**Correct Implementation:**
```rust
fn compress_zstd_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    // Use bulk API for now, research streaming API later
    compress(data, self.config.compression_level as i32)
        .map_err(|e| CompressionError::CompressionFailed(format!("Zstd compression error: {}", e)))
}

fn decompress_zstd_pooled(&self, data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    // Remove hardcoded limit - let zstd determine appropriate size
    decompress(data, 0) // 0 means use size from compressed data
        .map_err(|e| CompressionError::DecompressionFailed(format!("Zstd decompression error: {}", e)))
}
```

## Implementation Priority

### Phase 2A: Fix Critical Bugs (High Priority)
1. Fix buffer pool return bug in gzip methods
2. Implement buffer pooling for deflate methods  
3. Remove hardcoded Zstd limit
4. Remove unused imports

### Phase 2B: Optimize Buffer Management (Medium Priority)
1. Research LZ4 streaming API
2. Research Zstd streaming API
3. Implement allocation tracking
4. Add performance benchmarks

### Phase 2C: Advanced Optimizations (Low Priority)
1. Implement custom compression wrappers
2. Add compression algorithm auto-tuning
3. Implement adaptive buffer sizing