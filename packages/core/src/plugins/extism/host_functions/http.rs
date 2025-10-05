use extism::{Function, UserData, Val, ValType};

use super::core::ExtismHostUserData;
// Note: HttpRequest will be used when HTTP host functions are implemented

/// Create async HTTP request host function
pub fn create_http_request_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "http_request_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let request_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'http_request_async' expected I64 for request_ptr",
                )
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'http_request_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'http_request_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let request_handle = plugin
                .memory_handle(request_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let request_str = plugin.memory_str(request_handle)?.to_string();
            let _request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in http_request_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in http_request_async"))?;
            let host_data_mut = &mut *guard;

            host_data_mut
                .http_sender
                .send(serde_json::from_str(&request_str).map_err(|e| {
                    extism::Error::msg(format!("Failed to parse HTTP request: {e}"))
                })?)
                .map_err(|e| extism::Error::msg(format!("Failed to send HTTP command: {e}")))?;

            Ok(())
        },
    )
}
