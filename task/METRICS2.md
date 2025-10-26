# Task: Complete Metric Lookup Implementation

## OBJECTIVE
Complete the incomplete metric lookup at line 613 in `violations.rs` by implementing dashboard query functionality for arbitrary custom metrics (counters and gauges).

## PRIORITY
P2 - HIGH - Affects metrics tracking and violation detection for custom metrics

## FILE LOCATION
**Primary file:** `packages/common/src/metrics/violations.rs:608-616`

**Related files:**
- [`packages/common/src/metrics/dashboard.rs`](../packages/common/src/metrics/dashboard.rs) - Dashboard data with SystemSnapshot
- [`packages/common/src/metrics/counters.rs`](../packages/common/src/metrics/counters.rs) - Counter snapshot structure
- [`packages/common/src/metrics/mod.rs`](../packages/common/src/metrics/mod.rs) - MetricsSystem coordinator

## CURRENT STATE

**Line 608-616 in violations.rs:**
```rust
// Default case - try to find in dashboard if available
_ => {
    // In a more advanced implementation, this could query additional metric sources
    // or plugin-specific metrics. For now, we return None for unknown metrics.
    None
},
```

The comment "For now, we return None" indicates this is incomplete placeholder code.

## ARCHITECTURE OVERVIEW

### Metrics System Structure

The metrics system has multiple components accessible through `MetricsSystem`:

1. **Hardcoded metric accessors** (already implemented in lines 559-607):
   - `memory_tracker()` - Memory statistics
   - `latency_tracker()` - Latency statistics  
   - `counters()` - Pre-defined counter metrics

2. **Dashboard system** (NOT YET QUERIED):
   - `dashboard()` - Returns `&DashboardData`
   - Stores arbitrary counters and gauges in HashMaps
   - Updated via `dashboard.update_from_system()`

### Dashboard Data Structure

From [`packages/common/src/metrics/dashboard.rs`](../packages/common/src/metrics/dashboard.rs):

```rust
pub struct DashboardData {
    current_snapshot: RwLock<SystemSnapshot>,
    // ... other fields
}

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

**Key methods:**
- `dashboard.current_snapshot() -> SystemSnapshot` - Get current metrics snapshot (line 400+)
- Snapshot is populated in `create_snapshot()` (line 218+) via `counters().snapshot()`

### Counter System Structure

From [`packages/common/src/metrics/counters.rs`](../packages/common/src/metrics/counters.rs):

```rust
pub struct CounterSnapshot {
    pub counters: HashMap<String, u64>,
}
```

The dashboard's `snapshot.counters` comes from `metrics_system.counters().snapshot().counters` (dashboard.rs line 238-239).

### How Metrics Flow Into Dashboard

1. Application code calls `metrics::counter!("my_metric").increment(1)` or `metrics::gauge!("my_gauge").set(42.0)`
2. `ZeroAllocCounters` tracks these via name-to-index mapping
3. `dashboard.update_from_system()` calls `counters().snapshot()` 
4. Snapshot data is stored in `SystemSnapshot.counters` and `SystemSnapshot.gauges`
5. **This data is NOT currently queried by violations.rs**

## IMPLEMENTATION

### Step 1: Replace Incomplete Default Case

**File:** `packages/common/src/metrics/violations.rs`  
**Lines:** 608-616

**Current code:**
```rust
// Default case - try to find in dashboard if available
_ => {
    // In a more advanced implementation, this could query additional metric sources
    // or plugin-specific metrics. For now, we return None for unknown metrics.
    None
},
```

**Replace with:**
```rust
// Default case - query dashboard for arbitrary counters and gauges
_ => {
    // Get current dashboard snapshot containing all registered metrics
    let snapshot = metrics_system.dashboard().current_snapshot();
    
    // First, check if this is a counter metric (u64 -> f64 conversion)
    if let Some(&counter_value) = snapshot.counters.get(metric_name) {
        return Some(counter_value as f64);
    }
    
    // Then, check if this is a gauge metric (already f64)
    if let Some(&gauge_value) = snapshot.gauges.get(metric_name) {
        return Some(gauge_value);
    }
    
    // Metric not found in any system (memory, latency, counter, gauge)
    None
},
```

### Step 2: Update Method Documentation

**File:** `packages/common/src/metrics/violations.rs`  
**Line:** ~549 (above `fn get_current_metric_value`)

**Current comment:**
```rust
/// Get current value for a specific metric from the metrics system
```

**Update to:**
```rust
/// Get current value for a specific metric from the metrics system
///
/// Supports multiple metric types:
/// - Memory metrics: `memory_current_usage`, `memory_peak_usage`, etc.
/// - Latency metrics: `latency_average_us`, `latency_max_us`, `latency_pXX_us`
/// - Counter metrics: `counter_<name>` (prefixed) or any dashboard counter
/// - Gauge metrics: Any gauge registered in the dashboard
/// - Custom metrics: Any metric exported via `metrics::counter!()` or `metrics::gauge!()`
///
/// Returns `Some(value)` if metric exists, `None` if not found in any system.
```

### Why This Implementation?

1. **Maintains existing behavior**: Hardcoded metrics (memory, latency, counters with `counter_` prefix) still work via early returns
2. **Adds dashboard query**: Default case now queries the dashboard's snapshot for arbitrary metrics
3. **No performance impact**: Dashboard snapshot is read-only with RwLock, O(1) HashMap lookups
4. **Type-safe**: Converts u64 counters to f64 for uniform return type
5. **Graceful fallback**: Still returns None if metric doesn't exist anywhere

## DEFINITION OF DONE

- [ ] Line 608-616 replaced with dashboard query implementation
- [ ] Method returns `Some(value)` for dashboard counters and gauges
- [ ] Method still returns `Some(value)` for hardcoded metrics (memory, latency, etc.)
- [ ] Method returns `None` for non-existent metrics
- [ ] Code compiles without warnings
- [ ] "For now" comment removed
- [ ] Method documentation updated to reflect new capability

## CONSTRAINTS

- **DO NOT** change the method signature `fn get_current_metric_value(&self, metric_name: &str, metrics_system: &crate::MetricsSystem) -> Option<f64>`
- **DO NOT** modify the hardcoded metric lookups (lines 559-607) - they work correctly
- **DO** query the dashboard snapshot for unknown metrics
- **DO** convert counter u64 values to f64
- **DO** handle missing metrics gracefully (return None)
- **DO NOT** panic or unwrap - use pattern matching

## RESEARCH FINDINGS

### Third-Party Dependencies

From `packages/common/Cargo.toml`:
- `metrics = "0.24.2"` - Core metrics-rs library for metric primitives
- `metrics-exporter-prometheus = "0.17.2"` - Prometheus exporter integration
- `parking_lot = "0.12.4"` - High-performance RwLock used by dashboard

### Existing Code Patterns

**Dashboard snapshot access pattern** (from dashboard.rs line 400):
```rust
pub fn current_snapshot(&self) -> SystemSnapshot {
    let snapshot = self.current_snapshot.read();
    snapshot.clone()
}
```

**Counter snapshot creation** (from dashboard.rs line 238):
```rust
let counter_snapshot = metrics_system.counters().snapshot();
snapshot.counters = counter_snapshot.counters;
```

**HashMap lookup patterns** (from dashboard.rs line 336):
```rust
let total_requests = snapshot.counters.get("requests_total").copied().unwrap_or(0);
```

### Performance Characteristics

- Dashboard snapshot: Clones the entire snapshot (acceptable since it's done during threshold checks)
- HashMap lookups: O(1) average case
- RwLock read: Multiple concurrent readers, no contention expected
- Memory overhead: None - reuses existing dashboard data

## VERIFICATION APPROACH

After implementation, verify by:

1. **Code inspection**: 
   - Confirm dashboard query is present in default case
   - Confirm "for now" comment is removed
   - Confirm no unwrap/panic calls

2. **Compilation**:
   - Run `cargo check -p action_items_common`
   - Verify no warnings about unused variables

3. **Manual validation**:
   - Set a violation threshold for a custom metric
   - Call `metrics::counter!("custom_metric").increment(100)`
   - Call `violation_detector.check_all_thresholds(metrics_system)`
   - Verify the custom metric is found and checked

## NOTES

- The dashboard is updated via `update_from_system()` called from `MetricsSystem::update()`
- Custom metrics flow through `metrics::counter!()` and `metrics::gauge!()` macros
- The `metrics` crate provides a global registry that the dashboard reads from
- This implementation enables violation detection for plugin-specific metrics without requiring code changes
