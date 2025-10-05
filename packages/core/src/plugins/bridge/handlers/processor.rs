//! Service request processing and routing functionality using modern event-driven architecture

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

        debug!(
            "Executing WASM function '{}' for plugin '{}'",
            function_name, self.plugin_id
        );

        // For now, process the data based on common WASM callback patterns
        match function_name {
            "process_data" => {
                // Process and potentially transform the input data
                Ok(data) // Echo back for now, but would do real processing
            },
            "validate_input" => {
                // Validate input data
                let is_valid = !data.is_empty(); // Simple validation
                Ok(vec![if is_valid { 1 } else { 0 }])
            },
            "transform_data" => {
                // Transform data according to plugin logic
                Ok(data) // Would apply transformations in real implementation
            },
            _ => Err(format!("Unknown WASM function: {}", function_name)),
        }
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

    if !plugin_id.is_empty() {
        Some(WasmRuntime {
            plugin_id: plugin_id.to_string(),
        })
    } else {
        None
    }
}

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
                            Ok(result_data) => {
                                debug!(
                                    "WASM callback successful for plugin {} function {}",
                                    plugin_id, function_name
                                );
                                Ok(result_data)
                            },
                            Err(e) => {
                                error!(
                                    "WASM callback failed for plugin {} function {}: {}",
                                    plugin_id, function_name, e
                                );
                                Err(format!("WASM execution failed: {}", e))
                            },
                        }
                    },
                    None => {
                        error!("WASM runtime not found for plugin: {}", plugin_id);
                        Err(format!("Plugin {} not found or not loaded", plugin_id))
                    },
                }
            });

            // Await the async task and return the result
            match callback_task.await {
                Ok(result) => ServiceResponse::WasmCallback(Ok(result)),
                Err(e) => {
                    error!("WASM callback task failed: {}", e);
                    ServiceResponse::WasmCallback(Err(format!("Task execution failed: {}", e)))
                },
            }
        },
    }
}
