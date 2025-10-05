// Note: PathBuf, bevy prelude, and info logging will be used when plugin wrapper core is
// implemented

use crate::error::Result;
use crate::plugins::core::PluginMetadata;
use crate::plugins::services::PluginId;
// Note: Search types will be used when plugin wrapper search integration is implemented

/// Bevy Plugin wrapper around Deno Runtime
///
/// This wrapper provides integration between the Deno runtime and the Bevy ECS system,
/// allowing JavaScript/TypeScript plugins to be managed alongside native plugins.
#[derive(Clone)]
pub struct DenoPluginWrapper {
    /// Plugin metadata for registration and discovery
    pub(super) metadata: PluginMetadata,
    /// Plugin ID within the Deno runtime
    pub(super) plugin_id: PluginId,
}

impl DenoPluginWrapper {
    /// Create a new wrapper around a Deno plugin
    pub fn new(plugin_id: PluginId, metadata: PluginMetadata) -> Result<Self> {
        Ok(Self {
            metadata,
            plugin_id,
        })
    }

    /// Get the plugin metadata
    pub fn metadata(&self) -> &PluginMetadata {
        &self.metadata
    }

    /// Get the plugin ID
    pub fn plugin_id(&self) -> &PluginId {
        &self.plugin_id
    }
}
