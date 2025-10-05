use action_items_native::NotificationRequest;
use extism::{Function, UserData, Val, ValType};

use super::core::ExtismHostUserData;

/// Create async notification show host function using modern event-driven architecture
pub fn create_notification_show_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "notification_show_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let notification_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'notification_show_async' expected I64 for notification_ptr",
                )
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'notification_show_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'notification_show_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let notification_handle = plugin
                .memory_handle(notification_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for notification_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let notification_str = plugin.memory_str(notification_handle)?.to_string();
            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data.get().map_err(|_| {
                extism::Error::msg("UserData has no data in notification_show_async")
            })?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in notification_show_async"))?;
            let host_data_mut = &mut *guard;

            // Parse notification JSON to extract title, body, and icon
            let notification_data: serde_json::Value = serde_json::from_str(&notification_str)
                .map_err(|e| extism::Error::msg(format!("Failed to parse notification: {e}")))?;

            let title = notification_data["title"]
                .as_str()
                .unwrap_or("Notification")
                .to_string();
            let body = notification_data["body"].as_str().unwrap_or("").to_string();
            let icon = notification_data["icon"].as_str().map(|s| s.to_string());

            let request = NotificationRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
                title,
                body,
                icon,
            };

            host_data_mut
                .notification_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send notification request: {e}"))
                })?;

            Ok(())
        },
    )
}
