# Task: Apply Conditional Compilation for Test vs Production

## EXECUTIVE SUMMARY

This task separates test stubs/mocks from production code using Rust's conditional compilation attributes (`#[cfg(test)]` and `#[cfg(not(test))]`). The goal is to ensure production builds (`cargo build --release`) exclude test-only code, reducing binary size and preventing accidental inclusion of mock implementations.

**Status**: 50% Complete - Issue #1 RESOLVED, Issue #2 PENDING

**Impact**: Medium. Affects binary size (estimated 20-30KB reduction), code clarity, and ensures production builds don't include test infrastructure.

**Core Objective**: Remove 629 lines of test infrastructure from production binaries while maintaining full test functionality.

---

## CURRENT STATUS (Updated 2025-10-27)

### ✅ COMPLETED

**Issue #1: Mock WASM Runtime in processor.rs - RESOLVED**

**File**: [`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs)

**What was done**:
- Removed entire mock `WasmRuntime` struct (previously returning fake data)
- Removed mock `get_wasm_runtime()` function
- Replaced with proper event-driven architecture documentation (lines 1-80)
- ServiceRequest::WasmCallback now returns immediate acknowledgment
- Actual WASM execution happens via ECS event system (WasmCallbackEvent)
- Added comprehensive architecture documentation explaining the event flow

**Result**: Production code no longer contains mock WASM implementations. The system now properly uses:
- `WasmCallbackHandler` ([packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs](../packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs))
- `ExtismPluginAdapter` for actual WASM execution
- Bevy ECS event system for asynchronous processing

**Verification**: Confirmed by reading processor.rs:1-80 - no mock code present.

---

### ❌ PENDING

**Issue #2: Test Infrastructure Unconditionally Compiled**

**File**: [`packages/common/src/metrics/memory/mod.rs`](../packages/common/src/metrics/memory/mod.rs)

**Problem**: Testing module (629 lines) is unconditionally compiled into production binaries

**Current State** (verified 2025-10-27):
- File: 355 lines total
- Testing module: [`packages/common/src/metrics/memory/testing.rs`](../packages/common/src/metrics/memory/testing.rs) - 629 lines
- Module declared at line 90 WITHOUT cfg attribute
- Re-exports at lines 99-103 WITHOUT cfg attribute  
- Struct field at line 121 WITHOUT cfg attribute
- Methods at lines 259-283 and 292-294 WITHOUT cfg attributes

**Impact**: 
- `testing.rs` (629 lines) compiled into all builds
- Includes `MemoryLeakTestSuite`, test scenarios, and test helpers
- Estimated binary size impact: 20-30KB
- Used only in test code - NEVER called in production (verified via codebase search)

**Evidence of Non-Use in Production**:
```bash
# Search results show ZERO production usage:
$ rg "run_comprehensive_tests" packages/*/src/
# No results - method only defined, never called

$ rg "MemoryMonitoringSystem::new" packages/*/src/
# No results in src/ - only in tests/
```

---

## IMPLEMENTATION APPROACHES

### Approach 1: Simple (RECOMMENDED)

Use `#[cfg(test)]` only - matches existing codebase patterns.

**Pros**:
- Simpler, cleaner code
- Matches patterns in ecs-progress, ecs-deno, ecs-search-aggregator
- No Cargo.toml changes needed
- Sufficient for current use case (no benchmarks or custom test harnesses)

**Cons**:
- If future benchmarks need test infrastructure, would need refactoring

**Example**:
```rust
#[cfg(test)]
pub mod testing;

#[cfg(test)]
pub use testing::{MemoryLeakTestSuite, /* ... */};
```

### Approach 2: Future-Proof

Use `#[cfg(any(test, feature = "test-utils"))]` - allows external access via feature flag.

**Pros**:
- Supports future benchmarks or custom test harnesses
- More flexible for complex testing scenarios
- Explicitly documents "this is for testing"

**Cons**:
- Requires adding "test-utils" feature to Cargo.toml
- More complex than needed for current use case
- No current need identified in codebase

**Example**:
```rust
#[cfg(any(test, feature = "test-utils"))]
pub mod testing;

#[cfg(any(test, feature = "test-utils"))]
pub use testing::{MemoryLeakTestSuite, /* ... */};
```

**This task will use Approach 1** unless instructed otherwise. Approach 2 details included for reference.

---

## IMPLEMENTATION GUIDE - APPROACH 1 (RECOMMENDED)

### File: `packages/common/src/metrics/memory/mod.rs`

**Total Changes**: 7 locations

#### Change 1: Module Declaration (Line 90)

**Current**:
```rust
// Line 88-90
#[cfg(feature = "dhat-heap")]
pub mod dhat_profiler;

// Automated testing framework
pub mod testing;
```

**Replace with**:
```rust
// Line 88-90
#[cfg(feature = "dhat-heap")]
pub mod dhat_profiler;

// Automated testing framework - only compiled in test builds
#[cfg(test)]
pub mod testing;
```

**Rationale**: Prevents testing module from being compiled in production builds.

---

#### Change 2: Re-exports (Lines 99-103)

**Current**:
```rust
// Lines 94-103
#[cfg(feature = "jemalloc-profiling")]
pub use jemalloc_profiler::{
    JemallocConfig, JemallocProfiler, JemallocStats, MemoryProfilingError, analysis::LeakAnalysis,
};
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
```

**Replace with**:
```rust
// Lines 94-103
#[cfg(feature = "jemalloc-profiling")]
pub use jemalloc_profiler::{
    JemallocConfig, JemallocProfiler, JemallocStats, MemoryProfilingError, analysis::LeakAnalysis,
};
#[cfg(test)]
pub use testing::{
    LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, TestMemoryStats,
    TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
};
```

**Rationale**: Prevents test types from being exported in production builds.

---

#### Change 3: Struct Field (Line 121)

**Current**:
```rust
// Lines 115-122
#[derive(Debug)]
pub struct MemoryMonitoringSystem {
    enhanced_tracker: Arc<EnhancedMemoryTracker>,
    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler: Option<JemallocProfiler>,
    #[cfg(feature = "dhat-heap")]
    dhat_profiler: Option<DhatProfiler>,
    leak_test_suite: MemoryLeakTestSuite,
}
```

**Replace with**:
```rust
// Lines 115-122
#[derive(Debug)]
pub struct MemoryMonitoringSystem {
    enhanced_tracker: Arc<EnhancedMemoryTracker>,
    #[cfg(feature = "jemalloc-profiling")]
    jemalloc_profiler: Option<JemallocProfiler>,
    #[cfg(feature = "dhat-heap")]
    dhat_profiler: Option<DhatProfiler>,
    #[cfg(test)]
    leak_test_suite: MemoryLeakTestSuite,
}
```

**Rationale**: Removes test suite field from production struct, reducing memory footprint.

---

#### Change 4: Constructor Initialization (Lines 156-163)

**Current**:
```rust
// Lines 145-169 (constructor context)
pub fn new() -> Result<Self, MemorySystemError> {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));

    // ... jemalloc and dhat initialization ...

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
}
```

**Replace with**:
```rust
// Lines 145-169 (constructor context)
pub fn new() -> Result<Self, MemorySystemError> {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));

    // ... jemalloc and dhat initialization ...

    #[cfg(test)]
    let mut leak_test_suite = MemoryLeakTestSuite::new();

    #[cfg(test)]
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

        #[cfg(test)]
        leak_test_suite,
    })
}
```

**Rationale**: Prevents test suite initialization in production builds, eliminating startup overhead.

---

#### Change 5: Fallback Constructor (Lines 172-186)

**Current**:
```rust
// Lines 172-186
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
// Lines 172-186
fn new_fallback() -> Self {
    let base_tracker = Arc::new(MemoryTracker::new());
    let enhanced_tracker = Arc::new(EnhancedMemoryTracker::new(base_tracker));
    
    #[cfg(test)]
    let leak_test_suite = MemoryLeakTestSuite::new(); // Don't initialize tracking in fallback

    Self {
        enhanced_tracker,

        #[cfg(feature = "jemalloc-profiling")]
        jemalloc_profiler: None,

        #[cfg(feature = "dhat-heap")]
        dhat_profiler: None,

        #[cfg(test)]
        leak_test_suite,
    }
}
```

**Rationale**: Consistent with main constructor - no test infrastructure in production fallback.

---

#### Change 6: Test Runner Method (Lines 259-283)

**Current**:
```rust
// Lines 259-283
/// Run comprehensive memory leak tests
pub async fn run_comprehensive_tests(&mut self) -> Result<TestResults, MemorySystemError> {
    info!("Running comprehensive memory leak tests");
    
    // Add standard test scenarios
    self.leak_test_suite
        .add_scenario(scenarios::plugin_isolation_test());
    self.leak_test_suite
        .add_scenario(scenarios::fragmentation_stress_test());
    self.leak_test_suite
        .add_scenario(scenarios::long_running_test());

    let results = self
        .leak_test_suite
        .run_all()
        .await
        .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

    if !results.failed.is_empty() {
        error!(
            "Memory leak tests failed: {}/{} tests passed",
            results.passed.len(),
            results.passed.len() + results.failed.len()
        );
    } else {
        info!("All {} memory leak tests passed", results.passed.len());
    }

    Ok(results)
}
```

**Replace with**:
```rust
// Lines 259-283
/// Run comprehensive memory leak tests
#[cfg(test)]
pub async fn run_comprehensive_tests(&mut self) -> Result<TestResults, MemorySystemError> {
    info!("Running comprehensive memory leak tests");
    
    // Add standard test scenarios
    self.leak_test_suite
        .add_scenario(scenarios::plugin_isolation_test());
    self.leak_test_suite
        .add_scenario(scenarios::fragmentation_stress_test());
    self.leak_test_suite
        .add_scenario(scenarios::long_running_test());

    let results = self
        .leak_test_suite
        .run_all()
        .await
        .map_err(|e| MemorySystemError::TestingError(e.to_string()))?;

    if !results.failed.is_empty() {
        error!(
            "Memory leak tests failed: {}/{} tests passed",
            results.passed.len(),
            results.passed.len() + results.failed.len()
        );
    } else {
        info!("All {} memory leak tests passed", results.passed.len());
    }

    Ok(results)
}
```

**Rationale**: Method is never called in production (verified), should only exist in test builds.

---

#### Change 7: Test Suite Accessor (Lines 292-294)

**Current**:
```rust
// Lines 292-294
/// Get test suite for custom testing
pub fn test_suite_mut(&mut self) -> &mut MemoryLeakTestSuite {
    &mut self.leak_test_suite
}
```

**Replace with**:
```rust
// Lines 292-294
/// Get test suite for custom testing
#[cfg(test)]
pub fn test_suite_mut(&mut self) -> &mut MemoryLeakTestSuite {
    &mut self.leak_test_suite
}
```

**Rationale**: Accessor for test-only field should only exist in test builds.

---

## RUST CONDITIONAL COMPILATION PRIMER

### How `#[cfg(test)]` Works

Rust's conditional compilation is evaluated at compile time, not runtime:

```rust
// This code only exists when compiling for tests
#[cfg(test)]
fn test_helper() { }

// This code only exists in non-test builds
#[cfg(not(test))]
fn production_only() { }
```

### When Test Code is Compiled

```bash
# Test builds - cfg(test) is TRUE
cargo test              # Includes test code
cargo test --workspace  # Includes test code

# Production builds - cfg(test) is FALSE  
cargo build             # Excludes test code
cargo build --release   # Excludes test code
cargo run               # Excludes test code
```

### Struct Fields with Conditional Compilation

When a struct field is conditionally compiled, you must also conditionally initialize it:

```rust
struct MyStruct {
    always_present: String,
    #[cfg(test)]
    test_only: TestSuite,
}

impl MyStruct {
    fn new() -> Self {
        #[cfg(test)]
        let test_only = TestSuite::new();
        
        Self {
            always_present: String::new(),
            #[cfg(test)]
            test_only,  // Only included in test builds
        }
    }
}
```

### Why This Pattern is Safe

1. **Type checking**: Compiler ensures all references to conditionally-compiled items are also conditional
2. **Binary exclusion**: Test code is completely absent from production binaries (not just dead code)
3. **Zero runtime cost**: No performance impact, no branch elimination needed
4. **Standard library pattern**: Used extensively in `std` (e.g., `std::io::Error` has test-only methods)

---

## VERIFICATION STEPS

After making changes to `mod.rs`:

### 1. Verify Production Build Excludes Test Code
```bash
cargo clean
cargo build --release --workspace
```
**Expected**: Succeeds without errors. Test infrastructure not compiled.

### 2. Verify Test Build Includes Test Code
```bash
cargo test --workspace --all-features
```
**Expected**: All tests pass. Test infrastructure available.

### 3. Check for Unused Code Warnings
```bash
cargo build --release 2>&1 | grep -i "warning.*unused"
```
**Expected**: No warnings about unused test infrastructure (it's properly excluded).

### 4. Verify Test-Specific Tests Still Work
```bash
cargo test -p action_items_common --test '*' -- --nocapture
```
**Expected**: Memory leak tests in `packages/common/tests/metrics/memory/` pass.

### 5. Optional: Check Binary Size Reduction
```bash
# Before changes
cargo clean && cargo build --release
ls -lh target/release/action_items

# After changes  
cargo clean && cargo build --release
ls -lh target/release/action_items
```
**Expected**: Binary size reduced by approximately 20-30KB.

---

## DEFINITION OF DONE

- [x] **Issue #1**: Mock WASM runtime removed from processor.rs (COMPLETED)
- [ ] **Issue #2**: Testing module conditionally compiled in mod.rs
  - [ ] Line 90: Module declaration has `#[cfg(test)]` 
  - [ ] Lines 99-103: Re-exports conditionally compiled
  - [ ] Line 121: `leak_test_suite` field conditionally included
  - [ ] Lines 156-163: Constructor conditionally initializes test suite
  - [ ] Lines 172-186: Fallback constructor conditionally includes field
  - [ ] Lines 259-283: `run_comprehensive_tests()` conditionally compiled
  - [ ] Lines 292-294: `test_suite_mut()` conditionally compiled
- [ ] Verified: `cargo build --release --workspace` succeeds without errors
- [ ] Verified: `cargo test --workspace --all-features` succeeds
- [ ] Verified: No unused code warnings for test infrastructure

---

## FILES TO MODIFY

### Primary
- [`packages/common/src/metrics/memory/mod.rs`](../packages/common/src/metrics/memory/mod.rs) - Add conditional compilation (7 changes)

### Reference (No Changes Needed)
- [`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs) - Already fixed (Issue #1 resolved)
- [`packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`](../packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs) - Real WASM handler
- [`packages/common/src/metrics/memory/testing.rs`](../packages/common/src/metrics/memory/testing.rs) - Test infrastructure module (629 lines)
- [`packages/common/tests/metrics/memory/mod_tests.rs`](../packages/common/tests/metrics/memory/mod_tests.rs) - Uses MemoryMonitoringSystem in tests
- [`packages/common/tests/metrics/memory/testing_tests.rs`](../packages/common/tests/metrics/memory/testing_tests.rs) - Uses MemoryLeakTestSuite in tests

---

## TECHNICAL DEEP DIVE

### Binary Size Impact Analysis

The `testing.rs` module includes:
- **629 lines** of test infrastructure code
- `MemoryLeakTestSuite` struct (73 lines)
- Test scenario definitions (multiple structs, ~150 lines)
- Memory threshold configurations (~50 lines)
- Result tracking and reporting (~100 lines)
- Async test execution logic (~200 lines)
- Helper functions and utilities (~56 lines)

**Estimated compiled size**: 20-30KB in release builds (with optimizations)

**Why this matters**:
- Embedded deployments: Every KB counts
- Container images: Smaller binaries = faster deployments
- Security surface: Less code = fewer potential vulnerabilities
- Binary scanning: Security tools must scan less code

### Existing Conditional Compilation Patterns in Codebase

The codebase already uses feature-gated modules extensively:

```rust
// From packages/common/src/metrics/memory/mod.rs:84-89
#[cfg(feature = "jemalloc-profiling")]
pub mod jemalloc_profiler;

#[cfg(feature = "dhat-heap")]
pub mod dhat_profiler;
```

And test modules are consistently gated:

```rust
// From packages/ecs-progress/src/assets.rs:288
#[cfg(test)]
mod tests { /* ... */ }

// From packages/ecs-progress/src/debug.rs:289
#[cfg(test)]
mod tests { /* ... */ }

// From packages/ecs-search-aggregator/tests/integration_tests.rs:4
#[cfg(test)]
mod tests { /* ... */ }
```

**This task brings `testing` module in line with these established patterns.**

### Why Use `#[cfg(test)]` Instead of Runtime Checks?

**BAD** (runtime check):
```rust
pub fn run_comprehensive_tests() {
    if cfg!(test) {  // ❌ Code still compiled!
        // test logic
    }
}
```

**GOOD** (compile-time exclusion):
```rust
#[cfg(test)]  // ✅ Code not compiled in production
pub fn run_comprehensive_tests() {
    // test logic
}
```

Runtime checks (`cfg!()` macro) still compile the code - it's just dead code elimination. Compile-time checks (`#[cfg()]` attribute) completely exclude the code from compilation.

### Why Not Use a `test-utils` Feature?

The task originally suggested `#[cfg(any(test, feature = "test-utils"))]`. However:

**Codebase search shows NO need for this**:
```bash
# No benchmarks using test infrastructure
$ find . -name "benches" -type d
# (no results)

# No custom test harnesses
$ rg "harness = false" Cargo.toml
# (no results)

# No external projects needing test utilities
# (this is not a library published to crates.io)
```

**If future needs arise**, adding the feature is trivial:
1. Add `test-utils = []` to `Cargo.toml` features
2. Change `#[cfg(test)]` to `#[cfg(any(test, feature = "test-utils"))]`

**For now, YAGNI principle applies** (You Aren't Gonna Need It).

### What About Test Helper Functions?

Not all test-related code needs `#[cfg(test)]`. Test **helpers** (as opposed to test **infrastructure**) can remain public:

```rust
// This is fine - it's a helper, not a mock
// From packages/ecs-surrealdb/src/service.rs:44-49
pub async fn new_in_memory() -> Result<Self, DatabaseError> {
    let config = DatabaseConfig {
        namespace: "test".to_string(),
        database: "test".to_string(),
        engine: DatabaseEngine::Mem,  // Real engine, just in-memory
    };
    Self::new(config).await
}
```

**Difference**:
- **Test helpers**: Create real instances with test-friendly configs (keep public)
- **Test infrastructure**: Test execution frameworks, mocks, fake data (gate with `#[cfg(test)]`)
- **Mock implementations**: Return fake data instead of doing real work (gate with `#[cfg(test)]`)

The removed processor.rs mock was returning fake WASM data - that's a mock. The `new_in_memory()` database helper creates a real database with a test config - that's a helper.

---

## CONSTRAINTS AND GUIDELINES

### ✅ DO

- Use `#[cfg(test)]` for all test-only code
- Remove test infrastructure from production builds
- Ensure all tests still pass after changes
- Follow existing codebase patterns (feature flags for optional functionality)
- Verify no compilation errors in production or test builds
- Keep test functionality fully intact

### ❌ DO NOT

- Write new tests for this change (just verify existing tests pass)
- Write benchmarks for this change
- Write extensive documentation beyond code comments
- Break existing test functionality
- Break existing production functionality  
- Add complexity without clear need (YAGNI)
- Use runtime checks instead of compile-time exclusion

---

## IMPLEMENTATION CHECKLIST

Use this checklist when implementing the changes:

```markdown
### packages/common/src/metrics/memory/mod.rs

- [ ] Line 90: Add `#[cfg(test)]` before `pub mod testing;`
- [ ] Lines 99-103: Add `#[cfg(test)]` before testing re-exports block
- [ ] Line 121: Add `#[cfg(test)]` before `leak_test_suite` field
- [ ] Lines 156-163: Wrap test suite initialization with `#[cfg(test)]`
- [ ] Lines 172-186: Wrap test suite field initialization with `#[cfg(test)]`
- [ ] Line 259: Add `#[cfg(test)]` to `run_comprehensive_tests()` method signature
- [ ] Line 292: Add `#[cfg(test)]` to `test_suite_mut()` method signature

### Verification
- [ ] Run `cargo build --release --workspace` (should succeed)
- [ ] Run `cargo test --workspace --all-features` (should pass)
- [ ] Check for no unused code warnings
- [ ] Verify tests in packages/common/tests/metrics/memory/ still pass
```

---

## RESEARCH CITATIONS

### Codebase File References

All relative paths from project root: `/Volumes/samsung_t9/action-items/`

1. **Primary file to modify**: [packages/common/src/metrics/memory/mod.rs](../packages/common/src/metrics/memory/mod.rs) - 355 lines
2. **Test infrastructure module**: [packages/common/src/metrics/memory/testing.rs](../packages/common/src/metrics/memory/testing.rs) - 629 lines
3. **Issue #1 resolved file**: [packages/core/src/plugins/bridge/handlers/processor.rs](../packages/core/src/plugins/bridge/handlers/processor.rs) - Lines 1-80 show event-driven architecture
4. **Test usage example**: [packages/common/tests/metrics/memory/mod_tests.rs](../packages/common/tests/metrics/memory/mod_tests.rs) - Shows MemoryMonitoringSystem usage in tests
5. **Existing cfg patterns**: [packages/ecs-progress/src/assets.rs:288](../packages/ecs-progress/src/assets.rs), [packages/ecs-progress/src/debug.rs:289](../packages/ecs-progress/src/debug.rs), [packages/ecs-search-aggregator/tests/integration_tests.rs:4](../packages/ecs-search-aggregator/tests/integration_tests.rs)
6. **Feature flag patterns**: [packages/common/src/metrics/memory/mod.rs:84-89](../packages/common/src/metrics/memory/mod.rs) - jemalloc-profiling and dhat-heap features
7. **Cargo.toml features**: [packages/common/Cargo.toml](../packages/common/Cargo.toml) - Lines 66-71 define existing features

### Search Commands Used

```bash
# Verify no production usage of test methods
rg "run_comprehensive_tests" packages/*/src/ -n
# Result: 0 matches in src/, only definition in mod.rs

# Verify MemoryMonitoringSystem only used in tests
rg "MemoryMonitoringSystem::new|MemoryMonitoringSystem::default" packages/*/src/ -n
# Result: 0 matches in src/, only in tests/

# Find conditional compilation patterns
rg "#\[cfg\(test\)\]" packages/ --type rust -n
# Result: 9 matches showing consistent test gating pattern

# Find testing module usage
rg "MemoryLeakTestSuite|leak_test_suite" packages/ --type rust -n
# Result: 156 lines in testing.rs and testing_tests.rs only
```

---

**Last Updated**: 2025-10-27  
**Implementation Status**: 50% Complete (1 of 2 issues resolved)  
**Ready for Implementation**: Yes - Proceed with mod.rs changes using Approach 1 (`#[cfg(test)]`)