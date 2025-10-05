use std::collections::HashSet;

use log::{debug, info, warn};

use super::scanning::scan_directory_for_plugin_wrappers;
use super::types::{DiscoveredPlugin, DiscoveryConfig};

/// Discovers and creates plugin wrappers from configured directories
pub fn discover_plugin_wrappers() -> Vec<DiscoveredPlugin> {
    let config = DiscoveryConfig::default();
    let mut discovered_wrappers = Vec::new();
    let mut visited_paths = HashSet::new();

    for dir in &config.plugin_directories {
        if !dir.exists() {
            // Try to create the directory if it doesn't exist
            if let Err(e) = std::fs::create_dir_all(dir) {
                debug!("Failed to create plugin directory {}: {}", dir.display(), e);
                continue;
            }
            info!("Created plugin directory: {}", dir.display());
        }

        match scan_directory_for_plugin_wrappers(dir, &config, &mut visited_paths) {
            Ok(wrappers) => {
                info!(
                    "Found {} plugin wrappers in {}",
                    wrappers.len(),
                    dir.display()
                );
                discovered_wrappers.extend(wrappers);
            },
            Err(e) => {
                warn!("Failed to scan plugin directory {}: {}", dir.display(), e);
            },
        }
    }

    info!(
        "Plugin wrapper discovery complete: {} wrappers created",
        discovered_wrappers.len()
    );

    discovered_wrappers
}
