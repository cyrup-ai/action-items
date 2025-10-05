use std::future::Future;
use std::path::PathBuf;
use std::pin::Pin;

use serde_json::Value;

use crate::Error;

/// FFI-safe boxed future type
pub type BoxFuture<T> = Pin<Box<dyn Future<Output = Result<T, Error>> + Send>>;

/// Plugin execution context passed to plugins
#[derive(Debug, Clone)]
pub struct PluginContext {
    pub plugin_id: String,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
    pub matched_args: Vec<String>,
}

/// Main plugin trait - this is what plugins implement
pub trait LauncherPlugin: Send + Sync {
    /// Get plugin metadata
    fn manifest(&self) -> PluginManifest;

    /// Search for items
    fn search(&self, query: String, context: PluginContext) -> BoxFuture<Vec<ActionItem>>;

    /// Execute an action
    fn execute_action(
        &self,
        action_id: String,
        context: PluginContext,
        metadata: Option<Value>,
    ) -> BoxFuture<()>;

    /// Optional: background refresh
    fn background_refresh(&self, _context: PluginContext) -> BoxFuture<()> {
        Box::pin(async { Ok(()) })
    }
}

/// FFI-safe plugin manifest
#[derive(Debug, Clone)]
pub struct PluginManifest {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub keywords: Vec<String>,
    pub capabilities: PluginCapabilities,
}

#[derive(Debug, Clone)]
pub struct PluginCapabilities {
    pub search: bool,
    pub actions: bool,
    pub background_refresh: bool,
}

#[derive(Debug, Clone)]
pub struct ActionItem {
    pub id: String,
    pub title: String,
    pub subtitle: String,
    pub icon: Option<String>,
    pub score: f32,
    pub action_id: String,
    pub metadata: Option<Value>,
}

/// FFI-safe wrapper for a trait object
/// Trait objects in Rust are fat pointers (pointer + vtable), so we need to
/// preserve both parts when crossing FFI boundaries
#[repr(C)]
pub struct LauncherPluginFFI {
    data: *mut std::ffi::c_void,
    vtable: *mut std::ffi::c_void,
}

/// Helper functions for converting between Box<dyn LauncherPlugin> and FFI handle
pub mod ffi_helpers {
    use super::*;

    /// Convert a boxed plugin to an FFI-safe handle
    pub fn plugin_to_ffi(plugin: Box<dyn LauncherPlugin>) -> LauncherPluginFFI {
        let raw = Box::into_raw(plugin);
        // Trait object pointers are fat pointers containing (data_ptr, vtable_ptr)
        // We need to extract both parts
        unsafe {
            let (data, vtable): (*mut std::ffi::c_void, *mut std::ffi::c_void) =
                std::mem::transmute(raw);
            LauncherPluginFFI { data, vtable }
        }
    }

    /// Convert an FFI handle back to a boxed plugin
    ///
    /// # Safety
    /// The caller must ensure that:
    /// - The handle was created by `plugin_to_ffi`
    /// - The handle hasn't been freed already
    /// - The data and vtable pointers are valid
    pub unsafe fn ffi_to_plugin(handle: LauncherPluginFFI) -> Box<dyn LauncherPlugin> {
        // Reconstruct the fat pointer from its parts
        let fat_ptr: *mut dyn LauncherPlugin =
            unsafe { std::mem::transmute((handle.data, handle.vtable)) };
        unsafe { Box::from_raw(fat_ptr) }
    }
}

/// The single FFI export that plugins provide
pub type CreatePluginFn = extern "C" fn() -> LauncherPluginFFI;
