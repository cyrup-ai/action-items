# PRODFIX_7: Fix Deno Script Execution Result Handling

## OBJECTIVE
Replace hardcoded "Script executed successfully" response with actual script execution results by properly extracting and serializing v8::Value from Deno runtime.

## PRIORITY
**P1 - HIGH (Data Integrity Violation)**

## LOCATION
`packages/ecs-deno/src/resources.rs`

## CURRENT STATE
Line 394 returns fake success message instead of actual script results. The TODO comment indicates the v8 API has changed in newer deno_core versions and proper extraction is not implemented.

Callers receive fake data, making script execution results unreliable.

## SUBTASK 1: Analyze Current Code
Read the context around line 394 to understand the current implementation.

**Changes needed:**
- Read `packages/ecs-deno/src/resources.rs` lines 380-410
- Identify the type of `global_value` (likely `v8::Global<v8::Value>`)
- Check what deno_core version is being used
- Review deno_core API docs for current v8 value extraction methods

## SUBTASK 2: Implement v8::Value Extraction
Convert `v8::Global<v8::Value>` to local handle in the current scope.

**Changes needed at line 394:**
```rust
Ok(global_value) => {
    // Get a handle scope for v8 operations
    let scope = &mut runtime.handle_scope();

    // Convert Global to Local handle
    let local = v8::Local::new(scope, global_value);

    // Now we can work with the value
    // Continue to SUBTASK 3
}
```

**Key points:**
- Must have active v8 scope to work with values
- `v8::Local::new()` converts Global to Local handle
- Local handles are tied to current scope lifetime

## SUBTASK 3: Serialize v8::Value to JSON
Convert the v8::Value to JSON string representation.

**Changes needed:**
```rust
Ok(global_value) => {
    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, global_value);

    // Serialize to JSON
    let json_value = match v8::json::stringify(scope, local) {
        Some(json_str) => json_str.to_rust_string_lossy(scope),
        None => {
            return Err(DenoError::SerializationFailed(
                "Failed to serialize script result to JSON".into()
            ));
        }
    };

    Ok(json_value)
}
```

## SUBTASK 4: Handle Different Value Types
Add proper handling for different v8::Value types (primitives, objects, null, undefined).

**Changes needed:**
```rust
Ok(global_value) => {
    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, global_value);

    // Handle different value types
    let result_string = if local.is_undefined() || local.is_null() {
        "null".to_string()
    } else if local.is_string() {
        // Direct string extraction
        local.to_string(scope)
            .ok_or_else(|| DenoError::SerializationFailed("Failed to convert string".into()))?
            .to_rust_string_lossy(scope)
    } else if local.is_number() || local.is_boolean() {
        // Primitives can be stringified directly
        v8::json::stringify(scope, local)
            .ok_or_else(|| DenoError::SerializationFailed("Failed to stringify primitive".into()))?
            .to_rust_string_lossy(scope)
    } else {
        // Objects, arrays, etc. - use JSON serialization
        v8::json::stringify(scope, local)
            .ok_or_else(|| DenoError::SerializationFailed(
                "Failed to serialize complex value to JSON".into()
            ))?
            .to_rust_string_lossy(scope)
    };

    Ok(result_string)
}
```

## SUBTASK 5: Add Error Handling for Circular References
Handle cases where objects contain circular references that cannot be JSON serialized.

**Changes needed:**
```rust
Ok(global_value) => {
    let scope = &mut runtime.handle_scope();
    let local = v8::Local::new(scope, global_value);

    let result_string = match v8::json::stringify(scope, local) {
        Some(json_str) => json_str.to_rust_string_lossy(scope),
        None => {
            // Serialization failed - likely circular reference or non-serializable value
            warn!("Failed to serialize script result, attempting fallback");

            // Fallback: use v8 string representation
            match local.to_string(scope) {
                Some(str_repr) => str_repr.to_rust_string_lossy(scope),
                None => {
                    return Err(DenoError::SerializationFailed(
                        "Script result is not serializable (possible circular reference)".into()
                    ));
                }
            }
        }
    };

    Ok(result_string)
}
```

## SUBTASK 6: Update deno_core Version (if needed)
Check if deno_core needs updating and update if the API has improved.

**Changes needed:**
- Check `packages/ecs-deno/Cargo.toml` for current deno_core version
- Review deno_core changelog for API improvements
- If newer version has better APIs, update dependency
- Test that updated version compiles and works correctly

## SUBTASK 7: Update Error Types
Ensure DenoError has appropriate variant for serialization failures.

**Changes needed in** `packages/ecs-deno/src/error.rs` **or similar:**
```rust
pub enum DenoError {
    // ... existing variants
    SerializationFailed(String),
    // ... other variants
}
```

## DEFINITION OF DONE
- [ ] v8::Global properly converted to v8::Local with scope
- [ ] JSON serialization implemented for script results
- [ ] All value types handled (string, number, boolean, object, null, undefined)
- [ ] Circular reference errors handled gracefully
- [ ] Appropriate error types added to DenoError enum
- [ ] TODO comment removed from line 394
- [ ] Code compiles without warnings
- [ ] Actual script results returned instead of fake message

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src

## RESEARCH NOTES
- v8::Global: Persistent handle that survives garbage collection
- v8::Local: Temporary handle tied to current scope
- v8::json::stringify: Built-in JSON serialization
- HandleScope: Required context for v8 operations
- Circular references: Cannot be JSON serialized, need fallback

## DOCUMENTATION LOCATIONS
- deno_core docs: https://docs.rs/deno_core/latest/deno_core/
- v8 crate docs: https://docs.rs/v8/latest/v8/
- Existing Deno integration: `packages/ecs-deno/src/`
- Error definitions: `packages/ecs-deno/src/error.rs` or `packages/ecs-deno/src/lib.rs`
