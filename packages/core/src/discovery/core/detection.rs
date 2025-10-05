use std::path::Path;

use crate::discovery::loader::is_native_plugin_file;

/// Checks if a file is a plugin file
pub fn is_plugin_file(path: &Path) -> bool {
    if let Some(extension) = path.extension().and_then(|ext| ext.to_str()) {
        match extension {
            // WASM plugins
            "wasm" => return true,
            // Deno/JavaScript/TypeScript plugins
            "js" | "ts" | "mjs" => return true,
            _ => {},
        }
    }

    // Check for native plugins
    is_native_plugin_file(path)
}
