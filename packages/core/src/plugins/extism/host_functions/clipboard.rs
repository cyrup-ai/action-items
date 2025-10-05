use action_items_native::{ClipboardReadRequest, ClipboardWriteRequest};
use extism::{Function, UserData, Val, ValType};

use super::core::ExtismHostUserData;

/// Create async clipboard read host function using modern event-driven architecture
pub fn create_clipboard_read_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "clipboard_read_async",
        [ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let request_id_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'clipboard_read_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'clipboard_read_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in clipboard_read_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in clipboard_read_async"))?;
            let host_data_mut = &mut *guard;

            let request = ClipboardReadRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
            };

            host_data_mut
                .clipboard_read_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send clipboard read request: {e}"))
                })?;

            Ok(())
        },
    )
}

/// Create async clipboard write host function using modern event-driven architecture
pub fn create_clipboard_write_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "clipboard_write_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let text_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'clipboard_write_async' expected I64 for text_ptr",
                )
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'clipboard_write_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'clipboard_write_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let text_handle = plugin
                .memory_handle(text_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for text_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let text_str = plugin.memory_str(text_handle)?.to_string();
            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in clipboard_write_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in clipboard_write_async"))?;
            let host_data_mut = &mut *guard;

            let request = ClipboardWriteRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
                text: text_str,
            };

            host_data_mut
                .clipboard_write_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send clipboard write request: {e}"))
                })?;

            Ok(())
        },
    )
}
