# Test Extraction Inventory - Implementation Guide

## Core Objective

**Extract all `#[cfg(test)]` modules from `src/` files into separate integration test files in the `tests/` directory.**

This transformation converts unit tests (compiled conditionally with source code) into integration tests (compiled as separate crates that test the public API). This enforces better API design, reduces compilation overhead, and separates test code from production code.

---

## Why Extract Tests to tests/ Directory?

### Rust Test Organization Philosophy

From [The Rust Book - Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html):

**Unit Tests (src/ with #[cfg(test)]):**
- Compiled conditionally alongside production code
- Can test private functions and internal implementation
- Use `#[cfg(test)]` to exclude from production builds
- Smaller, focused tests of individual components

**Integration Tests (tests/ directory):**
- Each file compiles as a separate crate
- Test only the public API (forces good API design)
- No `#[cfg(test)]` needed - Cargo handles this automatically
- Simulate how external code uses your library

### Benefits of Extraction

1. **Faster Production Builds**: Test code not compiled into production artifacts
2. **Better API Design**: Forces thinking about public interfaces
3. **Clear Separation**: Test code physically separated from implementation
4. **Reduced Binary Size**: No test infrastructure in release builds

---

## Complete Verified Inventory

### ✅ ecs-user-settings (COMPLETE - Reference Implementation)

Example of properly extracted tests:

- [x] [`src/table_names.rs`](../../packages/ecs-user-settings/src/table_names.rs) → [`tests/test_table_names.rs`](../../packages/ecs-user-settings/tests/test_table_names.rs)
- [x] [`src/types.rs`](../../packages/ecs-user-settings/src/types.rs) → [`tests/test_types.rs`](../../packages/ecs-user-settings/tests/test_types.rs)
- [x] `src/tests.rs` → [`tests/test_integration.rs`](../../packages/ecs-user-settings/tests/test_integration.rs) (deleted `src/tests.rs`)

---

### ✅ ecs-progress (COMPLETE)

- [x] [`src/utils.rs`](../../packages/ecs-progress/src/utils.rs#L288) → `tests/test_utils.rs`
  - Tests: `wait_frames`, `count_frames`, `constant_progress`, `always_complete`, `never_complete`, `wait_for_condition`, `count_successes`
  
- [x] [`src/entity.rs`](../../packages/ecs-progress/src/entity.rs#L218) → `tests/test_entity.rs`
  - Tests: `progress_entity_creation`, `progress_entity_completion`, `combined_fraction`
  - Includes custom test state: `TestState` enum

---

### ⏳ ecs-notifications (7+ files)

- [ ] [`src/components/content.rs`](../../packages/ecs-notifications/src/components/content.rs#L1110) → `tests/components/test_content.rs`
  - Tests: `notification_content_builder`, `rich_text_conversion`, `action_validation`

- [ ] [`src/components/platform.rs`](../../packages/ecs-notifications/src/components/platform.rs#L1184) → `tests/components/test_platform.rs`
  - Tests: `platform_capabilities`

- [ ] [`src/components/tracing.rs`](../../packages/ecs-notifications/src/components/tracing.rs#L333) → `tests/components/test_tracing.rs`
  - Tests: `tracing_context_creation`

- [ ] [`src/components/analytics.rs`](../../packages/ecs-notifications/src/components/analytics.rs#L1458) → `tests/components/test_analytics.rs`
  - Tests: `analytics_creation`

- [ ] [`src/components/lifecycle.rs`](../../packages/ecs-notifications/src/components/lifecycle.rs#L956) → `tests/components/test_lifecycle.rs`
  - Tests: `lifecycle_state_transitions`

- [ ] [`src/components/mod.rs`](../../packages/ecs-notifications/src/components/mod.rs#L649) → `tests/components/test_mod.rs`
  - Tests: `notification_id_generation`

**Note**: Additional test modules may exist in `systems.rs`, `integration.rs`, `manager.rs` - requires verification

---

### ⏳ ecs-fetch (9 files - CORRECTED INVENTORY)

- [ ] [`src/streaming.rs`](../../packages/ecs-fetch/src/streaming.rs#L776) → `tests/test_streaming.rs`
  - Tests: `streaming_config_defaults`, `stream_chunk`, `streaming_stats`, `stream_progress`, `chunk_metadata`

- [ ] [`src/tracing.rs`](../../packages/ecs-fetch/src/tracing.rs#L570) → `tests/test_tracing.rs`

- [ ] [`src/deduplication.rs`](../../packages/ecs-fetch/src/deduplication.rs#L513) → `tests/test_deduplication.rs`

- [ ] [`src/circuit_breaker.rs`](../../packages/ecs-fetch/src/circuit_breaker.rs#L404) → `tests/test_circuit_breaker.rs`
  - Tests: `circuit_breaker_closed_state`

- [ ] [`src/plugin.rs`](../../packages/ecs-fetch/src/plugin.rs#L550) → `tests/test_plugin.rs`
  - Tests: `http_plugin_default`

- [ ] [`src/prioritization.rs`](../../packages/ecs-fetch/src/prioritization.rs#L670) → `tests/test_prioritization.rs`
  - Tests: `prioritized_request_creation`

- [ ] [`src/auth/mod.rs`](../../packages/ecs-fetch/src/auth/mod.rs#L479) → `tests/auth/test_mod.rs`
  - Tests: `bearer_token_auth`

- [ ] [`src/middleware/mod.rs`](../../packages/ecs-fetch/src/middleware/mod.rs#L638) → `tests/middleware/test_mod.rs`
  - Tests: `compression_algorithm_header_values`

- [ ] [`src/metrics/mod.rs`](../../packages/ecs-fetch/src/metrics/mod.rs#L815) → `tests/metrics/test_mod.rs`
  - Tests: `latency_histogram`

---

### ⏳ core (action_items_core) (2 files)

- [ ] [`src/plugins/service_bridge_integration/permission_mapper.rs`](../../packages/core/src/plugins/service_bridge_integration/permission_mapper.rs#L247) → `tests/plugins/service_bridge_integration/test_permission_mapper.rs`

- [ ] [`src/plugins/service_bridge_integration/message_translator.rs`](../../packages/core/src/plugins/service_bridge_integration/message_translator.rs#L686) → `tests/plugins/service_bridge_integration/test_message_translator.rs`
  - Tests: `clipboard_read_translation`

---

## Step-by-Step Extraction Process

### Pattern: src/filename.rs → tests/test_filename.rs

Using [`ecs-user-settings/src/types.rs`](../../packages/ecs-user-settings/src/types.rs) as reference implementation.

### Step 1: Read the Source File Test Module

**Original in src/types.rs:**

```rust
#[inline]
pub fn parse_record_id(table: &str, key: &str) -> Result<RecordId, SettingsError> {
    validate_table_name(table)?;
    Ok(RecordId::from((table, key)))
}

#[cfg(test)]  // ← This entire block gets extracted
mod tests {
    use super::*;
    
    #[test]
    fn test_all_valid_tables_accepted() {
        for table in VALID_TABLES {
            assert!(validate_table_name(table).is_ok());
        }
    }
    
    #[test]
    fn test_invalid_table_rejected() {
        assert!(validate_table_name("invalid_table").is_err());
    }
    
    // ... more tests
}
```

### Step 2: Create tests/ Directory Structure

```bash
# Mirror src/ structure in tests/
mkdir -p tests/components/     # For src/components/*.rs
mkdir -p tests/auth/           # For src/auth/*.rs
mkdir -p tests/middleware/     # For src/middleware/*.rs
```

### Step 3: Write New Integration Test File

**New file: tests/test_types.rs**

```rust
//! Tests for types.rs

// Import from crate public API - MUST use full crate path
use action_items_ecs_user_settings::types::{validate_table_name, parse_record_id, VALID_TABLES};

#[test]  // ← No #[cfg(test)] needed - Cargo handles this
fn test_all_valid_tables_accepted() {
    for table in VALID_TABLES {
        assert!(validate_table_name(table).is_ok());
    }
}

#[test]
fn test_invalid_table_rejected() {
    assert!(validate_table_name("invalid_table").is_err());
    assert!(validate_table_name("users; DROP TABLE").is_err());
}

#[test]
fn test_record_id_construction() {
    let result = parse_record_id("user_preferences", "main");
    assert!(result.is_ok());
    
    let record_id = result.expect("should parse");
    assert_eq!(record_id.to_string(), "user_preferences:main");
}

// ... rest of tests
```

### Step 4: Clean Up Source File

**Modified src/types.rs:**

```rust
#[inline]
pub fn parse_record_id(table: &str, key: &str) -> Result<RecordId, SettingsError> {
    validate_table_name(table)?;
    Ok(RecordId::from((table, key)))
}

// ← REMOVE entire #[cfg(test)] mod tests { ... } block
// File ends here - no test code remains
```

### Step 5: Verify Extraction

```bash
# Run tests to ensure they still pass
cd packages/ecs-user-settings
cargo test

# Check that tests run from tests/ directory
cargo test --test test_types
```

---

## Critical Implementation Rules

### 1. Directory Mirroring

```
src/components/content.rs    → tests/components/test_content.rs
src/auth/mod.rs              → tests/auth/test_mod.rs
src/middleware/mod.rs        → tests/middleware/test_mod.rs
```

**Rule**: `tests/` structure MUST mirror `src/` structure exactly.

### 2. File Naming Convention

```
src/streaming.rs     → tests/test_streaming.rs
src/types.rs         → tests/test_types.rs
src/utils.rs         → tests/test_utils.rs
```

**Rule**: Prefix with `test_`, keep original filename.

### 3. Import Pattern Changes

**BEFORE (in src/ with #[cfg(test)]):**
```rust
#[cfg(test)]
mod tests {
    use super::*;  // ← Imports from parent module
    
    #[test]
    fn my_test() { ... }
}
```

**AFTER (in tests/ as integration test):**
```rust
//! Tests for filename.rs

// ← Use FULL crate path to public API
use action_items_ecs_progress::utils::{Progress, always_complete, never_complete};

#[test]  // ← No #[cfg(test)] wrapper
fn my_test() { ... }
```

### 4. Module Declaration Removal

**No module wrapper** - tests are top-level in the file:

```rust
// ❌ WRONG - Don't wrap in mod tests
#[cfg(test)]
mod tests {
    use crate::*;
    #[test]
    fn my_test() { }
}

// ✅ CORRECT - Tests are top-level
use action_items_ecs_fetch::streaming::StreamingConfig;

#[test]
fn my_test() { }
```

### 5. Handling Private Functions

If tests require private functions, you have two options:

**Option A**: Make the function `pub` (preferred for library crates)
```rust
// In src/types.rs
pub fn validate_table_name(table: &str) -> Result<(), Error> { ... }
```

**Option B**: Keep tests in src/ as unit tests (only if necessary)
- Use this only when testing truly internal implementation details
- Document why the test remains in src/

---

## Code Examples by Package

### ecs-progress Example

**Extract from:** [`src/utils.rs:288`](../../packages/ecs-progress/src/utils.rs#L288)

**Create:** `tests/test_utils.rs`

```rust
//! Tests for utils.rs

use action_items_ecs_progress::utils::{
    Progress, 
    always_complete,
    never_complete,
    constant_progress,
    wait_for_condition,
};
use bevy::ecs::schedule::Schedule;
use bevy::ecs::world::World;

#[test]
fn test_always_complete() {
    let progress = always_complete();
    assert!(progress.is_complete());
    assert_eq!(progress.fraction(), 1.0);
}

#[test]
fn test_never_complete() {
    let progress = never_complete();
    assert!(!progress.is_complete());
    assert_eq!(progress.fraction(), 0.0);
}

#[test]
fn test_constant_progress() {
    let progress = constant_progress::<3, 10>();
    assert_eq!(progress.done, 3);
    assert_eq!(progress.total, 10);
    assert_eq!(progress.fraction(), 0.3);
}

// ... rest of tests
```

### ecs-notifications Example

**Extract from:** [`src/components/content.rs:1110`](../../packages/ecs-notifications/src/components/content.rs#L1110)

**Create:** `tests/components/test_content.rs`

```rust
//! Tests for components/content.rs

use action_items_ecs_notifications::components::{
    NotificationContent,
    RichText,
    Priority,
    NotificationAction,
    ActionId,
    ActionStyle,
    ActivationType,
};

#[test]
fn test_notification_content_builder() {
    let content = NotificationContent::new("Test Title", RichText::plain("Test body"))
        .with_subtitle("Test subtitle")
        .with_priority(Priority::High)
        .with_custom_data("key1", "value1");

    assert_eq!(content.title, "Test Title");
    assert_eq!(content.subtitle, Some("Test subtitle".to_string()));
    assert_eq!(content.priority, Priority::High);
    assert_eq!(content.custom_data.get("key1"), Some(&"value1".to_string()));
}

#[test]
fn test_rich_text_conversion() {
    let plain = RichText::plain("Hello world");
    assert_eq!(plain.to_plain_text(), "Hello world");

    let markdown = RichText::markdown("**Bold** and *italic*");
    let plain_from_md = markdown.to_plain_text();
    assert_eq!(plain_from_md, "Bold and italic");
}
```

---

## Special Cases

### Case 1: Entire File is Tests (src/tests.rs)

If a file like `src/tests.rs` contains ONLY tests:

1. Extract tests to `tests/test_integration.rs`
2. **DELETE** `src/tests.rs` entirely
3. Remove from `src/lib.rs` if declared as module

**Example:** [`ecs-user-settings/tests/test_integration.rs`](../../packages/ecs-user-settings/tests/test_integration.rs) (formerly `src/tests.rs` - now deleted)

### Case 2: Tests in Nested Modules

For `src/auth/mod.rs` with tests:

```
src/auth/mod.rs  →  tests/auth/test_mod.rs
```

Maintain the directory hierarchy in tests/.

### Case 3: Tests Requiring Private State

Some tests in `ecs-progress/src/entity.rs` use:

```rust
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestState {
    #[default]
    Loading,
}
```

**Solution**: Move `TestState` to the test file - it's test-specific:

```rust
//! Tests for entity.rs

use action_items_ecs_progress::entity::ProgressEntity;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TestState {
    #[default]
    Loading,
}

#[test]
fn test_progress_entity_creation() {
    let entity = ProgressEntity::<TestState>::new()
        .with_progress(5, 10);
    // ...
}
```

---

## Common Mistakes to Avoid

### ❌ Mistake 1: Keeping #[cfg(test)] in tests/ files

```rust
// tests/test_utils.rs
#[cfg(test)]  // ❌ WRONG - Not needed in tests/ directory
mod tests {
    #[test]
    fn my_test() { }
}
```

### ❌ Mistake 2: Using use super::*

```rust
// tests/test_utils.rs
use super::*;  // ❌ WRONG - No parent module in integration tests
```

### ❌ Mistake 3: Not using full crate paths

```rust
// tests/test_utils.rs
use utils::Progress;  // ❌ WRONG - Not a valid path
```

```rust
// ✅ CORRECT
use action_items_ecs_progress::utils::Progress;
```

### ❌ Mistake 4: Forgetting directory structure

```rust
// src/components/content.rs → tests/test_content.rs  
// ❌ WRONG - Should be tests/components/test_content.rs
```

---

## Definition of Done

For each file extraction:

1. ✅ New test file created in `tests/` with correct naming and structure
2. ✅ All test functions copied with proper imports (full crate paths)
3. ✅ Tests compile without errors: `cargo test --package <package_name>`
4. ✅ All tests pass: `cargo test --package <package_name> --test <test_file>`
5. ✅ `#[cfg(test)]` module removed from source file
6. ✅ Source file compiles without test code: `cargo build --package <package_name>`
7. ✅ Git diff shows clean extraction (tests moved, not duplicated)

**Completion Criteria:**
- All listed files have tests extracted
- All packages pass `cargo test` in workspace
- No `#[cfg(test)]` blocks remain in src/ files (except where explicitly documented as necessary for private API testing)

---

## Summary Statistics

- **Total Packages:** 5 (ecs-user-settings, ecs-progress, ecs-notifications, ecs-fetch, core)
- **Completed Files:** 3 (ecs-user-settings package)
- **Remaining Files:** ~20+ (inventory corrected from original 19)
- **Verified Files:** All ecs-progress, ecs-fetch files; partial ecs-notifications, core

---

## References

- [Rust Book: Test Organization](https://doc.rust-lang.org/book/ch11-03-test-organization.html)
- [Cargo Book: Tests](https://doc.rust-lang.org/cargo/guide/tests.html)
- [Example: ecs-user-settings tests/](../../packages/ecs-user-settings/tests/)
- [Example: ecs-user-settings src/](../../packages/ecs-user-settings/src/)

---

**Last Updated:** 2025-10-27 (Augmented with implementation details and verified file inventory)
