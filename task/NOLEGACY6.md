# Task: Fix Plugin Manager Raycast "Legacy" Comment

## OBJECTIVE

Update misleading "legacy" reference in plugin manager - Raycast format support is active compatibility, not deprecated code.

---

## PRIORITY

**P2 - HIGH** - Misleading comments suggest Raycast format support is deprecated when it's intentional compatibility.

---

## SUBTASK 1: Update load_package_json_manifest Documentation

**File**: `packages/core/src/runtime/deno/plugin_manager.rs`  
**Line**: 289

**What to Change**: Function documentation comment

**Before**:
```rust
/// Load legacy package.json manifest for Raycast compatibility
fn load_package_json_manifest(
    &self,
    package_json_path: &Path,
) -> Result<PluginManifest, String> {
```

**After**:
```rust
/// Load Raycast-format package.json manifest for compatibility
fn load_package_json_manifest(
    &self,
    package_json_path: &Path,
) -> Result<PluginManifest, String> {
```

**Why**: 
- Supporting Raycast plugin format is deliberate ecosystem compatibility
- Raycast actively uses package.json manifests - not legacy
- This is format compatibility, not deprecation
- "Raycast-format" is more accurate and descriptive

---

## SUBTASK 2: Verify Changes

**Command**:
```bash
rg -i "legacy" --type rust packages/core/src/runtime/deno/plugin_manager.rs
```

**Expected Result**: Zero matches for "legacy" in plugin manifest loading context

---

## SUBTASK 3: Verify Compilation

**Command**:
```bash
cargo check --workspace
```

**Expected Result**: Clean compilation with no errors

---

## DEFINITION OF DONE

- [ ] "legacy package.json manifest" changed to "Raycast-format package.json manifest"
- [ ] No "legacy" references in Raycast compatibility context
- [ ] Comment accurately describes format compatibility
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

### Raycast Plugin Format

The application supports multiple plugin manifest formats:
- **Native format**: Our own plugin manifest structure
- **Raycast format**: package.json-based manifest (active Raycast ecosystem format)

**Why support Raycast format**:
- Compatibility with existing Raycast plugin ecosystem
- Allows users to run Raycast plugins
- Raycast still actively uses this format
- Intentional interoperability, not legacy support

### Why This Matters

Calling Raycast format "legacy":
- Suggests Raycast ecosystem is deprecated (it's not)
- Implies we plan to drop this compatibility
- Confuses developers about support status
- Makes intentional compatibility sound like technical debt
