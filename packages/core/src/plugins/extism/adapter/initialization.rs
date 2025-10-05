use std::sync::Arc;

use action_items_native::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};

use super::core::ExtismPluginAdapter;
use crate::plugins::interface::PluginContext;

impl ExtismPluginAdapter {
    pub fn initialize(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        let plugin_arc = Arc::clone(&self.plugin);
        let context_arc = Arc::clone(&self.context);

        task_pool.spawn(async move {
            let context_json = serde_json::to_string(&context).map_err(|e| {
                Error::PluginError(format!(
                    "ExtismAdapter: Failed to serialize context for initialize: {e}"
                ))
            })?;

            let mut plugin_guard = plugin_arc.lock();

            plugin_guard
                .call::<String, ()>("plugin_initialize", context_json)
                .map_err(|e| {
                    Error::PluginError(format!(
                        "ExtismAdapter: Failed to call plugin_initialize: {e}"
                    ))
                })?;

            let mut context_guard = context_arc.lock();
            *context_guard = Some(context);
            Ok(())
        })
    }
}
