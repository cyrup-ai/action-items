# Task: Fix Wayland Protocol "Legacy" Comments

## OBJECTIVE

Update misleading "legacy" references in Wayland protocol code - wl_output is the base protocol fallback, not deprecated code.

---

## PRIORITY

**P2 - HIGH** - Misleading comments suggest standard Wayland protocol is deprecated.

---

## SUBTASK 1: Fix WaylandState Struct Field Comment

**File**: `packages/app/src/window/focused_window/linux.rs`  
**Line**: 269 (approximately - in WaylandState struct)

**What to Change**: Inline comment on output_geometry field

**Before**:
```rust
struct WaylandState {
    focused_toplevel: Option<ToplevelInfo>,
    toplevels: Vec<ToplevelInfo>,
    output_heads: std::collections::HashMap<String, OutputInfo>,
    output_name_to_output: std::collections::HashMap<String, wl_output::WlOutput>,
    output_geometry: std::collections::HashMap<wl_output::WlOutput, (i32, i32, i32, i32)>, /* output -> (x, y, width, height) - legacy */
    ready: bool,
}
```

**After**:
```rust
struct WaylandState {
    focused_toplevel: Option<ToplevelInfo>,
    toplevels: Vec<ToplevelInfo>,
    output_heads: std::collections::HashMap<String, OutputInfo>,
    output_name_to_output: std::collections::HashMap<String, wl_output::WlOutput>,
    output_geometry: std::collections::HashMap<wl_output::WlOutput, (i32, i32, i32, i32)>, // Fallback for when output management protocol unavailable
    ready: bool,
}
```

**Why**: 
- wl_output is the base Wayland protocol - always available
- Output management protocol is a newer extension (not always present)
- This is protocol fallback, not deprecation
- Accurate description of why this field exists

---

## SUBTASK 2: Fix Dispatch Implementation Comment

**File**: `packages/app/src/window/focused_window/linux.rs`  
**Line**: 345 (approximately - in wl_output Dispatch impl)

**What to Change**: Comment in event dispatch implementation

**Before**:
```rust
impl Dispatch<wl_output::WlOutput, ()> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _output: &wl_output::WlOutput,
        _event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // Legacy wl_output events are handled by output management protocol
        // This dispatch is required but we don't process these events directly
    }
}
```

**After**:
```rust
impl Dispatch<wl_output::WlOutput, ()> for WaylandQueueState {
    fn event(
        _state: &mut Self,
        _output: &wl_output::WlOutput,
        _event: wl_output::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        // wl_output events handled by output management protocol when available
        // This dispatch is required but we don't process these events directly
    }
}
```

**Why**: 
- wl_output isn't legacy - it's the base protocol
- Output management protocol is used when available (extension)
- Comment should reflect "when available" conditional, not deprecation

---

## SUBTASK 3: Verify Changes

**Command**:
```bash
rg -i "legacy" --type rust packages/app/src/window/focused_window/linux.rs
```

**Expected Result**: Zero matches for "legacy" in Wayland protocol context

---

## SUBTASK 4: Verify Compilation

**Command**:
```bash
cargo check --workspace
```

**Expected Result**: Clean compilation with no errors

---

## DEFINITION OF DONE

- [ ] Line ~269: "legacy" changed to "Fallback for when output management protocol unavailable"
- [ ] Line ~345: "Legacy wl_output events" changed to "wl_output events handled by... when available"
- [ ] No "legacy" references in Wayland protocol code
- [ ] Code compiles successfully

---

## CONSTRAINTS

- DO NOT write unit tests
- DO NOT write benchmarks
- DO NOT create documentation files
- DO NOT change any functional code - only comments
- DO verify compilation after changes

---

## RESEARCH NOTES

### Wayland Protocol Layers

Wayland has layered protocol support:

1. **wl_output** (base protocol): Core protocol, always available, basic output info
2. **output_management protocol** (extension): Additional protocol with better functionality, may not be available

**Architecture**:
- Code uses output management protocol when available (preferred)
- Falls back to wl_output when extension unavailable (always works)
- Both are active - not "legacy vs modern" but "base vs extension"

### Why This Matters

Calling wl_output "legacy":
- Suggests the core Wayland protocol is deprecated (it's not)
- Confuses protocol versioning with deprecation
- Makes necessary fallback code seem like technical debt
- Obscures intentional protocol layering design
