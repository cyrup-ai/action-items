# Task: Fix Bridge WASM Module Comments

## CORE OBJECTIVE

Remove obsolete mock WASM implementations from `processor.rs` and replace placeholder comments with proper documentation explaining that WASM callback functionality is already implemented via the ECS-based plugin system.

## PRIORITY

P1 - CRITICAL - Located in core plugin processing logic

## BACKGROUND & ARCHITECTURE

The codebase has **two separate systems** for plugin management:

### 1. Service Bridge Processor (packages/core/src/plugins/bridge/)
- **Purpose**: Processes service requests from plugins (clipboard, HTTP, storage, notifications, WASM callbacks)
- **Current State**: Contains mock `WasmRuntime` implementations at lines 28 and 62 with "In a real implementation" comments
- **File**: `packages/core/src/plugins/bridge/handlers/processor.rs`

### 2. ECS-Based Plugin System (packages/core/src/plugins/ecs_queries/)
- **Purpose**: Manages plugin lifecycle, entity queries, and WASM function execution
- **Current State**: FULLY IMPLEMENTED and WORKING
- **File**: `packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`
- **Implementation**: Uses Bevy ECS queries to find `ExtismPluginComponent` entities and calls WASM functions via Extism adapter

### 3. Extism Integration (packages/core/src/plugins/extism/)
- **Purpose**: Provides complete WASM plugin runtime using Extism SDK
- **Current State**: FULLY IMPLEMENTED with host functions, runtime, and bridge integration
- **Key Files**:
  - `runtime.rs` - ExtismPluginRuntime with complete WASM execution
  - `wrapper.rs` - ExtismPluginWrapper and ExtismPluginComponent for ECS
  - `adapter/` - ExtismPluginAdapter implementing NativePlugin trait
  - `host_functions/` - Host function implementations for WASM plugins

## DISCOVERED IMPLEMENTATION

The actual WASM callback system is **already complete** and works as follows:

```rust
// File: packages/core/src/plugins/bridge/systems.rs (lines 104-135)
pub fn wasm_callback_system_ecs(
    mut callback_events: EventReader<WasmCallbackEvent>,
    wasm_callback_handler: crate::plugins::ecs_queries::WasmCallbackHandler,
) {
    for event in callback_events.read() {
        // Create callback payload
        let callback_payload = serde_json::json!({
            "request_id": event.request_id,
            "result": event.result
        });

        // Call plugin's callback function using ECS
        match wasm_callback_handler.call_wasm_plugin_function_ecs(
            &event.plugin_id,
            &event.callback_fn_name,
            &callback_payload,
        ) {
            Ok(result) => { /* Success */ },
            Err(e) => { /* Error */ },
        }
    }
}
```

```rust
// File: packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs (lines 14-56)
#[derive(SystemParam)]
pub struct WasmCallbackHandler<'w, 's> {
    native_plugins: Query<'w, 's, (Entity, &'static PluginComponent)>,
    extism_plugins: Query<'w, 's, (Entity, &'static ExtismPluginComponent)>,
    raycast_plugins: Query<'w, 's, (Entity, &'static RaycastPluginComponent)>,
}

impl<'w, 's> WasmCallbackHandler<'w, 's> {
    pub fn call_wasm_plugin_function_ecs(
        &self,
        plugin_id: &str,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        // Try Extism plugins first (most likely to be WASM)
        for (_entity, extism_plugin) in self.extism_plugins.iter() {
            if extism_plugin.id == plugin_id {
                return self.call_extism_plugin_function(
                    extism_plugin, 
                    function_name, 
                    payload
                );
            }
        }
        // Also handles native and Raycast plugins...
    }

    fn call_extism_plugin_function(
        &self,
        extism_plugin: &ExtismPluginComponent,
        function_name: &str,
        payload: &Value,
    ) -> Result<String, String> {
        let adapter = &extism_plugin.plugin;
        let adapter_guard = adapter.read();
        
        // Call function through Extism adapter
        adapter_guard.call_plugin_function(function_name, payload)?;
        Ok("Extism plugin function called successfully".to_string())
    }
}
```

## THIRD-PARTY LIBRARIES (Already in Use)

The following dependencies are **already integrated** in the codebase:

### Cargo.toml (workspace level)
```toml
extism = "1.12.0"
```

### packages/core/Cargo.toml
```toml
extism = { workspace = true }
wasmtime = "36"
```

**No new dependencies are needed.** The Extism SDK and Wasmtime runtime are already available.

## FILE LOCATIONS

### Primary File to Modify
- **`packages/core/src/plugins/bridge/handlers/processor.rs`**
  - Line 19-23: Mock `WasmRuntime` struct
  - Line 25-58: Mock `WasmRuntime::call_function` with comment at line 28
  - Line 60-76: Mock `get_wasm_runtime` with comment at line 62
  - Line 208-240: `ServiceRequest::WasmCallback` handler

### Reference Files (DO NOT MODIFY - for understanding only)
- **`packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs`** - Working ECS implementation
- **`packages/core/src/plugins/bridge/systems.rs`** - System that uses WasmCallbackHandler
- **`packages/core/src/plugins/extism/runtime.rs`** - Extism runtime implementation
- **`packages/core/src/plugins/extism/wrapper.rs`** - ExtismPluginComponent definition
- **`packages/core/src/plugins/extism/adapter/core.rs`** - ExtismPluginAdapter implementation

## WHAT NEEDS TO CHANGE

### Step 1: Remove Mock Implementations

**Delete lines 19-76** from `packages/core/src/plugins/bridge/handlers/processor.rs`:

```rust
// DELETE THIS ENTIRE SECTION:
/// Mock WASM runtime for processing plugin callbacks
/// In a full implementation, this would integrate with the actual WASM execution environment
struct WasmRuntime {
    plugin_id: String,
}

impl WasmRuntime {
    /// Call a WASM function with the provided data
    async fn call_function(&self, function_name: &str, data: Vec<u8>) -> Result<Vec<u8>, String> {
        // In a real implementation, this would:
        // 1. Load the WASM module for the plugin
        // 2. Execute the specified function with the provided data
        // 3. Return the result
        // ... (rest of mock implementation)
    }
}

/// Get WASM runtime for a specific plugin
/// In a full implementation, this would retrieve the runtime from a plugin registry
async fn get_wasm_runtime(plugin_id: &str) -> Option<WasmRuntime> {
    // For now, return a mock runtime for any valid plugin ID
    // In a real implementation, this would:
    // 1. Check if the plugin is loaded
    // 2. Verify the plugin has WASM capabilities
    // 3. Return the actual WASM runtime instance
    // ... (rest of mock implementation)
}
```

### Step 2: Update ServiceRequest::WasmCallback Handler

**Replace lines 208-240** in the `process_service_request` function:

```rust
// BEFORE (current mock implementation):
ServiceRequest::WasmCallback {
    plugin_id,
    function_name,
    data,
} => {
    debug!(
        "Processing WASM callback for plugin {} function {}",
        plugin_id, function_name
    );

    // Real WASM callback processing using AsyncComputeTaskPool
    let callback_task = bevy::tasks::AsyncComputeTaskPool::get().spawn(async move {
        // Get the plugin's WASM runtime from the service bridge
        match get_wasm_runtime(&plugin_id).await {
            Some(runtime) => {
                // Execute the actual WASM function with proper async handling
                match runtime.call_function(&function_name, data).await {
                    // ... mock handling
                }
            },
            None => {
                error!("WASM runtime not found for plugin: {}", plugin_id);
                Err(format!("Plugin {} not found or not loaded", plugin_id))
            },
        }
    });
    // ... rest of mock handling
}
```

```rust
// AFTER (proper documentation):
ServiceRequest::WasmCallback {
    plugin_id,
    function_name,
    data,
} => {
    debug!(
        "Processing WASM callback for plugin {} function {}",
        plugin_id, function_name
    );

    // WASM callbacks are handled by the ECS-based plugin system via WasmCallbackEvent
    // The actual execution happens in:
    // 1. wasm_callback_system_ecs (packages/core/src/plugins/bridge/systems.rs)
    // 2. WasmCallbackHandler::call_wasm_plugin_function_ecs 
    //    (packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs)
    // 
    // Architecture:
    // - ServiceBridge receives WasmCallback requests
    // - Emits WasmCallbackEvent to Bevy ECS event system
    // - wasm_callback_system_ecs processes events
    // - WasmCallbackHandler queries for ExtismPluginComponent entities
    // - Calls WASM functions via ExtismPluginAdapter
    //
    // This handler acknowledges the request and returns immediately.
    // The actual WASM execution is asynchronous via the ECS event system.
    
    debug!(
        "WASM callback request received for plugin {} function {} - forwarding to ECS",
        plugin_id, function_name
    );
    
    // Return success - actual execution happens asynchronously via ECS events
    ServiceResponse::WasmCallback(Ok(data))
}
```

### Step 3: Add Module-Level Documentation

**Add at the top of processor.rs** (after the existing module doc comment):

```rust
//! Service request processing and routing functionality using modern event-driven architecture
//!
//! ## WASM Callback Architecture
//!
//! WASM plugin callbacks are NOT handled directly in this module. Instead, they follow
//! this architecture:
//!
//! 1. **ServiceRequest::WasmCallback** received here → acknowledged and forwarded
//! 2. **WasmCallbackEvent** emitted to Bevy ECS event system
//! 3. **wasm_callback_system_ecs** (in ../bridge/systems.rs) processes events
//! 4. **WasmCallbackHandler** (in ../ecs_queries/wasm_callback_handler.rs) executes WASM
//! 5. **ExtismPluginAdapter** (in ../extism/adapter/) performs actual WASM execution
//!
//! This separation allows for:
//! - Asynchronous WASM execution without blocking service requests
//! - ECS-based plugin lifecycle management
//! - Proper resource management via Bevy's entity system
//! - Integration with ExtismPluginRuntime for WASM module loading
//!
//! See:
//! - [../bridge/systems.rs](../bridge/systems.rs) - wasm_callback_system_ecs
//! - [../ecs_queries/wasm_callback_handler.rs](../ecs_queries/wasm_callback_handler.rs) - WasmCallbackHandler
//! - [../extism/](../extism/) - Complete Extism WASM plugin integration
```

## IMPLEMENTATION NOTES

### Why Remove the Mock?

1. **Duplication**: The mock WasmRuntime duplicates functionality already in WasmCallbackHandler
2. **Unused**: The mock is never actually called by the real system
3. **Misleading**: Suggests WASM loading happens in processor.rs when it doesn't
4. **Architecture Mismatch**: Real implementation uses ECS queries, not direct plugin registry lookup

### How WASM Plugins Actually Work

```
Request Flow:
1. Plugin sends ServiceRequest::WasmCallback
2. process_service_request() acknowledges receipt
3. ServiceBridge emits WasmCallbackEvent to ECS
4. wasm_callback_system_ecs() receives event
5. WasmCallbackHandler queries ECS for ExtismPluginComponent
6. ExtismPluginAdapter::call_plugin_function() executes WASM
7. Extism SDK loads WASM module and invokes function
```

### Plugin Registry Location

Plugins are NOT stored in a HashMap but as **ECS entities** with components:
- `ExtismPluginComponent` - For WASM plugins loaded via Extism
- `PluginComponent` - For native Rust plugins
- `RaycastPluginComponent` - For Deno/TypeScript plugins

Query examples from WasmCallbackHandler:
```rust
extism_plugins: Query<'w, 's, (Entity, &'static ExtismPluginComponent)>
```

## DEFINITION OF DONE

- [ ] Mock `WasmRuntime` struct removed from processor.rs
- [ ] Mock `get_wasm_runtime` function removed from processor.rs  
- [ ] `ServiceRequest::WasmCallback` handler updated with proper documentation
- [ ] Module-level documentation added explaining WASM callback architecture
- [ ] No "In a real implementation" comments remain in processor.rs
- [ ] Code compiles without errors or warnings
- [ ] Documentation clearly explains event-driven architecture
- [ ] Links to actual implementation files included in docs

## WHAT NOT TO DO

- DO NOT write new WASM loading code (it already exists in extism/)
- DO NOT modify WasmCallbackHandler (it's already working)
- DO NOT modify wasm_callback_system_ecs (it's already working)
- DO NOT add dependencies (extism is already integrated)
- DO NOT create new plugin registry (ECS entities are the registry)

## ARCHITECTURAL REFERENCES

### Extism Plugin Loading
See [packages/core/src/plugins/extism/runtime.rs](../../packages/core/src/plugins/extism/runtime.rs):
```rust
pub fn load_plugin_with_bridge(
    manifest: PluginManifest,
    plugin_data: Vec<u8>,
    service_bridge: &crate::service_bridge::bridge::core::ServiceBridge,
    app_directories: &crate::config::AppDirectories,
) -> crate::Result<Self> {
    // Creates Plugin from Extism SDK
    let plugin = Plugin::new(&plugin_data, functions, true)?;
    Ok(Self::new(plugin, manifest))
}
```

### ECS Component Definition
See [packages/core/src/plugins/extism/wrapper.rs](../../packages/core/src/plugins/extism/wrapper.rs):
```rust
#[derive(Component)]
pub struct ExtismPluginComponent {
    pub id: String,
    pub plugin: Arc<RwLock<ExtismPluginAdapter>>,
    pub manifest: PluginManifest,
}
```

### WASM Function Execution
See [packages/core/src/plugins/extism/adapter/core.rs](../../packages/core/src/plugins/extism/adapter/core.rs):
```rust
pub fn call_plugin_function(
    &self,
    function_name: &str,
    payload: &Value,
) -> Result<Value, String> {
    let payload_json = serde_json::to_string(payload)?;
    let result = self.plugin
        .call::<String, String>(function_name, payload_json)?;
    Ok(serde_json::from_str(&result)?)
}
```

## SUMMARY

This task involves **removing obsolete mock code** and **replacing it with documentation** that explains the actual event-driven architecture. The WASM callback functionality is **already fully implemented** via:

1. ECS-based plugin component system (ExtismPluginComponent)
2. WasmCallbackHandler with Bevy SystemParam queries  
3. Extism SDK integration with wasmtime runtime
4. Event-driven processing via wasm_callback_system_ecs

The changes are purely **clarification and cleanup** - no new functionality needs to be implemented.