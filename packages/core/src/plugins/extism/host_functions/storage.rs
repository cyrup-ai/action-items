use action_items_native::{StorageReadRequest, StorageWriteRequest};
use extism::{Function, UserData, Val, ValType};

use super::core::ExtismHostUserData;

/// Create async storage get host function using modern event-driven architecture
pub fn create_storage_get_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "storage_get_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'storage_get_async' expected I64 for key_ptr")
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_get_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_get_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let key_handle = plugin
                .memory_handle(key_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for key_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let key_str = plugin.memory_str(key_handle)?.to_string();
            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in storage_get_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in storage_get_async"))?;
            let host_data_mut = &mut *guard;

            let request = StorageReadRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
                key: key_str,
            };

            host_data_mut
                .storage_read_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send storage read request: {e}"))
                })?;

            Ok(())
        },
    )
}

/// Create async storage set host function using modern event-driven architecture
pub fn create_storage_set_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "storage_set_async",
        [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'storage_set_async' expected I64 for key_ptr")
            })? as u64;
            let value_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'storage_set_async' expected I64 for value_ptr")
            })? as u64;
            let request_id_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_set_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[3].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_set_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let key_handle = plugin
                .memory_handle(key_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for key_ptr"))?;
            let value_handle = plugin
                .memory_handle(value_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for value_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let key_str = plugin.memory_str(key_handle)?.to_string();
            let value_str = plugin.memory_str(value_handle)?.to_string();
            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in storage_set_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in storage_set_async"))?;
            let host_data_mut = &mut *guard;

            let request = StorageWriteRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
                key: key_str,
                value: value_str,
            };

            host_data_mut
                .storage_write_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send storage write request: {e}"))
                })?;

            Ok(())
        },
    )
}

/// Create async storage delete host function using modern event-driven architecture
pub fn create_storage_delete_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "storage_delete_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'storage_delete_async' expected I64 for key_ptr")
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_delete_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'storage_delete_async' expected I64 for callback_fn_ptr",
                )
            })? as u64;

            let key_handle = plugin
                .memory_handle(key_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for key_ptr"))?;
            let request_id_handle = plugin
                .memory_handle(request_id_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for request_id_ptr"))?;
            let callback_fn_handle = plugin
                .memory_handle(callback_fn_ptr)
                .ok_or_else(|| extism::Error::msg("Invalid memory handle for callback_fn_ptr"))?;

            let key_str = plugin.memory_str(key_handle)?.to_string();
            let request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in storage_delete_async"))?;
            let mut guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in storage_delete_async"))?;
            let host_data_mut = &mut *guard;

            // For delete operations, we use StorageWriteRequest with empty value to indicate
            // deletion
            let request = StorageWriteRequest {
                plugin_id: host_data_mut.plugin_id.clone(),
                request_id: request_id_str,
                key: key_str,
                value: String::new(), // Empty value indicates deletion
            };

            host_data_mut
                .storage_write_sender
                .send(request)
                .map_err(|e| {
                    extism::Error::msg(format!("Failed to send storage delete request: {e}"))
                })?;

            Ok(())
        },
    )
}
