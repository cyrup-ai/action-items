# Task: Complete Metric Lookup Implementation

## OBJECTIVE
Complete the incomplete metric lookup at line 625 in `violations.rs` by implementing dashboard query functionality for arbitrary custom metrics (counters and gauges).

## PRIORITY
P2 - HIGH - Affects metrics tracking and violation detection for custom metrics

## FILE LOCATION
**Primary file:** [`packages/common/src/metrics/violations.rs:625-632`](../packages/common/src/metrics/violations.rs)

**Related files:**
- [`packages/common/src/metrics/dashboard.rs:53-76`](../packages/common/src/metrics/dashboard.rs) - SystemSnapshot structure with counters/gauges HashMaps
- [`packages/common/src/metrics/dashboard.rs:235-326`](../packages/common/src/metrics/dashboard.rs) - Snapshot creation and population
- [`packages/common/src/metrics/counters.rs:1-100`](../packages/common/src/metrics/counters.rs) - ZeroAllocCounters with register/increment pattern
- [`packages/common/src/metrics/mod.rs:57-100`](../packages/common/src/metrics/mod.rs) - MetricsSystem coordinator structure

## CURRENT STATE

**Lines 625-632 in violations.rs:**
```rust
_ => {
    // Intentional: Unknown metrics return None to maintain type safety.
    // Registered sources: jemalloc (memory), app (UI perf), fetch (network).
    // Dynamic discovery deferred to plugin system (see core/src/plugins/mod.rs).
    // This prevents runtime errors from unknown metric types.
    // If needed, register new sources in MetricsRegistry::new() rather than dynamic lookup.
    None
},
```

The comment indicates this is intentional placeholder behavior, but the task objective is to enable dynamic lookup via dashboard query for registered custom metrics.

## ARCHITECTURE OVERVIEW

### Metrics System Structure

The metrics system has multiple components accessible through `MetricsSystem` ([`mod.rs:71-80`](../packages/common/src/metrics/mod.rs)):

```rust
pub struct MetricsSystem {
    config: MetricsConfig,
    prometheus_handle: Option<()>,
    collectors: MetricCollector,
    counters: ZeroAllocCounters,              // ← Custom counter storage
    memory_tracker: MemoryTracker,
    enhanced_memory_tracker: Arc<EnhancedMemoryTracker>,
    latency_tracker: LatencyTracker,
    violation_detector: ViolationDetector,
    dashboard: DashboardData,                 // ← Query target
}
```

**Existing hardcoded metric accessors** (already implemented in violations.rs:565-624):
- `memory_tracker()` - Memory statistics (current_usage, peak_usage, efficiency)
- `latency_tracker()` - Latency statistics (average_us, max_us, min_us, percentiles)
- `counters()` - Pre-defined counter metrics with `counter_` prefix

**Dashboard system** (NOT YET QUERIED):
- `dashboard()` - Returns `&DashboardData`
- Stores arbitrary counters and gauges in HashMaps
- Updated via `dashboard.update_from_system()`

### Dashboard Data Structure

From [`dashboard.rs:53-76`](../packages/common/src/metrics/dashboard.rs):

```rust
/// Current system metrics snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSnapshot {
    pub timestamp: u64,
    pub counters: HashMap<String, u64>,      // ← Query this for counter metrics
    pub gauges: HashMap<String, f64>,        // ← Query this for gauge metrics
    pub latency_stats: HashMap<String, LatencySnapshot>,
    pub memory_stats: MemorySnapshot,
    pub violation_stats: ViolationSnapshot,
    pub health_score: f64,
    pub uptime_seconds: u64,
}
```

**Key method:**
- `dashboard.current_snapshot() -> SystemSnapshot` - Get current metrics snapshot (returns cloned snapshot from RwLock)

### Counter System Structure

From [`counters.rs:17-27`](../packages/common/src/metrics/counters.rs):

```rust
/// Zero-allocation counter system with cache-line aligned atomic counters
#[repr(align(64))]
#[derive(Debug)]
pub struct ZeroAllocCounters {
    /// Pre-allocated atomic counters aligned to cache lines
    counters: [AtomicU64; MAX_COUNTERS],
    /// Counter name to index mapping
    name_to_index: parking_lot::RwLock<HashMap<String, usize>>,
    /// Next available counter index
    next_index: AtomicU64,
}
```

**Key methods:**
- `register_counter(name: &str) -> Option<usize>` - Register new counter by name ([`counters.rs:41-66`](../packages/common/src/metrics/counters.rs))
- `increment_by_name(name: &str, value: u64)` - Increment counter by name ([`counters.rs:84-89`](../packages/common/src/metrics/counters.rs))
- `snapshot() -> CounterSnapshot` - Get snapshot of all counters ([`counters.rs:106-115`](../packages/common/src/metrics/counters.rs))

### How Metrics Flow Into Dashboard

From [`dashboard.rs:235-326`](../packages/common/src/metrics/dashboard.rs):

1. **Counter Collection** (line 237-239):
```rust
// Collect counter data
let counter_snapshot = metrics_system.counters().snapshot();
snapshot.counters = counter_snapshot.counters;
```

2. **Gauge Collection** (lines 306-322) - **LIMITED SCOPE**:
```rust
// Add system resource gauges if enabled
if self.config.include_system_metrics {
    snapshot.gauges.insert(
        "memory_usage_mb".to_string(),
        (memory_stats.current_usage.max(0) as f64) / (1024.0 * 1024.0),
    );
    snapshot.gauges.insert(
        "memory_efficiency".to_string(),
        memory_stats.memory_efficiency,
    );
    snapshot.gauges.insert(
        "average_latency_ms".to_string(),
        latency_stats.average_us / 1000.0,
    );
    snapshot.gauges.insert(
        "health_score".to_string(), 
        snapshot.health_score
    );
}
```

**IMPORTANT NOTE ON GAUGES:** The `snapshot.gauges` HashMap contains ONLY manually inserted gauges (memory_usage_mb, memory_efficiency, average_latency_ms, health_score). It does NOT automatically collect arbitrary gauges from `metrics::gauge!()` calls. Custom gauges go to the Prometheus registry, not the dashboard snapshot. This implementation will work for the hardcoded gauges that ARE present in the snapshot.

### Practical Custom Metric Workflow

Example: Plugin tracks error count and wants violation detection

1. **Register counter** ([`counters.rs:355`](../packages/common/src/metrics/counters.rs)):
```rust
metrics_system.counters().register_counter("plugin_errors");
```

2. **Increment counter** ([`counters.rs:84-89`](../packages/common/src/metrics/counters.rs)):
```rust
metrics_system.counters().increment_by_name("plugin_errors", 1);
```

3. **Set violation threshold** ([`violations.rs:216-247`](../packages/common/src/metrics/violations.rs)):
```rust
violation_detector.set_threshold("plugin_errors", ViolationThreshold {
    threshold: 10.0,
    operator: ComparisonOperator::GreaterThan,
    severity: ViolationSeverity::Warning,
    window_seconds: None,
});
```

4. **Check thresholds** ([`violations.rs:427`](../packages/common/src/metrics/violations.rs)):
```rust
violation_detector.check_all_thresholds(&metrics_system);
```

5. **Current behavior**: `get_current_metric_value("plugin_errors")` returns `None` ❌

6. **Desired behavior**: Returns `Some(10.0)` from `snapshot.counters` ✅

## IMPLEMENTATION

### Step 1: Replace Incomplete Default Case

**File:** `packages/common/src/metrics/violations.rs`  
**Lines:** 625-632

**Current code:**
```rust
_ => {
    // Intentional: Unknown metrics return None to maintain type safety.
    // Registered sources: jemalloc (memory), app (UI perf), fetch (network).
    // Dynamic discovery deferred to plugin system (see core/src/plugins/mod.rs).
    // This prevents runtime errors from unknown metric types.
    // If needed, register new sources in MetricsRegistry::new() rather than dynamic lookup.
    None
},
```

**Replace with:**
```rust
_ => {
    // Query dashboard snapshot for registered counters and gauges
    let snapshot = metrics_system.dashboard().current_snapshot();
    
    // First, check if this is a registered counter (u64 -> f64 conversion)
    if let Some(&counter_value) = snapshot.counters.get(metric_name) {
        return Some(counter_value as f64);
    }
    
    // Then, check if this is a dashboard gauge (already f64)
    // Note: Only contains manually inserted gauges (memory_usage_mb, etc.)
    if let Some(&gauge_value) = snapshot.gauges.get(metric_name) {
        return Some(gauge_value);
    }
    
    // Metric not found in any system (memory, latency, counter, gauge)
    None
},
```

### Step 2: Update Method Documentation

**File:** `packages/common/src/metrics/violations.rs`  
**Line:** ~559 (above `fn get_current_metric_value`)

**Current comment:**
```rust
/// Get current value for a specific metric from the metrics system
```

**Update to:**
```rust
/// Get current value for a specific metric from the metrics system
///
/// Supports multiple metric types:
/// - Memory metrics: `memory_current_usage`, `memory_peak_usage`, `memory_efficiency`
/// - Latency metrics: `latency_average_us`, `latency_max_us`, `latency_min_us`, `latency_pXX_us`
/// - Counter metrics: `counter_<name>` (prefixed) or any registered counter name
/// - Dashboard gauges: `memory_usage_mb`, `memory_efficiency`, `average_latency_ms`, `health_score`
/// - Custom metrics: Any counter registered via `counters().register_counter(name)`
///
/// Returns `Some(value)` if metric exists, `None` if not found in any system.
```

### Why This Implementation Works

1. **Maintains existing behavior**: Hardcoded metrics (memory, latency, prefixed counters) still work via early returns in lines 565-624
2. **Adds dashboard query**: Default case now queries the dashboard's snapshot for registered metrics
3. **No performance impact**: 
   - Dashboard snapshot is read-only with RwLock, O(1) HashMap lookups
   - Snapshot cloning is acceptable during threshold checks (not hot path)
4. **Type-safe**: Converts u64 counters to f64 for uniform return type
5. **Graceful fallback**: Still returns None if metric doesn't exist anywhere
6. **Enables custom metrics**: Any counter registered with `register_counter()` will be found

### Code References - HashMap Access Pattern

Existing pattern from [`dashboard.rs:336`](../packages/common/src/metrics/dashboard.rs):
```rust
let total_requests = snapshot.counters.get("requests_total").copied().unwrap_or(0);
```

Our implementation uses the safer pattern with `if let Some(&value)` to avoid unwrap.

### Code References - Snapshot Access Pattern

From [`dashboard.rs:436`](../packages/common/src/metrics/dashboard.rs):
```rust
pub fn current_snapshot(&self) -> SystemSnapshot {
    let snapshot = self.current_snapshot.read();
    snapshot.clone()
}
```

This is called in our implementation via `metrics_system.dashboard().current_snapshot()`.

## DEFINITION OF DONE

- [ ] Lines 625-632 replaced with dashboard query implementation
- [ ] Method returns `Some(value)` for registered counters from ZeroAllocCounters
- [ ] Method returns `Some(value)` for dashboard gauges (memory_usage_mb, etc.)
- [ ] Method still returns `Some(value)` for hardcoded metrics (memory, latency)
- [ ] Method returns `None` for non-existent metrics
- [ ] Code compiles without warnings: `cargo check -p action_items_common`
- [ ] Intentional comment updated to reflect new dashboard query capability
- [ ] Method documentation updated to reflect new capability

## CONSTRAINTS

- **DO NOT** change the method signature `fn get_current_metric_value(&self, metric_name: &str, metrics_system: &crate::MetricsSystem) -> Option<f64>`
- **DO NOT** modify the hardcoded metric lookups (lines 565-624) - they work correctly
- **DO** query the dashboard snapshot for unknown metrics
- **DO** convert counter u64 values to f64
- **DO** handle missing metrics gracefully (return None)
- **DO NOT** panic or unwrap - use pattern matching with `if let Some(&value)`
- **DO NOT** add complexity - this is a simple HashMap lookup operation

## VERIFICATION APPROACH

After implementation, verify by:

1. **Code inspection**: 
   - Confirm dashboard query is present in default case at lines 625-632
   - Confirm intentional comment is updated to reflect new behavior
   - Confirm no unwrap/panic calls
   - Confirm pattern matching with `if let Some(&value)` for type-safe access

2. **Compilation**:
   - Run `cargo check -p action_items_common`
   - Verify no warnings about unused variables or type mismatches

3. **Manual validation** (if desired):
   - Register a custom counter: `metrics_system.counters().register_counter("test_metric")`
   - Increment it: `metrics_system.counters().increment_by_name("test_metric", 100)`
   - Set a violation threshold for it
   - Call `violation_detector.check_all_thresholds(metrics_system)`
   - Verify the custom metric is found and checked (look for violation records if threshold exceeded)

## NOTES

- The dashboard is updated via `update_from_system()` called from `MetricsSystem::update()`
- Counter snapshot is populated from ZeroAllocCounters via `counters().snapshot()` ([`dashboard.rs:237-239`](../packages/common/src/metrics/dashboard.rs))
- Gauge snapshot contains only manually inserted gauges, not arbitrary custom gauges
- This implementation enables violation detection for plugin-specific metrics without requiring code changes
- Performance characteristics: O(1) HashMap lookup, no allocations in hot path
- The `counter_` prefix check (lines 600-608) provides backward compatibility for metrics accessed via that pattern

## DEPENDENCIES

From `packages/common/Cargo.toml`:
- `parking_lot = "0.12.4"` - RwLock used by dashboard snapshot
- `metrics = "0.24.2"` - Core metrics-rs library (for Prometheus export)
- `metrics-exporter-prometheus = "0.17.2"` - Prometheus exporter integration

No new dependencies required for this implementation.
