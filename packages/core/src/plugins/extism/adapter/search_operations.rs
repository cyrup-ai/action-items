use std::sync::Arc;

use action_items_common::plugin_interface::ActionItem;
use action_items_native::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde::Serialize;

use super::core::ExtismPluginAdapter;
use crate::plugins::interface::PluginContext;

impl ExtismPluginAdapter {
    pub fn search(
        &self,
        query: String,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, Error>> {
        let plugin_arc = Arc::clone(&self.plugin);

        task_pool.spawn(async move {
            #[derive(Serialize)]
            struct SearchRequestInternal {
                query: String,
                context: PluginContext,
            }

            let request = SearchRequestInternal { query, context };

            let request_json = serde_json::to_string(&request).map_err(|e| {
                Error::PluginError(format!(
                    "ExtismAdapter: Failed to serialize search request: {e}"
                ))
            })?;

            let mut plugin_guard = plugin_arc.lock();

            let response_json = plugin_guard
                .call::<String, String>("plugin_search", request_json)
                .map_err(|e| {
                    Error::PluginError(format!("ExtismAdapter: Failed to call plugin_search: {e}"))
                })?;

            let results: Vec<ActionItem> = serde_json::from_str(&response_json).map_err(|e| {
                Error::PluginError(format!(
                    "ExtismAdapter: Failed to deserialize search results: {e}"
                ))
            })?;

            Ok(results)
        })
    }
}
