use std::path::{Path, PathBuf};

use crate::plugins::extism::wrapper::ExtismPluginWrapper;
use crate::plugins::native::wrapper::NativePluginWrapper;
use crate::raycast::wrapper::RaycastPluginWrapper;
use crate::runtime::plugin_wrapper::core::DenoPluginWrapper;

/// Trait for unified metadata access across all plugin types
pub trait MetadataProvider {
    fn get_id(&self) -> &str;
    fn get_name(&self) -> &str;
    fn get_description(&self) -> &str;
    fn get_version(&self) -> &str;
    fn get_path(&self) -> Option<&Path>;
}

/// Unified plugin wrapper type for discovery
#[derive(Clone)]
pub enum DiscoveredPlugin {
    Native(NativePluginWrapper),
    Extism(ExtismPluginWrapper),
    Raycast(RaycastPluginWrapper),
    Deno(DenoPluginWrapper),
}

impl DiscoveredPlugin {
    /// Get the plugin name for logging
    pub fn name(&self) -> &str {
        match self {
            Self::Native(wrapper) => &wrapper.metadata().name,
            Self::Extism(wrapper) => &wrapper.metadata().name,
            Self::Raycast(wrapper) => &wrapper.metadata().name,
            Self::Deno(wrapper) => &wrapper.metadata().name,
        }
    }

    /// Get the plugin ID
    pub fn id(&self) -> &str {
        match self {
            Self::Native(wrapper) => &wrapper.metadata().id,
            Self::Extism(wrapper) => &wrapper.metadata().id,
            Self::Raycast(wrapper) => &wrapper.metadata().id,
            Self::Deno(wrapper) => &wrapper.metadata().id,
        }
    }
}

/// Plugin discovery configuration
pub struct DiscoveryConfig {
    pub plugin_directories: Vec<PathBuf>,
    pub auto_discover: bool,
    pub recursive_scan: bool,
    pub max_depth: usize,
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        let mut dirs = Vec::new();

        // Development directory
        dirs.push(PathBuf::from("./plugins"));

        // User-specific directories
        if let Some(config_dir) = dirs::config_dir() {
            dirs.push(config_dir.join("Action Items/plugins"));
        }
        if let Some(data_dir) = dirs::data_local_dir() {
            dirs.push(data_dir.join("Action Items/plugins"));
        }

        // System directories
        #[cfg(target_os = "linux")]
        {
            dirs.push(PathBuf::from("/usr/share/action-items/plugins"));
            dirs.push(PathBuf::from("/usr/local/share/action-items/plugins"));
        }

        #[cfg(target_os = "macos")]
        {
            dirs.push(PathBuf::from(
                "/Applications/Action Items.app/Contents/Resources/plugins",
            ));
            if let Some(home) = dirs::home_dir() {
                dirs.push(home.join("Library/Application Support/Action Items/plugins"));
            }
        }

        #[cfg(target_os = "windows")]
        {
            if let Ok(program_files) = std::env::var("ProgramFiles") {
                dirs.push(PathBuf::from(program_files).join("Action Items/plugins"));
            }
            if let Some(app_data) = dirs::data_dir() {
                dirs.push(app_data.join("Action Items/plugins"));
            }
        }

        Self {
            plugin_directories: dirs,
            auto_discover: true,
            recursive_scan: true,
            max_depth: 3,
        }
    }
}
