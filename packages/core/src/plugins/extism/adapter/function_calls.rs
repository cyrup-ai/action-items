use log::debug;

use super::core::ExtismPluginAdapter;

impl ExtismPluginAdapter {
    /// Call a function in the plugin (used by callback system)
    #[allow(dead_code)]
    pub fn call_plugin_function(
        &self,
        function_name: &str,
        payload: &serde_json::Value,
    ) -> crate::Result<()> {
        let mut plugin = self.plugin.lock();

        if !plugin.function_exists(function_name) {
            return Err(crate::Error::PluginError(format!(
                "Plugin {} does not have function {}",
                self.manifest.id, function_name
            )));
        }

        let payload_json = serde_json::to_string(payload)?;
        match plugin.call::<String, ()>(function_name, payload_json) {
            Ok(_) => {},
            Err(e) => return Err(crate::Error::Extism(e.to_string())),
        };

        debug!(
            "Successfully called function {} on plugin {} with payload",
            function_name, self.manifest.id
        );
        Ok(())
    }

    /// Check if a function exists in the plugin
    pub fn function_exists(&self, function_name: &str) -> crate::Result<bool> {
        let plugin = self.plugin.lock();

        Ok(plugin.function_exists(function_name))
    }
}
