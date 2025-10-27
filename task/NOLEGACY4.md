# Task: Fix Deno Library "Legacy" Comment

## OBJECTIVE

Update misleading "legacy" reference in Deno library comment - these are internal functions, not deprecated code.

---

## PRIORITY

**P2 - HIGH** - Misleading comments confuse developers about API design.

---

## SUBTASK 1: Update Deno-Ops Comment

**File**: `packages/ecs-deno/src/lib.rs`  
**Line**: 46

**What to Change**: Comment describing deno-ops functions

**Before**:
```rust
// Legacy deno-ops functions are now internal-only - use ECS events for external API:
// - ExtensionDiscoveryRequested -> ExtensionDiscoveryCompleted/Failed
// - DenoScriptExecutionRequested -> DenoScriptExecutionCompleted/Failed
```

**After**:
```rust
// Direct deno-ops functions are internal-only - use ECS events for external API:
// - ExtensionDiscoveryRequested -> ExtensionDiscoveryCompleted/Failed
// - DenoScriptExecutionRequested -> DenoScriptExecutionCompleted/Failed
```

**Why**: 
- These functions are internal implementation details, not legacy code
- The ECS event system is the public API surface
- This is API design (internal vs external), not deprecation
- "Direct" accurately describes low-level internal functions

---

## SUBTASK 2: Verify Changes

**Command**:
```bash
rg -i "legacy" --type rust packages/ecs-deno/src/lib.rs
```

**Expected Result**: Zero matches for "legacy" in this file

---

## SUBTASK 3: Verify Compilation

**Command**:
```bash
cargo check --workspace
```

**Expected Result**: Clean compilation with no errors

---

## DEFINITION OF DONE

- [ ] "Legacy deno-ops functions" changed to "Direct deno-ops functions"
- [ ] No "legacy" references in lib.rs deno-ops context
- [ ] Comment accurately describes internal vs external API design
- [ ] Code compiles successfully

---

## CONSTRAINTS

- DO NOT write unit tests
- DO NOT write benchmarks
- DO NOT create documentation files
- DO NOT change any functional code - only comment text
- DO verify compilation after changes

---

## RESEARCH NOTES

### Deno API Architecture

The Deno integration has two API layers:

1. **Direct deno-ops functions** (internal): Low-level functions that interact directly with Deno runtime
2. **ECS event system** (external): High-level public API using Bevy events

**Architecture reasoning**:
- Direct functions are implementation details - kept internal
- ECS events provide clean public interface with better decoupling
- This is intentional API design, not deprecation of old code

### Why This Matters

Calling internal functions "legacy":
- Suggests they should eventually be removed
- Confuses developers about API stability
- Obscures the intentional internal/external boundary
- Implies the design is temporary rather than intentional
