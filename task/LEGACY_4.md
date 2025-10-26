# Task: Fix Raycast "Adapter" Not "Shim" Terminology

## OBJECTIVE
Replace all "shim" terminology with "adapter" in Raycast integration code. "Shim" implies a temporary workaround or hack, but this adapter layer is a permanent architectural component implementing the well-known Adapter design pattern (GoF). This change improves code clarity and helps developers understand that Raycast compatibility is a core, permanent feature.

## PRIORITY
P3 - MEDIUM - Code quality and clarity

## ARCHITECTURE CONTEXT

### What is the Adapter Pattern?
The Adapter pattern (Gang of Four design pattern) converts one interface into another interface that clients expect. It allows classes with incompatible interfaces to work together.

### Why "Adapter" Not "Shim"?
- **Adapter**: Permanent architectural component, well-established design pattern
- **Shim**: Temporary compatibility hack, suggests will be removed later
- **Reality**: Raycast integration is a permanent feature, not temporary

### Current Implementation
The Raycast adapter at `packages/core/src/raycast/adapter/` provides:
- Conversion between Raycast extension format and Action Items plugin interface
- JavaScript compatibility layer for Raycast API
- Configuration mapping and host function registration
- This is permanent architecture that enables Raycast extensions to run natively

## COMPLETE FILE INVENTORY

### Files Requiring Changes

#### 1. File Rename Required
**Location:** `packages/core/src/raycast/adapter/`
- **Current:** `api_shim.rs`
- **Target:** `api_adapter.rs`
- **Note:** This file already uses "adapter" terminology internally - only the filename is wrong

#### 2. Code Changes Required
1. [`packages/core/src/raycast/adapter/mod.rs`](../packages/core/src/raycast/adapter/mod.rs) - 2 changes
2. [`packages/core/src/raycast/adapter/implementation.rs`](../packages/core/src/raycast/adapter/implementation.rs) - 4 changes
3. [`packages/core/src/raycast/discovery.rs`](../packages/core/src/raycast/discovery.rs) - 1 change
4. [`packages/ecs-service-bridge/src/resources.rs`](../packages/ecs-service-bridge/src/resources.rs) - 1 change

## DETAILED IMPLEMENTATION GUIDE

### STEP 1: Verify Current State

First, confirm all "shim" references:

```bash
# From project root
cd /Volumes/samsung_t9/action-items

# Search for all "shim" references in relevant code
grep -rn "shim" packages/core/src/raycast/ --color=never
grep -rn "shim" packages/ecs-service-bridge/src/resources.rs --color=never

# Verify the file exists
ls -la packages/core/src/raycast/adapter/api_shim.rs
```

Expected output should show 7 references to "shim" in raycast code.

### STEP 2: Rename the File

**IMPORTANT:** Do this first to avoid breaking module references.

```bash
# From project root
cd packages/core/src/raycast/adapter/

# Rename the file
mv api_shim.rs api_adapter.rs

# Verify
ls -la api_adapter.rs
```

### STEP 3: Update Module Declaration - mod.rs

**File:** `packages/core/src/raycast/adapter/mod.rs`

**Change 1 (Line 7):**
```rust
// BEFORE:
pub use api_shim::*;

// AFTER:
pub use api_adapter::*;
```

**Change 2 (Line 16):**
```rust
// BEFORE:
mod api_shim;

// AFTER:
mod api_adapter;
```

**Complete context:**
```rust
//! Raycast Adapter - Modular Architecture
//!
//! Zero-allocation Raycast extension adapter with blazing-fast modular organization.
//! Converts Raycast extensions to our plugin interface format with full compatibility.

// Re-export all public items for backward compatibility
pub use api_adapter::*;  // <-- CHANGED
pub use configuration::*;
pub use conversion::*;
pub use host_functions::{
    HostFunction, HostFunctionRegistry, get_host_function_registry as create_host_functions,
};
pub use implementation::RaycastAdapter;

// Import the modular implementation
mod api_adapter;  // <-- CHANGED
mod configuration;
mod conversion;
mod host_functions;
mod implementation;
```

### STEP 4: Update Implementation - implementation.rs

**File:** `packages/core/src/raycast/adapter/implementation.rs`

**Change 1 (Line 19) - Update comment:**
```rust
// BEFORE:
#[allow(dead_code)] // Used by create_api_shim and WASM wrapper

// AFTER:
#[allow(dead_code)] // Used by create_api_adapter and WASM wrapper
```

**Change 2 (Line 20) - Update field name:**
```rust
// BEFORE:
raycast_api_shim_path: PathBuf,

// AFTER:
raycast_api_adapter_path: PathBuf,
```

**Change 3 (Line 26) - Update variable name:**
```rust
// BEFORE:
let raycast_api_shim_path = runtime_path.join("raycast-api-shim.js");

// AFTER:
let raycast_api_adapter_path = runtime_path.join("raycast-api-adapter.js");
```

**Change 4 (Line 30) - Update struct initialization:**
```rust
// BEFORE:
Self {
    deno_runtime_path: runtime_path,
    raycast_api_shim_path,
}

// AFTER:
Self {
    deno_runtime_path: runtime_path,
    raycast_api_adapter_path,
}
```

**Complete context for struct and constructor:**
```rust
/// Adapter to run Raycast extensions as WASM plugins using Deno
pub struct RaycastAdapter {
    #[allow(dead_code)] // Used by create_wasm_wrapper when implemented
    deno_runtime_path: PathBuf,
    #[allow(dead_code)] // Used by create_api_adapter and WASM wrapper  // <-- CHANGED
    raycast_api_adapter_path: PathBuf,  // <-- CHANGED
}

impl RaycastAdapter {
    /// Create new Raycast adapter with runtime path
    pub fn new(runtime_path: PathBuf) -> Self {
        let raycast_api_adapter_path = runtime_path.join("raycast-api-adapter.js");  // <-- CHANGED

        Self {
            deno_runtime_path: runtime_path,
            raycast_api_adapter_path,  // <-- CHANGED
        }
    }
    // ... rest of implementation
}
```

### STEP 5: Update Discovery Comment - discovery.rs

**File:** `packages/core/src/raycast/discovery.rs`

**Change (Line 44):**
```rust
// BEFORE:
// API shim creation is handled by the adapter internally

// AFTER:
// API adapter creation is handled by the adapter internally
```

**Complete context:**
```rust
// Initialize Raycast loader (clone repo if needed)
if !raycast_manager.initialized {
    match raycast_manager.loader.initialize() {
        Ok(_) => {
            // API adapter creation is handled by the adapter internally  // <-- CHANGED
            info!("Raycast extensions initialized successfully");
            raycast_manager.initialized = true;
        },
        Err(e) => {
            error!("Failed to initialize Raycast extensions: {}", e);
            return;
        },
    }
}
```

### STEP 6: Update Service Bridge Comment - resources.rs

**File:** `packages/ecs-service-bridge/src/resources.rs`

**Change (Line 58):**
```rust
// BEFORE:
// This method is a compatibility shim - actual registration should be done via

// AFTER:
// This method is a compatibility adapter - actual registration should be done via
```

**Complete context:**
```rust
/// Register a plugin with simple configuration (delegates to PluginRegistryResource)
pub fn register_plugin_simple(
    &self,
    _plugin_id: String,
    _name: String,
    _capabilities: Vec<Capability>,
) -> Result<(), String> {
    // This method is a compatibility adapter - actual registration should be done via  // <-- CHANGED
    // PluginRegistryResource
    Ok(())
}
```

### STEP 7: Verify Compilation

After making all changes, verify the code compiles:

```bash
# From project root
cd /Volumes/samsung_t9/action-items

# Check syntax and compilation
cargo check --package action-items-core

# If that passes, build the package
cargo build --package action-items-core

# Verify no "shim" references remain (should return no results in raycast code)
grep -rn "shim" packages/core/src/raycast/ --color=never
```

Expected: No results (or only results in comments explaining what changed)

### STEP 8: Verify Git Changes

```bash
# See what changed
git status

# Review the diff
git diff packages/core/src/raycast/
git diff packages/ecs-service-bridge/src/resources.rs

# Expected changes:
# - 1 file renamed: api_shim.rs -> api_adapter.rs
# - 2 lines in mod.rs
# - 4 lines in implementation.rs  
# - 1 line in discovery.rs
# - 1 line in resources.rs
```

## SUMMARY OF ALL CHANGES

| File | Line(s) | Type | Change |
|------|---------|------|--------|
| `adapter/api_shim.rs` | - | **FILE RENAME** | → `api_adapter.rs` |
| `adapter/mod.rs` | 7 | Code | `pub use api_shim::*;` → `pub use api_adapter::*;` |
| `adapter/mod.rs` | 16 | Code | `mod api_shim;` → `mod api_adapter;` |
| `adapter/implementation.rs` | 19 | Comment | `create_api_shim` → `create_api_adapter` |
| `adapter/implementation.rs` | 20 | Code | `raycast_api_shim_path` → `raycast_api_adapter_path` |
| `adapter/implementation.rs` | 26 | Code | `raycast_api_shim_path` → `raycast_api_adapter_path` |
| `adapter/implementation.rs` | 30 | Code | `raycast_api_shim_path,` → `raycast_api_adapter_path,` |
| `discovery.rs` | 44 | Comment | `API shim` → `API adapter` |
| `ecs-service-bridge/resources.rs` | 58 | Comment | `compatibility shim` → `compatibility adapter` |

**Total Changes:** 1 file rename + 8 text replacements across 5 files

## DEFINITION OF DONE

This task is complete when:

- [ ] File `api_shim.rs` has been renamed to `api_adapter.rs`
- [ ] All module declarations reference `api_adapter` instead of `api_shim`
- [ ] All struct fields use `api_adapter` naming instead of `api_shim`
- [ ] All comments use "adapter" terminology instead of "shim"
- [ ] JavaScript file reference changed from `raycast-api-shim.js` to `raycast-api-adapter.js`
- [ ] Code compiles successfully with `cargo check` and `cargo build`
- [ ] No "shim" references remain in Raycast-related code (excluding third-party libs in /tmp)
- [ ] Git diff shows only the expected 9 changes (1 rename + 8 replacements)

## CONSTRAINTS & SCOPE

### IN SCOPE
- Terminology changes only (shim → adapter)
- File rename (api_shim.rs → api_adapter.rs)
- Code comments and documentation strings
- Variable, field, and filename updates

### OUT OF SCOPE
- Functional changes to code behavior
- Architectural changes
- Performance optimizations
- Adding new features

### DO NOT
- Write or modify any tests
- Create benchmarks
- Write extensive documentation
- Change any logic or behavior
- Add logging or metrics
- Modify any third-party code in /tmp directory

## TECHNICAL NOTES

### Why This Matters
Using correct terminology helps developers:
1. Understand this is permanent architecture, not a temporary workaround
2. Recognize the Adapter design pattern being used
3. Avoid mistakenly trying to "remove the shim" thinking it's temporary
4. Properly document and discuss the integration architecture

### Runtime Impact
The JavaScript filename change (`raycast-api-shim.js` → `raycast-api-adapter.js`) is for a file that gets generated at runtime. This is not a file in the repository - it's created dynamically by the `create_api_adapter()` function in `api_adapter.rs`.

### Module Structure
The adapter module is well-organized:
```
raycast/adapter/
├── mod.rs              # Public interface and re-exports
├── api_adapter.rs      # JavaScript compatibility layer generation
├── configuration.rs    # Preference mapping
├── conversion.rs       # Extension to manifest conversion  
├── host_functions.rs   # Host function registry
└── implementation.rs   # Core RaycastAdapter struct
```

### Pattern Recognition
This implements the classic Adapter pattern:
- **Target Interface:** Action Items plugin interface (`PluginManifest`, etc.)
- **Adaptee:** Raycast extensions (TypeScript/JavaScript)
- **Adapter:** The code in `raycast/adapter/` that translates between them
- **Client:** The plugin loading and execution system

## REFERENCES

### Source Files
- [packages/core/src/raycast/adapter/api_shim.rs](../packages/core/src/raycast/adapter/api_shim.rs) - TO BE RENAMED
- [packages/core/src/raycast/adapter/mod.rs](../packages/core/src/raycast/adapter/mod.rs)
- [packages/core/src/raycast/adapter/implementation.rs](../packages/core/src/raycast/adapter/implementation.rs)
- [packages/core/src/raycast/discovery.rs](../packages/core/src/raycast/discovery.rs)
- [packages/ecs-service-bridge/src/resources.rs](../packages/ecs-service-bridge/src/resources.rs)

### Design Patterns
- Adapter Pattern (Gang of Four): Converts one interface to another
- This is structural pattern, not behavioral (it doesn't change what objects do, just how they're accessed)

## EXECUTION CHECKLIST

Follow these steps in order:

1. [ ] Create a new branch: `git checkout -b fix/raycast-adapter-terminology`
2. [ ] Verify current state with grep commands
3. [ ] Rename `api_shim.rs` to `api_adapter.rs`
4. [ ] Update `mod.rs` (2 changes)
5. [ ] Update `implementation.rs` (4 changes)
6. [ ] Update `discovery.rs` (1 change)
7. [ ] Update `resources.rs` (1 change)
8. [ ] Run `cargo check --package action-items-core`
9. [ ] Run `cargo build --package action-items-core`
10. [ ] Verify no "shim" references remain in raycast code
11. [ ] Review git diff to confirm only expected changes
12. [ ] Commit changes with descriptive message
13. [ ] Push branch and create PR if needed

## ESTIMATED EFFORT
**Time:** 15-20 minutes  
**Complexity:** LOW (straightforward find-and-replace refactoring)  
**Risk:** LOW (terminology only, no functional changes)
