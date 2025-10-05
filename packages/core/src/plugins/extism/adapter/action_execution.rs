use std::sync::Arc;

use action_items_native::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde::Serialize;
use serde_json::Value;

use super::core::ExtismPluginAdapter;
use crate::plugins::interface::PluginContext;

impl ExtismPluginAdapter {
    pub fn execute_action(
        &mut self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        let plugin_arc = Arc::clone(&self.plugin);

        task_pool.spawn(async move {
            #[derive(Serialize)]
            struct ActionRequestInternal {
                action_id: String,
                args: Option<Value>,
                context: PluginContext,
            }

            let request = ActionRequestInternal {
                action_id,
                args,
                context,
            };

            let request_json = serde_json::to_string(&request).map_err(|e| {
                Error::PluginError(format!(
                    "ExtismAdapter: Failed to serialize action request: {e}"
                ))
            })?;

            let mut plugin_guard = plugin_arc.lock();

            let response_json =
                match plugin_guard.call::<String, String>("plugin_execute_action", request_json) {
                    Ok(response) => response,
                    Err(e) => {
                        return Err(Error::PluginError(format!(
                            "ExtismAdapter: Failed to call plugin_execute_action: {e}"
                        )));
                    },
                };

            if response_json.is_empty() || response_json.to_lowercase() == "null" {
                Ok(None)
            } else {
                let result_value: Value = serde_json::from_str(&response_json).map_err(|e| {
                    Error::PluginError(format!(
                        "ExtismAdapter: Failed to deserialize action result: {e}"
                    ))
                })?;
                Ok(Some(result_value))
            }
        })
    }
}
