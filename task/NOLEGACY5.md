# Task: Fix Window Activation "Legacy" References

## OBJECTIVE

Update misleading "legacy" references in window activation code - this is a limited-functionality API variant, not deprecated code.

---

## PRIORITY

**P2 - HIGH** - Misleading comments and log messages discourage use of valid API.

---

## SUBTASK 1: Update Function Documentation

**File**: `packages/app/src/window/activation/manager.rs`  
**Line**: 84

**What to Change**: Function documentation comment

**Before**:
```rust
/// Legacy function for backwards compatibility
pub fn activate_window(window: &mut Window) {
```

**After**:
```rust
/// Activate window without handle (limited functionality)
///
/// Prefer `activate_window_with_handle` which provides full activation control.
/// This variant has limited functionality as it cannot target a specific window.
pub fn activate_window(window: &mut Window) {
```

**Why**: 
- This is a valid limited-functionality variant, not legacy code
- Some use cases don't need window handle targeting
- Documentation should describe what it does, not discourage its use

---

## SUBTASK 2: Update Warning Message

**File**: `packages/app/src/window/activation/manager.rs`  
**Line**: 87 (inside activate_window function)

**What to Change**: Warning log message

**Before**:
```rust
warn!(
    "Using legacy activate_window function - consider migrating to the \
     window_activation_system"
);
```

**After**:
```rust
warn!(
    "activate_window called without window handle - functionality limited. \
     Use activate_window_with_handle for full control"
);
```

**Why**: 
- Removes "legacy" terminology from runtime logs
- Clearly describes the limitation (no window handle)
- Suggests alternative without implying deprecation

---

## SUBTASK 3: Update Debug Message

**File**: `packages/app/src/window/activation/manager.rs`  
**Line**: 94 (inside activate_window function)

**What to Change**: Debug log message

**Before**:
```rust
debug!("Legacy window activation completed - functionality limited without window handle");
```

**After**:
```rust
debug!("Window activation completed (limited functionality without handle)");
```

**Why**: 
- Removes "legacy" from runtime logging
- More concise and accurate
- Describes the limitation without deprecation implication

---

## SUBTASK 4: Verify Changes

**Command**:
```bash
rg -i "legacy" --type rust packages/app/src/window/activation/manager.rs
```

**Expected Result**: Zero matches for "legacy" in window activation context

---

## SUBTASK 5: Verify Compilation

**Command**:
```bash
cargo check --workspace
```

**Expected Result**: Clean compilation with no errors

---

## DEFINITION OF DONE

- [ ] Line 84: Function docs updated to describe limited functionality (not "legacy")
- [ ] Line 87: Warning message updated to remove "legacy" reference
- [ ] Line 94: Debug message updated to remove "legacy" reference
- [ ] No "legacy" references in window activation code
- [ ] Code compiles successfully

---

## CONSTRAINTS

- DO NOT write unit tests
- DO NOT write benchmarks
- DO NOT create documentation files
- DO NOT change any functional code - only comments and log messages
- DO verify compilation after changes

---

## RESEARCH NOTES

### Window Activation API Variants

Two activation functions exist:

1. **`activate_window_with_handle(winit_window)`**: Full activation with specific window targeting
2. **`activate_window(window)`**: Limited activation without handle - sets visibility flags only

**Why both exist**:
- Handle variant provides platform-specific window targeting
- No-handle variant is simpler for cases that don't need targeting
- This is API design (full vs limited), not deprecation

### Why This Matters

Calling the limited variant "legacy":
- Discourages valid use cases that don't need handle targeting
- Suggests it should be removed when it serves a purpose
- Generates misleading warning logs in production
- Confuses developers about which function to use
