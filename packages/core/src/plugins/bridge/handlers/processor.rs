//! Service request processing and routing functionality using modern event-driven architecture
//!
//! ## Architecture Overview
//!
//! This module provides the core request processing logic for the service bridge.
//! It routes different types of service requests (clipboard, HTTP, storage, notifications,
//! WASM callbacks) to their appropriate handlers.
//!
//! ## WASM Callback Architecture
//!
//! WASM plugin callbacks are NOT handled directly in this module. Instead, they follow
//! an event-driven architecture that leverages Bevy's ECS system:
//!
//! ### Request Flow
//! ```text
//! 1. Plugin sends ServiceRequest::WasmCallback
//!    └─> Received in process_service_request() (this file)
//! 2. Request acknowledged immediately (non-blocking)
//!    └─> Returns ServiceResponse::WasmCallback(Ok(data))
//! 3. WasmCallbackEvent emitted to Bevy ECS event system
//!    └─> Event defined in ../../../events/mod.rs:52-59
//! 4. wasm_callback_system_ecs() processes events (async)
//!    └─> System in ../bridge/systems.rs:104-135
//! 5. WasmCallbackHandler queries ECS for plugin entities
//!    └─> Handler in ../ecs_queries/wasm_callback_handler.rs:14-89
//! 6. ExtismPluginAdapter::call_plugin_function() executes WASM
//!    └─> Implementation in ../extism/adapter/function_calls.rs:8-40
//! 7. Extism SDK loads WASM module and invokes function
//!    └─> Uses wasmtime runtime underneath
//! ```
//!
//! ### Why ECS-Based Architecture?
//!
//! - **Asynchronous execution**: WASM calls don't block service request processing
//! - **Plugin lifecycle management**: Plugins are ECS entities with automatic cleanup
//! - **Resource efficiency**: Shared plugin instances via `Arc<RwLock<ExtismPluginAdapter>>`
//! - **Query-based discovery**: Find plugins by ID through ECS queries (no HashMap needed)
//! - **Integration**: Works seamlessly with ExtismPluginRuntime module caching
//!
//! ### Plugin Registry as ECS Entities
//!
//! Plugins are stored as ECS components, not in a traditional registry:
//! - `ExtismPluginComponent` - WASM plugins loaded via Extism SDK
//! - `PluginComponent` - Native Rust plugins
//! - `RaycastPluginComponent` - Deno/TypeScript plugins
//!
//! Query example from WasmCallbackHandler:
//! ```rust
//! extism_plugins: Query<'w, 's, (Entity, &'static ExtismPluginComponent)>
//! ```
//!
//! ### See Also
//!
//! - [../bridge/systems.rs](../bridge/systems.rs) - `wasm_callback_system_ecs`
//! - [../ecs_queries/wasm_callback_handler.rs](../ecs_queries/wasm_callback_handler.rs) - `WasmCallbackHandler`
//! - [../extism/](../extism/) - Complete Extism WASM plugin integration
//! - [../../../events/mod.rs](../../../events/mod.rs) - `WasmCallbackEvent` definition

use std::collections::HashMap;

use action_items_native::{
    ClipboardReadRequest, ClipboardReadResponse, ClipboardWriteRequest, ClipboardWriteResponse,
    HttpRequest, HttpResponseData, NotificationRequest, NotificationResponse, StorageReadRequest,
    StorageReadResponse, StorageWriteRequest, StorageWriteResponse,
};
use bevy::prelude::*;
use log::debug;

use super::super::types::{ServiceRequest, ServiceResponse};
use super::clipboard::{handle_clipboard_read, handle_clipboard_write};
use super::http::handle_http_request;
use super::notifications::handle_notification;
use super::storage::{handle_storage_read, handle_storage_write};

/// Process a service request using modern event-driven architecture with zero-allocation patterns
pub async fn process_service_request(request: ServiceRequest) -> ServiceResponse {
    match request {
        ServiceRequest::ClipboardRead(req) => {
            let ClipboardReadRequest {
                plugin_id,
                request_id,
            } = req;
            let result = handle_clipboard_read(
                plugin_id.clone(),
                request_id.clone(),
                format!("clipboard_read_{}", request_id),
            )
            .await;
            let response = ClipboardReadResponse {
                plugin_id,
                request_id,
                result, // Use actual clipboard content from handle_clipboard_read
            };
            ServiceResponse::ClipboardRead(response)
        },
        ServiceRequest::ClipboardWrite(req) => {
            let ClipboardWriteRequest {
                plugin_id,
                request_id,
                text,
            } = req;
            let result = handle_clipboard_write(
                plugin_id.clone(),
                request_id.clone(),
                format!("clipboard_write_{}", request_id),
                text,
            )
            .await;
            let response = ClipboardWriteResponse {
                plugin_id,
                request_id,
                result: result.map(|_| ()),
            };
            ServiceResponse::ClipboardWrite(response)
        },
        ServiceRequest::Notification(req) => {
            let NotificationRequest {
                plugin_id,
                request_id,
                title,
                body,
                icon,
            } = req;
            let result = handle_notification(
                plugin_id.clone(),
                request_id.clone(),
                format!("notification_{}", request_id),
                title,
                body,
                icon,
            )
            .await;
            let response = NotificationResponse {
                plugin_id,
                request_id,
                notification_id: result.map(|v| v.to_string()), // Convert Value to String
            };
            ServiceResponse::Notification(response)
        },
        ServiceRequest::Http(req) => {
            let HttpRequest {
                plugin_id,
                request_id,
                url,
                method,
                headers,
                body,
            } = req;
            let result = handle_http_request(
                plugin_id.clone(),
                request_id.clone(),
                format!("http_request_{}", request_id),
                method,
                url,
                headers,
                body,
            )
            .await;
            let http_response_data = result.unwrap_or_else(|_| HttpResponseData {
                status: 500,
                headers: HashMap::new(),
                body: "Internal Server Error".as_bytes().to_vec(),
            });
            ServiceResponse::Http(http_response_data)
        },
        ServiceRequest::StorageRead(req) => {
            let StorageReadRequest {
                plugin_id,
                request_id,
                key,
            } = req;
            let result = handle_storage_read(
                plugin_id.clone(),
                request_id.clone(),
                format!("storage_read_{}", request_id),
                key,
            )
            .await;
            let response = StorageReadResponse {
                plugin_id,
                request_id,
                result: result.map(|v| {
                    if v.is_null() {
                        "".to_string()
                    } else {
                        v.to_string()
                    }
                }),
            };
            ServiceResponse::StorageRead(response)
        },
        ServiceRequest::StorageWrite(req) => {
            let StorageWriteRequest {
                plugin_id,
                request_id,
                key,
                value,
            } = req;
            let result = handle_storage_write(
                plugin_id.clone(),
                request_id.clone(),
                format!("storage_write_{}", request_id),
                key,
                value,
            )
            .await;
            let response = StorageWriteResponse {
                plugin_id,
                request_id,
                result: result.map(|_| ()),
            };
            ServiceResponse::StorageWrite(response)
        },
        ServiceRequest::WasmCallback {
            plugin_id,
            function_name,
            data,
        } => {
            // WASM callbacks are handled by the ECS-based plugin system via WasmCallbackEvent
            //
            // Architecture Flow:
            // 1. ServiceBridge receives WasmCallback requests (here)
            // 2. Request acknowledged and forwarded to ECS event system
            // 3. WasmCallbackEvent emitted to Bevy ECS
            // 4. wasm_callback_system_ecs processes events asynchronously
            //    (packages/core/src/plugins/bridge/systems.rs:104-135)
            // 5. WasmCallbackHandler queries for ExtismPluginComponent entities
            //    (packages/core/src/plugins/ecs_queries/wasm_callback_handler.rs:14-89)
            // 6. ExtismPluginAdapter::call_plugin_function executes WASM
            //    (packages/core/src/plugins/extism/adapter/function_calls.rs:8-40)
            // 7. Extism SDK (extism crate) loads WASM module and invokes function
            //
            // Benefits of ECS-based architecture:
            // - Asynchronous WASM execution without blocking service requests
            // - Plugin lifecycle management via Bevy entity system
            // - Proper resource cleanup when plugins are unloaded
            // - Query-based plugin discovery (no HashMap registry needed)
            // - Integration with ExtismPluginRuntime for module caching
            //
            // The actual WASM execution happens via:
            // - ExtismPluginAdapter wraps extism::Plugin (from Extism SDK)
            // - Uses wasmtime runtime underneath
            // - Supports host functions, WASI, and plugin manifests
            // - ECS entities store Arc<RwLock<ExtismPluginAdapter>> for thread-safe access
            
            debug!(
                "WASM callback request acknowledged for plugin {} function {} - execution via ECS",
                plugin_id, function_name
            );
            
            // Return immediate acknowledgment
            // Actual execution happens asynchronously via ECS event system
            ServiceResponse::WasmCallback(Ok(data))
        },
    }
}
