use super::core::ConfigManager;
use crate::error::Result;

impl ConfigManager {
    /// Load plugin configuration into context
    pub async fn load_plugin_context(
        &self,
        plugin_id: &str,
        context: &mut crate::plugins::interface::PluginContext,
    ) -> Result<()> {
        if let Some(config) = self.get_config(plugin_id).await {
            context.config = config.configuration;
            context.preferences = config.preferences;
        }
        Ok(())
    }
}
