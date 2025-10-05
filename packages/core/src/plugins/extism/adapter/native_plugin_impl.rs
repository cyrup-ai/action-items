use action_items_native::Error;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use serde_json::Value;

use super::core::ExtismPluginAdapter;
use crate::plugins::interface::{PluginContext, PluginManifest};

impl action_items_native::native::NativePlugin for ExtismPluginAdapter {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn initialize(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        self.initialize(context, task_pool)
    }

    fn execute_command(
        &mut self,
        command_id: String,
        context: PluginContext,
        args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        self.execute_command(command_id, context, args, task_pool)
    }

    fn search(
        &self,
        query: String,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<action_items_common::plugin_interface::ActionItem>, Error>> {
        self.search(query, context, task_pool)
    }

    fn execute_action(
        &mut self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        self.execute_action(action_id, context, args, task_pool)
    }

    fn background_refresh(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        self.background_refresh(context, task_pool)
    }

    fn cleanup(&mut self, task_pool: &AsyncComputeTaskPool) -> Task<Result<(), Error>> {
        self.cleanup(task_pool)
    }
}
