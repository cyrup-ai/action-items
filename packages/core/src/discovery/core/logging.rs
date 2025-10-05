use log::info;

use super::types::DiscoveryConfig;
use crate::plugins::ecs_queries::{PluginCounter, PluginNames};

/// Logs information about loaded plugins
pub fn log_loaded_plugins(plugin_counter: PluginCounter, plugin_names: PluginNames) {
    let plugin_count = plugin_counter.count();
    info!("=== Loaded Plugins ===");
    info!("Total plugins: {}", plugin_count);

    for (index, name) in plugin_names.collect().iter().enumerate() {
        info!("  {}. {}", index + 1, name);
    }

    if plugin_count == 0 {
        info!("No plugins loaded. Place .wasm files in one of these directories:");
        let config = DiscoveryConfig::default();
        for dir in &config.plugin_directories {
            info!("  - {}", dir.display());
        }
    }
}
