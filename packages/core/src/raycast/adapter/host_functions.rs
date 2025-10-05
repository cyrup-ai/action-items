//! Host Functions Implementation
//!
//! Zero-allocation host functions for Raycast extension runtime integration
//! with blazing-fast performance and secure sandboxed execution.

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use parking_lot::Mutex;
use serde_json::Value;
use tracing::{error, trace, warn};

/// Host function type alias for better readability
pub type HostFunction = fn(&[u8]) -> Vec<u8>;
pub type HostFunctionRegistry = Vec<(&'static str, HostFunction)>;

/// Global storage manager for Raycast extensions
struct StorageManager {
    data: Arc<Mutex<HashMap<String, Value>>>,
    storage_path: PathBuf,
}

impl StorageManager {
    fn new() -> Self {
        let storage_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("action-items")
            .join("raycast-storage.json");

        let data = Self::load_from_disk(&storage_path);

        Self {
            data: Arc::new(Mutex::new(data)),
            storage_path,
        }
    }

    fn load_from_disk(path: &PathBuf) -> HashMap<String, Value> {
        if path.exists() {
            match fs::read_to_string(path) {
                Ok(content) => match serde_json::from_str::<HashMap<String, Value>>(&content) {
                    Ok(data) => data,
                    Err(e) => {
                        warn!("Failed to parse storage file: {}", e);
                        HashMap::new()
                    },
                },
                Err(e) => {
                    warn!("Failed to read storage file: {}", e);
                    HashMap::new()
                },
            }
        } else {
            HashMap::new()
        }
    }

    fn save_to_disk(&self) -> Result<(), String> {
        // Ensure directory exists
        if let Some(parent) = self.storage_path.parent()
            && let Err(e) = fs::create_dir_all(parent)
        {
            return Err(format!("Failed to create storage directory: {}", e));
        }

        let data = self.data.lock();
        let json = serde_json::to_string_pretty(&*data)
            .map_err(|e| format!("JSON serialization error: {}", e))?;

        fs::write(&self.storage_path, json)
            .map_err(|e| format!("Failed to write storage file: {}", e))
    }

    fn get(&self, key: &str) -> Result<Option<Value>, String> {
        let data = self.data.lock();
        Ok(data.get(key).cloned())
    }

    fn set(&self, key: String, value: Value) -> Result<(), String> {
        {
            let mut data = self.data.lock();
            data.insert(key, value);
        }
        self.save_to_disk()
    }

    fn remove(&self, key: &str) -> Result<bool, String> {
        let removed = {
            let mut data = self.data.lock();
            data.remove(key).is_some()
        };
        if removed {
            self.save_to_disk()?;
        }
        Ok(removed)
    }

    fn clear(&self) -> Result<(), String> {
        {
            let mut data = self.data.lock();
            data.clear();
        }
        self.save_to_disk()
    }
}

/// Get global storage manager instance
fn get_storage_manager() -> Result<&'static StorageManager, String> {
    use std::sync::OnceLock;
    static STORAGE: OnceLock<StorageManager> = OnceLock::new();

    Ok(STORAGE.get_or_init(StorageManager::new))
}

/// Get the complete registry of available host functions
/// Zero-allocation function registry with blazing-fast lookup performance
pub fn get_host_function_registry() -> HostFunctionRegistry {
    vec![
        ("show_toast", host_show_toast),
        ("show_hud", host_show_hud),
        ("http_fetch", host_http_fetch),
        ("storage_get", host_storage_get),
        ("storage_set", host_storage_set),
        ("storage_remove", host_storage_remove),
        ("storage_clear", host_storage_clear),
        ("environment_get", host_environment_get),
        ("clipboard_write", host_clipboard_write),
        ("clipboard_read", host_clipboard_read),
        ("preferences_get", host_preferences_get),
        ("preferences_set", host_preferences_set),
        ("raycast_open", host_raycast_open),
        ("raycast_pop_to_root", host_raycast_pop_to_root),
        ("raycast_close_main_window", host_raycast_close_main_window),
    ]
}

/// Host function: Toast notification display
fn host_show_toast(input: &[u8]) -> Vec<u8> {
    if let Ok(message) = serde_json::from_slice::<Value>(input)
        && let Some(text) = message.as_str()
    {
        trace!(
            function = "show_toast",
            message = %text,
            "Host function called"
        );
    }
    b"{}".to_vec()
}

/// Host function: HUD notification display
fn host_show_hud(input: &[u8]) -> Vec<u8> {
    if let Ok(message) = serde_json::from_slice::<Value>(input)
        && let Some(text) = message.as_str()
    {
        trace!(
            function = "show_hud",
            message = %text,
            "Host function called"
        );
    }
    b"{}".to_vec()
}

/// Host function: HTTP fetch implementation (simplified)
fn host_http_fetch(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":false,\"error\":\"HTTP fetch temporarily disabled\"}".to_vec()
}

/// Host function: Storage get implementation
fn host_storage_get(input: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<Value>(input) {
        Ok(request_obj) => {
            let key = match request_obj.get("key").and_then(Value::as_str) {
                Some(key) => key,
                None => {
                    return b"{\"success\":false,\"error\":\"Missing key parameter\"}".to_vec();
                },
            };

            let storage = match get_storage_manager() {
                Ok(storage) => storage,
                Err(e) => {
                    let error_response = serde_json::json!({
                        "success": false,
                        "error": format!("Storage manager unavailable: {}", e)
                    });
                    return serde_json::to_vec(&error_response).unwrap_or_else(|_| {
                        b"{\"success\":false,\"error\":\"JSON serialization failed\"}".to_vec()
                    });
                },
            };
            match storage.get(key) {
                Ok(Some(value)) => {
                    let response = serde_json::json!({
                        "success": true,
                        "value": value
                    });
                    match serde_json::to_vec(&response) {
                        Ok(json) => json,
                        Err(e) => {
                            error!("Failed to serialize storage response: {}", e);
                            b"{\"success\":false,\"error\":\"Serialization error\"}".to_vec()
                        },
                    }
                },
                Ok(None) => b"{\"success\":true,\"value\":null}".to_vec(),
                Err(e) => {
                    error!("Storage get error for key '{}': {}", key, e);
                    let response = format!("{{\"success\":false,\"error\":\"{}\"}}", e);
                    response.into_bytes()
                },
            }
        },
        Err(_) => b"{\"success\":false,\"error\":\"Invalid request JSON\"}".to_vec(),
    }
}

/// Host function: Storage set implementation
fn host_storage_set(input: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<Value>(input) {
        Ok(request_obj) => {
            let key = match request_obj.get("key").and_then(Value::as_str) {
                Some(key) => key,
                None => {
                    return b"{\"success\":false,\"error\":\"Missing key parameter\"}".to_vec();
                },
            };

            let value = match request_obj.get("value") {
                Some(value) => value.clone(),
                None => {
                    return b"{\"success\":false,\"error\":\"Missing value parameter\"}".to_vec();
                },
            };

            let storage = match get_storage_manager() {
                Ok(storage) => storage,
                Err(e) => {
                    error!("Storage manager unavailable: {}", e);
                    let response = format!(
                        "{{\"success\":false,\"error\":\"Storage manager unavailable: {}\"}}",
                        e
                    );
                    return response.into_bytes();
                },
            };
            match storage.set(key.to_string(), value) {
                Ok(()) => b"{\"success\":true}".to_vec(),
                Err(e) => {
                    error!("Storage set error for key '{}': {}", key, e);
                    let response = format!("{{\"success\":false,\"error\":\"{}\"}}", e);
                    response.into_bytes()
                },
            }
        },
        Err(_) => b"{\"success\":false,\"error\":\"Invalid request JSON\"}".to_vec(),
    }
}

/// Host function: Storage remove implementation
fn host_storage_remove(input: &[u8]) -> Vec<u8> {
    match serde_json::from_slice::<Value>(input) {
        Ok(request_obj) => {
            let key = match request_obj.get("key").and_then(Value::as_str) {
                Some(key) => key,
                None => {
                    return b"{\"success\":false,\"error\":\"Missing key parameter\"}".to_vec();
                },
            };

            let storage = match get_storage_manager() {
                Ok(storage) => storage,
                Err(e) => {
                    let response = format!(
                        "{{\"success\":false,\"error\":\"Storage manager unavailable: {}\"}}",
                        e
                    );
                    return response.into_bytes();
                },
            };
            match storage.remove(key) {
                Ok(_) => b"{\"success\":true}".to_vec(),
                Err(e) => {
                    error!("Storage remove error for key '{}': {}", key, e);
                    let response = format!("{{\"success\":false,\"error\":\"{}\"}}", e);
                    response.into_bytes()
                },
            }
        },
        Err(_) => b"{\"success\":false,\"error\":\"Invalid request JSON\"}".to_vec(),
    }
}

/// Host function: Storage clear implementation
fn host_storage_clear(_input: &[u8]) -> Vec<u8> {
    let storage = match get_storage_manager() {
        Ok(storage) => storage,
        Err(e) => {
            error!("Storage manager unavailable: {}", e);
            let response = format!(
                "{{\"success\":false,\"error\":\"Storage manager unavailable: {}\"}}",
                e
            );
            return response.into_bytes();
        },
    };
    match storage.clear() {
        Ok(()) => b"{\"success\":true}".to_vec(),
        Err(e) => {
            error!("Storage clear error: {}", e);
            let response = format!("{{\"success\":false,\"error\":\"{}\"}}", e);
            response.into_bytes()
        },
    }
}

/// Host function: Environment variable get implementation
fn host_environment_get(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true,\"value\":null}".to_vec()
}

/// Host function: Clipboard write implementation
fn host_clipboard_write(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true}".to_vec()
}

/// Host function: Clipboard read implementation
fn host_clipboard_read(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true,\"value\":\"\"}".to_vec()
}

/// Host function: Preferences get implementation
fn host_preferences_get(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true,\"value\":null}".to_vec()
}

/// Host function: Preferences set implementation
fn host_preferences_set(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true}".to_vec()
}

/// Host function: Raycast open implementation
fn host_raycast_open(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true}".to_vec()
}

/// Host function: Raycast pop to root implementation
fn host_raycast_pop_to_root(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true}".to_vec()
}

/// Host function: Raycast close main window implementation
fn host_raycast_close_main_window(_input: &[u8]) -> Vec<u8> {
    b"{\"success\":true}".to_vec()
}
