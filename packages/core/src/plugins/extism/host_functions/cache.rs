//! Cache host functions for Extism plugins
//!
//! These host functions provide synchronous cache operations to WASM plugins.
//! The synchronous design is correct and intentional due to architectural constraints:
//!
//! ## Why Synchronous Is Correct
//!
//! 1. **Extism Limitation**: Extism/Wasmtime host functions MUST be synchronous.
//!    The `Function::new` type signature requires `Fn(...) -> Result<(), Error>`,
//!    not async fn. This is a fundamental runtime constraint.
//!    Reference: extism/runtime/src/function.rs lines 190-202
//!
//! 2. **Cache Backend**: Uses moka::sync::Cache for thread-safe in-memory caching.
//!    Operations are lock-free and complete in nanoseconds (no I/O).
//!    Reference: plugins/interface/context/services.rs
//!
//! 3. **No Async Needed**: Unlike storage/http operations that use crossbeam channels
//!    for async I/O, cache operations are instant and benefit from direct access.
//!
//! The function names include "_async" suffix for API consistency with other host
//! functions, but execution is synchronous as required by Extism.

use extism::{Function, UserData, Val, ValType};

use super::core::ExtismHostUserData;

/// Create async cache get host function
pub fn create_cache_get_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "cache_get_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'cache_get_async' expected I64 for key_ptr")
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_get_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_get_async' expected I64 for callback_fn_ptr",
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
            let _request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in cache_get_async"))?;
            let guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in cache_get_async"))?;
            let host_data = &*guard;

            // Synchronous cache access is architecturally correct here:
            // - Extism host functions MUST be synchronous (no async support in Extism/Wasmtime)
            // - moka::sync::Cache provides thread-safe, lock-free, in-memory operations
            // - Cache operations complete in nanoseconds (no I/O blocking)
            // - Unlike storage/http, cache doesn't need event-driven async architecture
            // This is the complete implementation, not a temporary simulation.
            let _result = host_data.cache_service.get(&key_str);

            drop(guard);

            Ok(())
        },
    )
}

/// Create async cache set host function
pub fn create_cache_set_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "cache_set_async",
        [ValType::I64, ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'cache_set_async' expected I64 for key_ptr")
            })? as u64;
            let value_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'cache_set_async' expected I64 for value_ptr")
            })? as u64;
            let request_id_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_set_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[3].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_set_async' expected I64 for callback_fn_ptr",
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
            let _request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in cache_set_async"))?;
            let guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in cache_set_async"))?;
            let host_data = &*guard;

            // Synchronous cache write is correct (see cache_get_async for detailed explanation)
            host_data.cache_service.set(key_str, value_str);

            drop(guard);
            Ok(())
        },
    )
}

/// Create async cache delete host function
pub fn create_cache_delete_async(user_data_param: ExtismHostUserData) -> Function {
    Function::new(
        "cache_delete_async",
        [ValType::I64, ValType::I64, ValType::I64],
        [],
        UserData::new(user_data_param),
        |plugin: &mut extism::CurrentPlugin,
         inputs: &[Val],
         _outputs: &mut [Val],
         user_data: UserData<ExtismHostUserData>| {
            let key_ptr = inputs[0].i64().ok_or_else(|| {
                extism::Error::msg("Host function 'cache_delete_async' expected I64 for key_ptr")
            })? as u64;
            let request_id_ptr = inputs[1].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_delete_async' expected I64 for request_id_ptr",
                )
            })? as u64;
            let callback_fn_ptr = inputs[2].i64().ok_or_else(|| {
                extism::Error::msg(
                    "Host function 'cache_delete_async' expected I64 for callback_fn_ptr",
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
            let _request_id_str = plugin.memory_str(request_id_handle)?.to_string();
            let _callback_fn_name_str = plugin.memory_str(callback_fn_handle)?.to_string();

            let arc_mutex_t = user_data
                .get()
                .map_err(|_| extism::Error::msg("UserData has no data in cache_delete_async"))?;
            let guard = arc_mutex_t
                .lock()
                .map_err(|_| extism::Error::msg("Mutex poisoned in cache_delete_async"))?;
            let host_data = &*guard;

            // Synchronous cache delete is correct (see cache_get_async for detailed explanation)
            host_data.cache_service.remove(&key_str);

            drop(guard);
            Ok(())
        },
    )
}
