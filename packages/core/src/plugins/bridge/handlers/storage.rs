//! Storage handling functionality for the service bridge

use std::path::PathBuf;

use log::{debug, error};
use serde_json;

/// Handle storage read request
pub async fn handle_storage_read(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
    key: String,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from("./plugin_data").join(&key);

    match std::fs::read_to_string(&path) {
        Ok(content) => {
            debug!(
                "Storage read successful for plugin {} key {}",
                plugin_id, key
            );
            Ok(serde_json::Value::String(content))
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!("Storage key {} not found for plugin {}", key, plugin_id);
            Ok(serde_json::Value::Null)
        },
        Err(e) => {
            error!(
                "Storage read error for plugin {} key {}: {}",
                plugin_id, key, e
            );
            Err(format!("Storage read error for key '{key}': {e}"))
        },
    }
}

/// Handle storage write request
pub async fn handle_storage_write(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
    key: String,
    value: String,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from("./plugin_data").join(&key);

    if let Some(parent) = path.parent()
        && let Err(e) = std::fs::create_dir_all(parent)
    {
        error!(
            "Failed to create storage directory for plugin {}: {}",
            plugin_id, e
        );
        return Err(format!("Failed to create storage directory: {e}"));
    }

    match std::fs::write(&path, value) {
        Ok(_) => {
            debug!(
                "Storage write successful for plugin {} key {}",
                plugin_id, key
            );
            Ok(serde_json::Value::Null)
        },
        Err(e) => {
            error!(
                "Storage write error for plugin {} key {}: {}",
                plugin_id, key, e
            );
            Err(format!("Storage write error for key '{key}': {e}"))
        },
    }
}

/// Handle storage delete request
pub async fn handle_storage_delete(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
    key: String,
) -> Result<serde_json::Value, String> {
    let path = PathBuf::from("./plugin_data").join(&key);

    match std::fs::remove_file(&path) {
        Ok(_) => {
            debug!(
                "Storage delete successful for plugin {} key {}",
                plugin_id, key
            );
            Ok(serde_json::Value::Null)
        },
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            debug!(
                "Storage key {} not found for deletion for plugin {}",
                key, plugin_id
            );
            Ok(serde_json::Value::Null) // Treat as success
        },
        Err(e) => {
            error!(
                "Storage delete error for plugin {} key {}: {}",
                plugin_id, key, e
            );
            Err(format!("Storage delete error for key '{key}': {e}"))
        },
    }
}
