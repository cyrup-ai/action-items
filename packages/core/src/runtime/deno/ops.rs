//! Deno runtime operations
//!
//! Real Deno ops with actual system integration for notifications, clipboard, and HUD.

use std::sync::{Arc, OnceLock};

use action_items_common::plugin_interface::ActionItem;
use deno_core::op2;
use tokio::sync::{Mutex, mpsc};
use tracing::debug;

use crate::runtime::deno::notifications::{NotificationManager, NotificationOptions};
use crate::runtime::plugin_wrapper::request_handling::{
    ActionItemAction, ActionItemRequest, ActionItemResponse, ActionItemUpdates, RequestHandler,
    SearchMessage, SearchQuery, StorageMessage,
};

/// Global notification manager instance
static NOTIFICATION_MANAGER: OnceLock<NotificationManager> = OnceLock::new();

/// Global ActionItem request handler
static ACTION_ITEM_HANDLER: OnceLock<Arc<Mutex<RequestHandler>>> = OnceLock::new();

/// Initialize ActionItem handler (called once at startup)
pub fn initialize_action_item_handler(
    storage_tx: mpsc::Sender<StorageMessage>,
    search_tx: mpsc::Sender<SearchMessage>,
) -> Result<(), &'static str> {
    let handler = RequestHandler::new(storage_tx, search_tx);
    ACTION_ITEM_HANDLER
        .set(Arc::new(Mutex::new(handler)))
        .map_err(|_| "ActionItem handler already initialized")
}

/// Get ActionItem handler
async fn action_item_handler() -> Result<Arc<Mutex<RequestHandler>>, &'static str> {
    ACTION_ITEM_HANDLER
        .get()
        .ok_or("ActionItem handler not initialized").cloned()
}

/// Get notification manager with database integration
fn notification_manager() -> &'static NotificationManager {
    NOTIFICATION_MANAGER.get_or_init(|| {
        // Create notification manager without persistence for ops use
        match NotificationManager::new_without_persistence() {
            Ok(manager) => {
                tracing::info!("Notification manager initialized without persistence");
                manager
            },
            Err(e) => {
                tracing::error!("Notification manager initialization failed: {}", e);
                panic!("Failed to create notification manager: {}", e)
            },
        }
    })
}

/// Raycast API: Show toast notification
/// Real toast notification using UserNotifications framework
#[op2]
#[string]
pub fn op_show_toast(#[string] message: String) -> String {
    match notification_manager().show_toast(&message) {
        Ok(id) => format!("toast_shown_id_{}", id.as_u64()),
        Err(e) => {
            tracing::error!("Toast failed: {}", e);
            "toast_error".to_string()
        },
    }
}

/// Raycast API: Show HUD notification
/// Real HUD notification using UserNotifications framework
#[op2]
#[string]
pub fn op_show_hud(#[string] message: String) -> String {
    let options = NotificationOptions {
        title: "Action Items",
        message: &message,
        sound: false,
        urgent: true,
        ..Default::default()
    };

    match notification_manager().show_notification(options) {
        Ok(id) => format!("hud_shown_id_{}", id.as_u64()),
        Err(e) => {
            tracing::error!("HUD failed: {}", e);
            "hud_error".to_string()
        },
    }
}

/// Raycast API: Get clipboard contents
/// Real clipboard access with system integration
#[op2]
#[string]
pub fn op_get_clipboard() -> String {
    match action_items_ecs_clipboard::Clipboard::new() {
        Ok(mut clipboard) => match clipboard.get_text() {
            Ok(text) => {
                debug!(
                    source = "raycast",
                    "Clipboard text retrieved: {} chars",
                    text.len()
                );
                text
            },
            Err(e) => {
                debug!(source = "raycast", "Clipboard access failed: {}", e);
                String::new()
            },
        },
        Err(e) => {
            debug!(source = "raycast", "Failed to initialize clipboard: {}", e);
            String::new()
        },
    }
}

/// Log from JavaScript runtime
/// Real logging with structured output
#[op2(fast)]
pub fn op_log(#[string] message: String) {
    debug!(
        source = "plugin",
        message = %message,
        "JavaScript runtime log"
    );
}

/// ActionItem API: Create new ActionItem
/// Async operation using the working RequestHandler
#[op2(async)]
#[string]
pub async fn op_action_item_create(#[string] item_json: String) -> String {
    let item: ActionItem = match serde_json::from_str(&item_json) {
        Ok(item) => item,
        Err(e) => {
            return serde_json::json!({
                "success": false,
                "error": format!("Invalid item JSON: {}", e)
            })
            .to_string();
        },
    };

    let handler = action_item_handler().await;
    let request = ActionItemRequest {
        action: ActionItemAction::Create(item),
    };

    match handler {
        Ok(handler_arc) => match handler_arc
            .lock()
            .await
            .handle_action_item_request(request)
            .await
        {
            Ok(ActionItemResponse::Created(item)) => serde_json::json!({
                "success": true,
                "item": item
            })
            .to_string(),
            Ok(_) => serde_json::json!({
                "success": false,
                "error": "Unexpected response type"
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "success": false,
                "error": e
            })
            .to_string(),
        },
        Err(e) => serde_json::json!({
            "success": false,
            "error": format!("Handler not available: {}", e)
        })
        .to_string(),
    }
}

/// ActionItem API: Search ActionItems  
/// Async operation using the working RequestHandler
#[op2(async)]
#[string]
pub async fn op_action_item_search(#[string] query_json: String) -> String {
    let query: SearchQuery = match serde_json::from_str(&query_json) {
        Ok(query) => query,
        Err(e) => {
            return serde_json::json!({
                "success": false,
                "error": format!("Invalid query JSON: {}", e)
            })
            .to_string();
        },
    };

    let handler = action_item_handler().await;
    let request = ActionItemRequest {
        action: ActionItemAction::Search(query),
    };

    match handler {
        Ok(handler_arc) => match handler_arc
            .lock()
            .await
            .handle_action_item_request(request)
            .await
        {
            Ok(ActionItemResponse::SearchResults(results)) => serde_json::json!({
                "success": true,
                "results": results
            })
            .to_string(),
            Ok(_) => serde_json::json!({
                "success": false,
                "error": "Unexpected response type"
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "success": false,
                "error": e
            })
            .to_string(),
        },
        Err(e) => serde_json::json!({
            "success": false,
            "error": format!("Handler not available: {}", e)
        })
        .to_string(),
    }
}

/// ActionItem API: Update ActionItem
/// Async operation using the working RequestHandler  
#[op2(async)]
#[string]
pub async fn op_action_item_update(#[string] id: String, #[string] updates_json: String) -> String {
    let updates: ActionItemUpdates = match serde_json::from_str(&updates_json) {
        Ok(updates) => updates,
        Err(e) => {
            return serde_json::json!({
                "success": false,
                "error": format!("Invalid updates JSON: {}", e)
            })
            .to_string();
        },
    };

    let handler = action_item_handler().await;
    let request = ActionItemRequest {
        action: ActionItemAction::Update { id, updates },
    };

    match handler {
        Ok(handler_arc) => match handler_arc
            .lock()
            .await
            .handle_action_item_request(request)
            .await
        {
            Ok(ActionItemResponse::Updated(item)) => serde_json::json!({
                "success": true,
                "item": item
            })
            .to_string(),
            Ok(_) => serde_json::json!({
                "success": false,
                "error": "Unexpected response type"
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "success": false,
                "error": e
            })
            .to_string(),
        },
        Err(e) => serde_json::json!({
            "success": false,
            "error": format!("Handler not available: {}", e)
        })
        .to_string(),
    }
}

/// ActionItem API: Delete ActionItem
/// Async operation using the working RequestHandler
#[op2(async)]
#[string]
pub async fn op_action_item_delete(#[string] id: String) -> String {
    let handler = action_item_handler().await;
    let request = ActionItemRequest {
        action: ActionItemAction::Delete(id),
    };

    match handler {
        Ok(handler_arc) => match handler_arc
            .lock()
            .await
            .handle_action_item_request(request)
            .await
        {
            Ok(ActionItemResponse::Deleted { id }) => serde_json::json!({
                "success": true,
                "deleted_id": id
            })
            .to_string(),
            Ok(_) => serde_json::json!({
                "success": false,
                "error": "Unexpected response type"
            })
            .to_string(),
            Err(e) => serde_json::json!({
                "success": false,
                "error": e
            })
            .to_string(),
        },
        Err(e) => serde_json::json!({
            "success": false,
            "error": format!("Handler not available: {}", e)
        })
        .to_string(),
    }
}
