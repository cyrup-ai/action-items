//! Clipboard handling functionality for the service bridge

use log::{debug, error};
use serde_json;

/// Handle clipboard read request
pub async fn handle_clipboard_read(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
) -> Result<String, String> {
    match action_items_ecs_clipboard::ArboardManager::get_text().await {
        Ok(text) => {
            debug!("Clipboard read successful for plugin {}", plugin_id);
            Ok(text)
        },
        Err(e) => {
            error!("Clipboard read error for plugin {}: {}", plugin_id, e);
            Err(format!("Clipboard read error: {e}"))
        },
    }
}

/// Handle clipboard write request
pub async fn handle_clipboard_write(
    plugin_id: String,
    _request_id: String,
    _callback_fn_name: String,
    text: String,
) -> Result<serde_json::Value, String> {
    match action_items_ecs_clipboard::ArboardManager::set_text(text).await {
        Ok(_) => {
            debug!("Clipboard write successful for plugin {}", plugin_id);
            Ok(serde_json::Value::Null)
        },
        Err(e) => {
            error!("Clipboard write error for plugin {}: {}", plugin_id, e);
            Err(format!("Clipboard write error: {e}"))
        },
    }
}
