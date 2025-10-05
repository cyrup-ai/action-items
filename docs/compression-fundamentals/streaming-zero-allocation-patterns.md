# Streaming Zero-Allocation Patterns - Research Findings

## Key Insights from Tokio Documentation

### 1. Tokio's Zero-Copy I/O Patterns
**Critical Discovery:**
- Tokio uses intrusive waker linked lists to avoid allocations
- `TcpStream::by_ref()` allows multiple concurrent operations without allocation
- Buffer management uses pre-allocated buffers with careful lifecycle management

**Zero-Allocation Pattern:**
```rust
// Tokio's approach - pre-allocated buffer reuse
let mut buf = [0; 1024];
loop {
    let n = socket.read(&mut buf).await?;
    // Process data without additional allocations
}
```

### 2. Intrusive Data Structures for Zero Allocation
**Key Pattern from Tokio:**
```rust
struct Waiters {
    list: LinkedList<Waiter>,  // Intrusive linked list
    reader: Option<Waker>,     // Single waker per direction
    writer: Option<Waker>,
}
```

**Application to Buffer Pools:**
- Use intrusive data structures to avoid allocation during buffer management
- Pre-allocate all necessary metadata structures
- Reuse waker objects instead of creating new ones

### 3. Resource Lifecycle Management
**Tokio's Approach:**
- Resources are acquired, used, and returned in a controlled manner
- Clear separation between resource acquisition and usage
- Explicit cleanup to prevent resource leaks

## Application to Compression Buffer Pools

### 1. Correct Buffer Pool Pattern
```rust
// WRONG (my current approach):
self.buffer_pool.return_buffer(Vec::with_capacity(8192));

// CORRECT (following Tokio patterns):
let mut buffer = self.buffer_pool.get_buffer();
// Use buffer for compression
self.buffer_pool.return_buffer(buffer);
```

### 2. Streaming Compression Architecture
**Insight:** Need to separate buffer acquisition from compression operation:
```rust
// Acquire buffer once
let mut output_buffer = self.buffer_pool.get_buffer();

// Use streaming API to write directly to buffer
let mut encoder = StreamingEncoder::new(&mut output_buffer);
encoder.write_all(input_data)?;
encoder.finish()?;

// Return buffer to pool
self.buffer_pool.return_buffer(output_buffer);
```

### 3. Library-Specific Streaming Strategies

**For flate2:**
- Use `Write` trait implementation with pre-allocated Vec
- Avoid `GzEncoder::new(Vec::new())` - provide existing buffer

**For LZ4:**
- Need to find streaming API or implement custom buffer management
- Current `compress_prepend_size()` always allocates

**For Zstd:**
- Use streaming API with pre-allocated output buffer
- Avoid bulk API that allocates internally

## Next Research Priorities

1. **Find Streaming APIs:** Research each compression library's streaming interfaces
2. **Buffer Ownership Patterns:** Understand how to provide pre-allocated buffers
3. **Memory Measurement:** Implement allocation tracking to verify zero-allocation claims
4. **Performance Benchmarks:** Compare streaming vs bulk APIs for each library