use std::sync::Arc;

use action_items_native::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use log::debug;

use super::core::ExtismPluginAdapter;
use crate::plugins::interface::PluginContext;

impl ExtismPluginAdapter {
    pub fn background_refresh(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        let plugin_arc = Arc::clone(&self.plugin);

        task_pool.spawn(async move {
            let mut plugin_guard = plugin_arc.lock();

            if plugin_guard.function_exists("plugin_background_refresh") {
                let context_json = serde_json::to_string(&context).map_err(|e| {
                    Error::PluginError(format!(
                        "ExtismAdapter: Failed to serialize context for background_refresh: {e}"
                    ))
                })?;

                plugin_guard
                    .call::<String, ()>("plugin_background_refresh", context_json)
                    .map_err(|e| {
                        Error::PluginError(format!(
                            "ExtismAdapter: Failed to call plugin_background_refresh: {e}"
                        ))
                    })?;
            }
            Ok(())
        })
    }

    pub fn cleanup(&mut self, task_pool: &AsyncComputeTaskPool) -> Task<Result<(), Error>> {
        let manifest_id = self.manifest.id.clone();
        task_pool.spawn(async move {
            debug!(
                "ExtismPluginAdapter cleanup called for plugin: {}",
                manifest_id
            );
            Ok(())
        })
    }
}
