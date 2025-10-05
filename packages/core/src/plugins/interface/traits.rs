use super::manifest::PluginManifest;
use std::path::Path;

/// Plugin loader trait for different plugin types
pub trait PluginLoader {
    type Plugin;

    fn load_from_path(&self, path: &Path) -> Result<Self::Plugin, String>;
    fn validate_manifest(&self, manifest: &PluginManifest) -> Result<(), String>;
    fn check_permissions(&self, manifest: &PluginManifest) -> Result<(), String>;
}
