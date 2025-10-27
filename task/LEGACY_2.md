# Task: Remove "Legacy" Dead Code and Fix Misleading "Legacy" Comments

## STATUS: 1/10 - SEVERELY INCOMPLETE

Only 1 out of 7 files partially addressed. Critical dead code deletion NOT implemented.

---

## REMAINING WORK - 6 FILES

### 1. ⚠️ CRITICAL: DELETE TLS Dead Code Methods

**File**: `packages/ecs-tls/src/tls/tls_config.rs`  
**Lines to DELETE**: 286-303

**Action**: DELETE both methods entirely:
- `start_ocsp_cleanup_task()` 
- `start_crl_cleanup_task()`

These methods are marked `#[allow(dead_code)]`, are never called, and do nothing but log a message. They are vestigial code that should be removed completely.

**Verification**:
```bash
# Should find ZERO results after deletion
rg "start_ocsp_cleanup_task|start_crl_cleanup_task" --type rust packages/
```

---

### 2. Fix Privacy Systems "Legacy" Comments (3 locations)

**File**: `packages/ui/src/ui/ai_menu/privacy_systems.rs`

**Line 190** - Function documentation:
```rust
// BEFORE:
/// Legacy system for backward compatibility with non-gradient privacy icons

// AFTER:
/// Fallback system for non-gradient privacy icons
```

**Line 308** - Comment in calculate_indicator_colors:
```rust
// BEFORE:
/// Maintained for backward compatibility with legacy privacy icons

// AFTER:
/// Maintained as fallback for non-gradient privacy icon rendering
```

**Line 399** - Comment in system chain:
```rust
// BEFORE:
// Legacy color-based updates (fallback)

// AFTER:
// Fallback color-based updates (for non-gradient icons)
```

---

### 3. Fix Deno Library Comment

**File**: `packages/ecs-deno/src/lib.rs`  
**Line**: 46

```rust
// BEFORE:
// Legacy deno-ops functions are now internal-only - use ECS events for external API:

// AFTER:
// Direct deno-ops functions are internal-only - use ECS events for external API:
```

---

### 4. Fix Window Activation Documentation (3 locations)

**File**: `packages/app/src/window/activation/manager.rs`

**Line 84** - Function documentation:
```rust
// BEFORE:
/// Legacy function for backwards compatibility
pub fn activate_window(window: &mut Window) {

// AFTER:
/// Activate window without handle (limited functionality)
///
/// Prefer `activate_window_with_handle` which provides full activation control.
/// This variant has limited functionality as it cannot target a specific window.
pub fn activate_window(window: &mut Window) {
```

**Line 87** - Warning message:
```rust
// BEFORE:
warn!(
    "Using legacy activate_window function - consider migrating to the \
     window_activation_system"
);

// AFTER:
warn!(
    "activate_window called without window handle - functionality limited. \
     Use activate_window_with_handle for full control"
);
```

**Line 94** - Debug message:
```rust
// BEFORE:
debug!("Legacy window activation completed - functionality limited without window handle");

// AFTER:
debug!("Window activation completed (limited functionality without handle)");
```

---

### 5. Fix Plugin Manager Raycast Comment

**File**: `packages/core/src/runtime/deno/plugin_manager.rs`  
**Line**: 289

```rust
// BEFORE:
/// Load legacy package.json manifest for Raycast compatibility

// AFTER:
/// Load Raycast-format package.json manifest for compatibility
```

---

### 6. Fix Linux Wayland Protocol Comments (2 locations)

**File**: `packages/app/src/window/focused_window/linux.rs`

**Line 269** - Struct field comment:
```rust
// BEFORE:
output_geometry: std::collections::HashMap<wl_output::WlOutput, (i32, i32, i32, i32)>, /* output -> (x, y, width, height) - legacy */

// AFTER:
output_geometry: std::collections::HashMap<wl_output::WlOutput, (i32, i32, i32, i32)>, // Fallback for when output management protocol unavailable
```

**Line 345** - Event dispatch comment:
```rust
// BEFORE:
// Legacy wl_output events are handled by output management protocol

// AFTER:
// wl_output events handled by output management protocol when available
```

---

## IMPLEMENTATION CHECKLIST

- [ ] Delete TLS dead code methods (lines 286-303) - `packages/ecs-tls/src/tls/tls_config.rs`
- [ ] Fix 3 privacy system comments - `packages/ui/src/ui/ai_menu/privacy_systems.rs`
- [ ] Fix Deno lib comment - `packages/ecs-deno/src/lib.rs`
- [ ] Fix 3 window activation messages - `packages/app/src/window/activation/manager.rs`
- [ ] Fix plugin manager comment - `packages/core/src/runtime/deno/plugin_manager.rs`
- [ ] Fix 2 Wayland protocol comments - `packages/app/src/window/focused_window/linux.rs`
- [ ] Verify compilation: `cargo check --workspace`
- [ ] Verify dead code removed: `rg "start_ocsp_cleanup_task|start_crl_cleanup_task" --type rust`

---

## WHY THIS MATTERS

**Dead Code**: Methods marked `#[allow(dead_code)]` that do nothing are technical debt and confuse developers.

**"Legacy" Misuse**: Calling active fallback systems "legacy" makes them sound deprecated when they're actually necessary compatibility layers.

---

## CONSTRAINTS

- DO NOT write unit tests
- DO NOT write benchmarks  
- DO NOT create documentation files
- DO delete the actual dead code
- DO verify compilation after changes

---

## COMPLETED ITEMS

✅ **Permission Screen Description** (`packages/ecs-permissions/src/wizard/ui/permission_screens.rs:213`)  
Changed from "legacy contact management features" to "contact management features" - acceptable.
