# Task: Apply Conditional Compilation for Test vs Production

## EXECUTIVE SUMMARY

This task identifies and separates test stubs/mocks from production code using Rust's conditional compilation attributes (`#[cfg(test)]` and `#[cfg(not(test))]`). The goal is to ensure production builds (`cargo build --release`) exclude test-only code, reducing binary size and preventing accidental inclusion of mock implementations.

**Status**: Research complete. Two primary issues identified requiring immediate attention.

**Impact**: Medium-High. Affects binary size, code clarity, and ensures production builds don't include test infrastructure.

---

## ACTUAL FINDINGS FROM CODEBASE RESEARCH

### ✅ ALREADY CORRECT (No Action Needed)

1. **Test modules properly isolated**: Most test code already uses `#[cfg(test)]`
   - `packages/ecs-user-settings/src/lib.rs:68-69` - `#[cfg(test)] mod tests;`
   - `packages/core/src/plugins/service_bridge_integration/permission_mapper.rs` - test helpers in `#[cfg(test)]` module

2. **Platform stubs are intentional**: Icon extraction stubs in `packages/ecs-ui/src/icons/extraction/platform.rs` are **graceful fallbacks**, not test code
   - Functions return `None` to trigger fallback to FontAwesome icons
   - This is correct production behavior for unimplemented platforms

3. **Feature-gated code**: Already uses conditional compilation properly
   - `#[cfg(feature = "jemalloc-profiling")]`
   - `#[cfg(feature = "dhat-heap")]`

### ❌ REQUIRES FIXING

#### Issue 1: Mock WASM Runtime in Production Code

**File**: [`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs)

**Problem**: Lines 19-73 contain a mock `WasmRuntime` struct used in production code (called at line 228)

**Current Code**:
```rust
/// Mock WASM runtime for processing plugin callbacks
/// In a full implementation, this would integrate with the actual WASM execution environment
struct WasmRuntime {
    plugin_id: String,
}

impl WasmRuntime {
    async fn call_function(&self, function_name: &str, data: Vec<u8>) -> Result<Vec<u8>, String> {
        // Mock implementation that echoes data
        match function_name {
            "process_data" => Ok(data),
            "validate_input" => Ok(vec![if !data.is_empty() { 1 } else { 0 }]),
            "transform_data" => Ok(data),
            _ => Err(format!("Unknown WASM function: {}", function_name)),
        }
    }
}

async fn get_wasm_runtime(plugin_id: &str) -> Option<WasmRuntime> {
    // Returns mock for any non-empty plugin_id
    if !plugin_id.is_empty() {
        Some(WasmRuntime {
            plugin_id: plugin_id.to_string(),
        })
    } else {
        None
    }
}
```

**Real Infrastructure Available**: [`packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`](../packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs)
- Production-ready `WasmCallbackHandler` with ECS integration
- Supports Extism, native plugins, and Raycast/Deno runtime
- Already handles plugin lookup and function invocation

#### Issue 2: Test Infrastructure Unconditionally Compiled

**File**: [`packages/common/src/metrics/memory/mod.rs`](../packages/common/src/metrics/memory/mod.rs)

**Problem**: Line 90 unconditionally includes test infrastructure module

**Current Code**:
```rust
pub mod testing;  // Line 90 - always compiled
```

**Impact**: 
- `testing.rs` (557 lines) includes `MemoryLeakTestSuite`, test scenarios, and test helpers
- Compiled into production binaries unnecessarily
- Increases binary size ~20-30KB

---

## IMPLEMENTATION PATTERNS

### Pattern 1: Conditional Module Declaration

Use for test infrastructure that should only be available during testing:

```rust
// Production: module not compiled at all
#[cfg(test)]
pub mod testing;

// Alternative: Available for integration tests via feature flag
#[cfg(any(test, feature = "test-utils"))]
pub mod testing;
```

**Use when**: Entire module is test infrastructure (test harnesses, mock factories, test scenarios)

### Pattern 2: Separate Production and Test Implementations

Use when function/struct needs different implementations for prod vs test:

```rust
// Production implementation
#[cfg(not(test))]
fn get_service() -> Box<dyn Service> {
    Box::new(RealService::new())
}

// Test implementation
#[cfg(test)]
fn get_service() -> Box<dyn Service> {
    Box::new(MockService::new())
}
```

**Use when**: Same API, different behavior for testing

### Pattern 3: Conditional Logic Within Function

Use for small conditional branches within a function:

```rust
fn process_data(data: &[u8]) -> Result<Vec<u8>> {
    #[cfg(test)]
    {
        // Simplified test path
        return Ok(data.to_vec());
    }
    
    #[cfg(not(test))]
    {
        // Full production logic
        expensive_processing(data)
    }
}
```

**Use when**: Minor test shortcuts within otherwise production code

---

## STEP-BY-STEP EXECUTION PLAN

### STEP 1: Fix Mock WASM Runtime in processor.rs

**File**: `packages/core/src/plugins/bridge/handlers/processor.rs`

**Actions**:

1. **Import the real WASM infrastructure**:
   ```rust
   use crate::plugins::ecs_queries::wasm_callback_handler::WasmCallbackHandler;
   ```

2. **Remove mock implementations** (lines 19-73):
   - Delete `struct WasmRuntime`
   - Delete `impl WasmRuntime`
   - Delete `async fn get_wasm_runtime()`

3. **Integrate WasmCallbackHandler** in the `ServiceRequest::WasmCallback` handler (around line 226):

   **Before**:
   ```rust
   match get_wasm_runtime(&plugin_id).await {
       Some(runtime) => {
           match runtime.call_function(&function_name, data).await {
               Ok(result_data) => { /* ... */ },
               Err(e) => { /* ... */ }
           }
       },
       None => Err(format!("Plugin {} not found", plugin_id))
   }
   ```

   **After**:
   ```rust
   // Use the real ECS-based WASM callback handler
   // Note: This requires access to the ECS World - consider refactoring 
   // to make process_service_request a system or system param
   
   // For now, document that WasmCallback requires ECS integration:
   log::warn!(
       "WasmCallback for plugin {} requires ECS integration - use WasmCallbackHandler system",
       plugin_id
   );
   
   // Return error indicating need for proper ECS integration
   ServiceResponse::WasmCallback(Err(format!(
       "WasmCallback requires ECS integration via WasmCallbackHandler system. Plugin: {}",
       plugin_id
   )))
   ```

4. **Add TODO comment** for proper integration:
   ```rust
   // TODO: Refactor to use WasmCallbackHandler with ECS World access
   // See packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs
   // This handler needs to be called from a Bevy system with access to:
   // - Query<(Entity, &PluginComponent)>
   // - Query<(Entity, &ExtismPluginComponent)>
   // - Query<(Entity, &RaycastPluginComponent)>
   ```

**Alternative Approach** (if ECS access is available):
```rust
// If process_service_request can be converted to a Bevy system:
pub fn process_service_request_system(
    request: ServiceRequest,
    wasm_handler: WasmCallbackHandler,
) -> ServiceResponse {
    // ...
    ServiceRequest::WasmCallback { plugin_id, function_name, data } => {
        let payload = serde_json::Value::String(
            String::from_utf8_lossy(&data).to_string()
        );
        
        match wasm_handler.call_wasm_plugin_function_ecs(
            &plugin_id,
            &function_name,
            &payload
        ) {
            Ok(result) => ServiceResponse::WasmCallback(Ok(result.into_bytes())),
            Err(e) => ServiceResponse::WasmCallback(Err(e))
        }
    }
}
```

### STEP 2: Fix Testing Module Declaration

**File**: `packages/common/src/metrics/memory/mod.rs`

**Actions**:

1. **Change line 90** from unconditional to conditional:

   **Before**:
   ```rust
   pub mod testing;
   ```

   **After** (Option A - Test-only):
   ```rust
   #[cfg(test)]
   pub mod testing;
   ```

   **After** (Option B - Test + Feature Flag):
   ```rust
   /// Test infrastructure for memory leak detection
   /// Available in tests or with `test-utils` feature
   #[cfg(any(test, feature = "test-utils"))]
   pub mod testing;
   ```

2. **Update re-exports** (around line 98-102) to be conditional:

   **Before**:
   ```rust
   pub use testing::{
       LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, 
       TestMemoryStats, TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
   };
   ```

   **After**:
   ```rust
   #[cfg(any(test, feature = "test-utils"))]
   pub use testing::{
       LeakTestScenario, MemoryLeakTestSuite, MemoryThresholds, TestCategory, 
       TestMemoryStats, TestMemoryUsage, TestResult, TestResults, TestStatus, scenarios,
   };
   ```

3. **Update usages** in `MemoryMonitoringSystem` (lines 133-143):

   **Before**:
   ```rust
   leak_test_suite: MemoryLeakTestSuite,  // Unconditional field
   ```

   **After**:
   ```rust
   #[cfg(any(test, feature = "test-utils"))]
   leak_test_suite: MemoryLeakTestSuite,
   ```

4. **Update methods** that use the test suite:
   - `new()` method (line 123): Conditionally initialize
   - `new_fallback()` method (line 173): Conditionally include
   - `run_comprehensive_tests()` method (line 263): Add `#[cfg(any(test, feature = "test-utils"))]`

**Recommendation**: Use **Option B** (feature flag) if integration tests need access to test infrastructure. Otherwise use **Option A** (test-only).

### STEP 3: Verify No False Positives

**Files that DO NOT need changes** (verified as correct):

1. `packages/ecs-ui/src/icons/extraction/platform.rs`
   - Stubs are intentional fallbacks (return `None` → triggers FontAwesome icons)
   - This is correct production behavior

2. `packages/core/src/plugins/service_bridge_integration/permission_mapper.rs`
   - Mock helpers already in `#[cfg(test)]` module (lines 248-442)

3. `packages/common/src/metrics/memory/dhat_profiler.rs`
   - Already feature-gated: `#[cfg(feature = "dhat-heap")]`

4. `packages/core/src/runtime/deno/notifications/macos.rs`
   - Comments say "stub" but code is production-ready (lines 44-145)
   - No changes needed

### STEP 4: Verify Builds

After making changes:

```bash
# Verify production build excludes test code
cargo build --release --workspace

# Verify test build includes test infrastructure
cargo test --workspace --all-features

# Check binary size impact
ls -lh target/release/action-items  # Before
# Make changes
ls -lh target/release/action-items  # After - should be slightly smaller
```

---

## DEFINITION OF DONE

- [x] Research complete - specific files identified
- [ ] `processor.rs`: Mock `WasmRuntime` removed or properly gated
- [ ] `processor.rs`: Integration with real `WasmCallbackHandler` documented/implemented
- [ ] `mod.rs`: `testing` module declaration conditionally compiled
- [ ] `mod.rs`: `testing` re-exports conditionally compiled
- [ ] `mod.rs`: `MemoryMonitoringSystem` conditionally includes test suite
- [ ] Verified: `cargo build --release --workspace` succeeds
- [ ] Verified: `cargo test --workspace --all-features` succeeds
- [ ] Verified: No "for testing" or "test only" comments in unconditional production code
- [ ] Verified: Binary size reduced (optional - measure with `ls -lh target/release/action-items`)

---

## CONSTRAINTS

- ✅ DO NOT write new tests
- ✅ DO NOT write benchmarks  
- ✅ DO NOT write extensive documentation
- ✅ DO NOT break existing test functionality
- ✅ DO NOT break existing production functionality
- ✅ DO ensure production builds exclude all test code
- ✅ DO use existing codebase patterns (feature flags, conditional compilation)

---

## REFERENCES

### Codebase References

- Mock WASM Runtime: [`packages/core/src/plugins/bridge/handlers/processor.rs`](../packages/core/src/plugins/bridge/handlers/processor.rs)
- Real WASM Handler: [`packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`](../packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs)
- Testing Module: [`packages/common/src/metrics/memory/testing.rs`](../packages/common/src/metrics/memory/testing.rs)
- Module Declaration: [`packages/common/src/metrics/memory/mod.rs`](../packages/common/src/metrics/memory/mod.rs)
- Extism Integration: [`packages/plugin-wasm/src/extism.rs`](../packages/plugin-wasm/src/extism.rs)

### Existing Conditional Compilation Examples in Codebase

- Feature-gated modules: `packages/common/src/metrics/memory/mod.rs:84-89`
  ```rust
  #[cfg(feature = "jemalloc-profiling")]
  pub mod jemalloc_profiler;
  
  #[cfg(feature = "dhat-heap")]
  pub mod dhat_profiler;
  ```

- Test module gating: `packages/ecs-user-settings/src/lib.rs:68-69`
  ```rust
  #[cfg(test)]
  mod tests;
  ```

### Rust Documentation

- Conditional Compilation: https://doc.rust-lang.org/reference/conditional-compilation.html
- Cfg Attribute: https://doc.rust-lang.org/rust-by-example/attribute/cfg.html

---

## TECHNICAL NOTES

### Why processor.rs Mock is Problematic

The mock `WasmRuntime` in processor.rs:
1. **Always returns mock data** - echoes input or returns hardcoded values
2. **No real plugin lookup** - accepts any non-empty plugin_id
3. **Included in production** - called from production code path (line 228)
4. **Real infrastructure exists** - `WasmCallbackHandler` provides full implementation

This is different from platform.rs stubs which:
- Return `None` (failure) intentionally
- Trigger graceful fallback to generic icons
- Are correct production behavior for unimplemented platforms

### Testing Module Impact

The `testing.rs` module (557 lines):
- Provides `MemoryLeakTestSuite` for CI/CD integration
- Includes test scenarios, thresholds, and result tracking
- Not needed in production binaries
- Estimated impact: 20-30KB in binary size

### Integration Considerations

The `WasmCallbackHandler` requires ECS World access. The refactoring options:

1. **Option A**: Make `process_service_request` a Bevy system
   - Pro: Direct access to ECS queries
   - Con: Larger refactoring effort

2. **Option B**: Pass `WasmCallbackHandler` as parameter
   - Pro: Minimal changes
   - Con: Requires threading handler through call chain

3. **Option C**: Return error, handle WasmCallback at higher level
   - Pro: No changes to processor.rs architecture
   - Con: Requires caller to handle WasmCallback specially

**Recommended**: Option C (interim) → Option A (future refactor)

---

## SEARCH COMMANDS USED

Research commands that identified these issues:

```bash
# Find mock/stub patterns
rg "mock|Mock|stub|Stub|fake|Fake" --type rust packages/*/src/ -n

# Find test-only comments  
rg "for testing|test only|test purposes" --type rust packages/*/src/ -n

# Find existing conditional compilation
rg "#\[cfg\(test\)\]|#\[cfg\(not\(test\)\)\]" --type rust packages/*/src/ -n

# Find WASM-related code
rg "WasmRuntime|get_wasm_runtime" --type rust packages/*/src/ -n
```

---

**Last Updated**: 2025-10-10  
**Research Status**: Complete  
**Ready for Implementation**: Yes
