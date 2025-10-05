use std::path::Path;
use std::sync::Arc;

use libloading::{Library, Symbol};

use crate::Error;
use crate::plugins::interface::ffi::{CreatePluginFn, LauncherPlugin, ffi_helpers};

/// Wrapper to keep the library alive while the plugin is in use
pub struct DynamicPlugin {
    _library: Arc<Library>,
    plugin: Box<dyn LauncherPlugin>,
}

impl DynamicPlugin {
    /// Load a plugin from a dynamic library file
    pub fn load_from_path(path: &Path) -> Result<Self, Error> {
        // Safety: We're loading a trusted plugin library
        let library = unsafe { Library::new(path) }.map_err(|e| {
            Error::PluginLoadError(format!("Failed to load library {}: {}", path.display(), e))
        })?;

        let library = Arc::new(library);

        // Get the plugin creation function
        let create_fn: Symbol<CreatePluginFn> =
            unsafe { library.get(b"_action_items_create_plugin") }.map_err(|e| {
                Error::PluginLoadError(format!(
                    "Plugin missing _action_items_create_plugin export: {e}"
                ))
            })?;

        // Create the plugin instance
        let plugin_ffi_handle = create_fn();

        // Safety: We created this plugin and trust it
        let plugin = unsafe { ffi_helpers::ffi_to_plugin(plugin_ffi_handle) };

        Ok(DynamicPlugin {
            _library: library,
            plugin,
        })
    }

    /// Get the inner plugin
    pub fn into_plugin(self) -> Box<dyn LauncherPlugin> {
        self.plugin
    }
}

/// Check if a file is a native plugin
pub fn is_native_plugin_file(path: &Path) -> bool {
    match path.extension().and_then(|ext| ext.to_str()) {
        Some("so") => true,    // Linux
        Some("dylib") => true, // macOS
        Some("dll") => true,   // Windows
        _ => false,
    }
}
