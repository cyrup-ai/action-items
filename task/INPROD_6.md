# Task: Apply Conditional Compilation for Test vs Production

## EXECUTIVE SUMMARY

This task separates test stubs/mocks from production code using Rust's conditional compilation attributes (`#[cfg(test)]` and `#[cfg(not(test))]`). The goal is to ensure production builds (`cargo build --release`) exclude test-only code, reducing binary size and preventing accidental inclusion of mock implementations.

**Status**: 50% Complete - Issue #1 RESOLVED, Issue #2 PENDING

**Impact**: Medium. Affects binary size, code clarity, and ensures production builds don't include test infrastructure.

---

## CURRENT STATUS (Updated 2025-10-27)

### ✅ COMPLETED

**Issue #1: Mock WASM Runtime in processor.rs - RESOLVED**

**File**: `packages/core/src/plugins/bridge/handlers/processor.rs`

**What was done**:
- Removed entire mock `WasmRuntime` struct (previously lines 19-73)
- Removed mock `get_wasm_runtime()` function
- Replaced with proper event-driven architecture documentation
- ServiceRequest::WasmCallback now returns immediate acknowledgment
- Actual WASM execution happens via ECS event system (WasmCallbackEvent)
- Added comprehensive architecture documentation explaining the event flow

**Result**: Production code no longer contains mock WASM implementations. The system now properly uses:
- `WasmCallbackHandler` (packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs)
- `ExtismPluginAdapter` for actual WASM execution
- Bevy ECS event system for asynchronous processing

See [`packages/core/src/plugins/bridge/handlers/processor.rs:1-71`](../packages/core/src/plugins/bridge/handlers/processor.rs) for current implementation.

---

### ❌ PENDING

**Issue #2: Test Infrastructure Unconditionally Compiled**

**File**: `packages/common/src/metrics/memory/mod.rs`

**Problem**: Testing module (557 lines) is unconditionally compiled into production binaries

**Current Code** (lines 89-107):
```rust
// Line 89 - Automated testing framework
pub mod testing;

// Lines 95-107 - Re-exports
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
```

**Impact**: 
- `testing.rs` (557 lines) compiled into all builds
- Includes `MemoryLeakTestSuite`, test scenarios, and test helpers
- Estimated binary size impact: 20-30KB
- Used by `MemoryMonitoringSystem` struct (line 117)

---

## IMPLEMENTATION GUIDE

### STEP 1: Conditional Compilation for Testing Module

**File**: `packages/common/src/metrics/memory/mod.rs`

#### Change 1: Module Declaration (Line 90)

**Current**:
```rust
// Automated testing framework
pub mod testing;
```

**Replace with**:
```rust
// Automated testing framework
// Available in tests and via test-utils feature for integration testing
#[cfg(any(test, feature = "test-utils"))]
pub mod testing;
```

#### Change 2: Re-exports (Lines 105-107)

**Current**:
```rust
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
```

**Replace with**:
```rust
#[cfg(any(test, feature = "test-utils"))]
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
```

#### Change 3: MemoryMonitoringSystem Field (Line 125)

**Current**:
```rust
pub struct MemoryMonitoringSystem {
    enhanced_tracker: Arc<EnhancedMemoryTracker>,
    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler: Option<JemallocProfiler>,
    #[cfg(feature = "dhat-heap")]
    dhat_profiler: Option<DhatProfiler>,
    leak_test_suite: MemoryLeakTestSuite,  // Line 125 - unconditional
}
```

**Replace with**:
```rust
pub struct MemoryMonitoringSystem {
    enhanced_tracker: Arc<EnhancedMemoryTracker>,
    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler: Option<JemallocProfiler>,
    #[cfg(feature = "dhat-heap")]
    dhat_profiler: Option<DhatProfiler>,
    #[cfg(any(test, feature = "test-utils"))]
    leak_test_suite: MemoryLeakTestSuite,
}
```

#### Change 4: Constructor - new() Method (Lines 129-165)

Locate the section that initializes `leak_test_suite`:

**Current** (around lines 156-162):
```rust
let mut leak_test_suite = MemoryLeakTestSuite::new();
leak_test_suite
    .initialize_tracking()
    .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

info!("Memory monitoring system initialized with all available layers");

Ok(Self {
    enhanced_tracker,

    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler,

    #[cfg(feature = "dhat-heap")]
    dhat_profiler,

    leak_test_suite,
})
```

**Replace with**:
```rust
#[cfg(any(test, feature = "test-utils"))]
let mut leak_test_suite = MemoryLeakTestSuite::new();

#[cfg(any(test, feature = "test-utils"))]
leak_test_suite
    .initialize_tracking()
    .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

info!("Memory monitoring system initialized with all available layers");

Ok(Self {
    enhanced_tracker,

    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler,

    #[cfg(feature = "dhat-heap")]
    dhat_profiler,

    #[cfg(any(test, feature = "test-utils"))]
    leak_test_suite,
})
```

#### Change 5: Fallback Constructor - new_fallback() Method (Lines 167-183)

**Current** (around lines 167-183):
```rust
fn new_fallback() -> Self {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));
    let leak_test_suite = MemoryLeakTestSuite::new(); // Don't initialize tracking in fallback

    Self {
        enhanced_tracker,

        #[cfg(feature = "jemalloc-profiling")]
        jemalloc_profiler: None,

        #[cfg(feature = "dhat-heap")]
        dhat_profiler: None,

        leak_test_suite,
    }
}
```

**Replace with**:
```rust
fn new_fallback() -> Self {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));
    
    #[cfg(any(test, feature = "test-utils"))]
    let leak_test_suite = MemoryLeakTestSuite::new(); // Don't initialize tracking in fallback

    Self {
        enhanced_tracker,

        #[cfg(feature = "jemalloc-profiling")]
        jemalloc_profiler: None,

        #[cfg(feature = "dhat-heap")]
        dhat_profiler: None,

        #[cfg(any(test, feature = "test-utils"))]
        leak_test_suite,
    }
}
```

#### Change 6: run_comprehensive_tests() Method (Lines 259-283)

Add conditional compilation attribute to entire method:

**Current**:
```rust
/// Run comprehensive memory leak tests
pub async fn run_comprehensive_tests(&mut self) -> Result<TestResults, MemorySystemError> {
    info!("Running comprehensive memory leak tests");
    
    // ... method body ...
}
```

**Replace with**:
```rust
/// Run comprehensive memory leak tests
#[cfg(any(test, feature = "test-utils"))]
pub async fn run_comprehensive_tests(&mut self) -> Result<TestResults, MemorySystemError> {
    info!("Running comprehensive memory leak tests");
    
    // ... method body ...
}
```

#### Change 7: test_suite_mut() Method (Lines 290-292)

Add conditional compilation attribute:

**Current**:
```rust
/// Get test suite for custom testing
pub fn test_suite_mut(&mut self) -> &mut MemoryLeakTestSuite {
    &mut self.leak_test_suite
}
```

**Replace with**:
```rust
/// Get test suite for custom testing
#[cfg(any(test, feature = "test-utils"))]
pub fn test_suite_mut(&mut self) -> &mut MemoryLeakTestSuite {
    &mut self.leak_test_suite
}
```

---

## IMPLEMENTATION PATTERNS (Reference)

### Pattern 1: Conditional Module Declaration
Use when entire module is test infrastructure:

```rust
#[cfg(any(test, feature = "test-utils"))]
pub mod testing;
```

### Pattern 2: Conditional Struct Fields
Use when struct contains test-only data:

```rust
pub struct MyStruct {
    production_field: String,
    
    #[cfg(any(test, feature = "test-utils"))]
    test_suite: TestSuite,
}
```

### Pattern 3: Conditional Method Implementation
Use when methods are only for testing:

```rust
#[cfg(any(test, feature = "test-utils"))]
pub fn run_tests(&mut self) -> TestResults {
    // test code
}
```

---

## VERIFICATION STEPS

After making changes to `mod.rs`:

### 1. Verify Production Build
```bash
cargo build --release --workspace
```
Expected: Succeeds without errors

### 2. Verify Test Build
```bash
cargo test --workspace --all-features
```
Expected: All tests pass, test infrastructure available

### 3. Verify with test-utils Feature
```bash
cargo build --features test-utils
```
Expected: Test infrastructure compiled when feature enabled

### 4. Check Binary Size (Optional)
```bash
# Before changes
cargo clean
cargo build --release
ls -lh target/release/action_items

# After changes
cargo clean
cargo build --release
ls -lh target/release/action_items
```
Expected: Binary slightly smaller (20-30KB reduction)

---

## DEFINITION OF DONE

- [x] **Issue #1**: Mock WASM runtime removed from processor.rs (COMPLETED)
- [ ] **Issue #2**: Testing module conditionally compiled in mod.rs
  - [ ] Line 90: Module declaration has `#[cfg(any(test, feature = "test-utils"))]`
  - [ ] Lines 105-107: Re-exports conditionally compiled
  - [ ] Line 125: `leak_test_suite` field conditionally included
  - [ ] Lines 156-162: Constructor conditionally initializes test suite
  - [ ] Lines 167-183: Fallback constructor conditionally includes field
  - [ ] Lines 259-283: `run_comprehensive_tests()` conditionally compiled
  - [ ] Lines 290-292: `test_suite_mut()` conditionally compiled
- [ ] Verified: `cargo build --release --workspace` succeeds
- [ ] Verified: `cargo test --workspace --all-features` succeeds
- [ ] Verified: No compilation errors in production or test builds

---

## FILES TO MODIFY

### Primary
- [`packages/common/src/metrics/memory/mod.rs`](../packages/common/src/metrics/memory/mod.rs) - Add conditional compilation (7 changes)

### Reference (Already Correct)
- [`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs) - Already fixed
- [`packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`](../packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs) - Real WASM handler
- [`packages/common/src/metrics/memory/testing.rs`](../packages/common/src/metrics/memory/testing.rs) - Test infrastructure module

---

## TECHNICAL NOTES

### Why Use `any(test, feature = "test-utils")`?

Two use cases for test infrastructure:
1. **Unit/integration tests** (`cfg(test)`): Built-in Rust test compilation
2. **Custom test harnesses** (`feature = "test-utils"`): External test binaries/benchmarks

Using `any(test, feature = "test-utils")` supports both:
- Regular tests: `cargo test`
- Feature-enabled: `cargo build --features test-utils`
- Production: Excluded by default

### What About Test Helper Functions?

Functions like `DatabaseService::new_in_memory()` are **intentionally public** test utilities:
- They're APIs for test code to call
- NOT mock implementations used in production paths
- Similar to standard test helpers in Rust ecosystem
- These do NOT need `#[cfg(test)]` gating

Example of correct test helper (no changes needed):
```rust
// packages/ecs-surrealdb/src/service.rs:44-49
/// Create in-memory database for testing
pub async fn new_in_memory() -> Result<Self, DatabaseError> {
    let config = DatabaseConfig {
        namespace: "test".to_string(),
        database: "test".to_string(),
        engine: DatabaseEngine::Mem,
    };
    Self::new(config).await
}
```

This is fine because:
- Returns real implementation (not mock data)
- Uses in-memory engine (valid production config)
- Doesn't echo fake data like the old processor.rs mock did

### Binary Size Impact

The `testing.rs` module includes:
- 557 lines of test infrastructure
- Test scenario definitions
- Memory threshold configurations
- Result tracking and reporting
- Estimated: 20-30KB in compiled binary

Not huge, but unnecessary overhead for production builds.

---

## EXISTING PATTERNS IN CODEBASE

### Feature-Gated Modules (Already Correct)
```rust
// packages/common/src/metrics/memory/mod.rs:84-89
#[cfg(feature = "jemalloc-profiling")]
pub mod jemalloc_profiler;

#[cfg(feature = "dhat-heap")]
pub mod dhat_profiler;
```

### Test Module Gating (Already Correct)
```rust
// packages/ecs-user-settings/src/lib.rs:68-69
#[cfg(test)]
mod tests;
```

Follow these existing patterns when adding conditional compilation to `testing` module.

---

## CONSTRAINTS

- ✅ DO NOT write tests for this change
- ✅ DO NOT write benchmarks
- ✅ DO NOT write extensive documentation beyond code comments
- ✅ DO ensure production builds exclude all test code
- ✅ DO ensure test builds include test infrastructure
- ✅ DO NOT break existing test functionality
- ✅ DO NOT break existing production functionality
- ✅ DO use existing codebase patterns (feature flags, conditional compilation)

---

## SEARCH COMMANDS USED IN RESEARCH

```bash
# Find mock/stub patterns in source code
rg "mock|Mock|stub|Stub" --type rust packages/*/src/ -n

# Find test-only comments in source code
rg "for testing|test only|test purposes" --type rust packages/*/src/ -n

# Find existing conditional compilation examples
rg "#\[cfg\(test\)\]|#\[cfg\(not\(test\)\)\]" --type rust packages/*/src/ -n

# Find WASM-related code
rg "WasmRuntime|get_wasm_runtime" --type rust packages/*/src/ -n

# Verify testing module usage
rg "MemoryLeakTestSuite|leak_test_suite" --type rust packages/common/src/ -n
```

---

## IMPLEMENTATION CHECKLIST

Use this checklist when implementing the changes:

```markdown
### packages/common/src/metrics/memory/mod.rs

- [ ] Line 90: Add `#[cfg(any(test, feature = "test-utils"))]` before `pub mod testing;`
- [ ] Lines 105-107: Add `#[cfg(any(test, feature = "test-utils"))]` before testing re-exports
- [ ] Line 125: Add `#[cfg(any(test, feature = "test-utils"))]` before `leak_test_suite` field
- [ ] Lines 156-162: Wrap test suite initialization in `#[cfg(any(test, feature = "test-utils"))]`
- [ ] Lines 167-183: Wrap test suite initialization in `#[cfg(any(test, feature = "test-utils"))]`
- [ ] Line 259: Add `#[cfg(any(test, feature = "test-utils"))]` to `run_comprehensive_tests()` method
- [ ] Line 290: Add `#[cfg(any(test, feature = "test-utils"))]` to `test_suite_mut()` method

### Verification
- [ ] Run `cargo build --release --workspace`
- [ ] Run `cargo test --workspace --all-features`
- [ ] Run `cargo build --features test-utils` (optional - if feature exists)
- [ ] Verify no warnings about unused code
- [ ] Verify tests still pass
```

---

**Last Updated**: 2025-10-27  
**Implementation Status**: 50% Complete (1 of 2 issues resolved)  
**Ready for Implementation**: Yes - Proceed with mod.rs changes
