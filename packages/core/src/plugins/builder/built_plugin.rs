//! Built plugin implementation

use std::sync::Arc;

use action_items_common::plugin_interface::ActionItem;
use bevy::tasks::{AsyncComputeTaskPool, Task};
use futures::future::BoxFuture;
use serde_json::Value;

use super::traits::LauncherPlugin;
use super::types::{ActionHandler, RefreshHandler, SearchHandler};
use crate::plugins::interface::{Error, NativePlugin, PluginContext, PluginManifest};
use crate::service_bridge::bridge::core::ServiceBridge;

/// Built plugin implementation
pub struct BuiltPlugin {
    pub(super) manifest: PluginManifest,
    pub(super) search_handler: Option<SearchHandler>,
    pub(super) action_handler: Option<ActionHandler>,
    pub(super) refresh_handler: Option<RefreshHandler>,
    pub(super) service_bridge: Arc<ServiceBridge>,
}

// Implement NativePlugin for BuiltPlugin
impl NativePlugin for BuiltPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn initialize(
        &mut self,
        _context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        task_pool.spawn(async { Ok(()) })
    }

    fn search(
        &self,
        query: String,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Vec<ActionItem>, Error>> {
        if let Some(handler) = &self.search_handler {
            let future = handler(query, context);
            task_pool.spawn(future)
        } else {
            task_pool.spawn(async { Ok(Vec::new()) })
        }
    }

    fn execute_command(
        &mut self,
        _command_id: String,
        _context: PluginContext,
        _args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        task_pool.spawn(async { Ok(None) })
    }

    fn execute_action(
        &mut self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<Option<Value>, Error>> {
        if let Some(handler) = &self.action_handler {
            let future = handler(action_id, context, args);
            task_pool.spawn(async move { future.await.map(|_| None) })
        } else {
            task_pool.spawn(async { Err(Error::PluginError("No action handler".to_string())) })
        }
    }

    fn background_refresh(
        &mut self,
        context: PluginContext,
        task_pool: &AsyncComputeTaskPool,
    ) -> Task<Result<(), Error>> {
        if let Some(handler) = &self.refresh_handler {
            let future = handler(context);
            task_pool.spawn(future)
        } else {
            task_pool.spawn(async { Ok(()) })
        }
    }

    fn cleanup(&mut self, task_pool: &AsyncComputeTaskPool) -> Task<Result<(), Error>> {
        task_pool.spawn(async { Ok(()) })
    }
}

// Implement LauncherPlugin for BuiltPlugin
impl LauncherPlugin for BuiltPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn search(
        &self,
        query: String,
        context: PluginContext,
    ) -> BoxFuture<'static, Result<Vec<ActionItem>, Error>> {
        if let Some(handler) = &self.search_handler {
            handler(query, context)
        } else {
            Box::pin(async { Ok(Vec::new()) })
        }
    }

    fn execute_action(
        &self,
        action_id: String,
        context: PluginContext,
        args: Option<Value>,
    ) -> BoxFuture<'static, Result<(), Error>> {
        if let Some(handler) = &self.action_handler {
            handler(action_id, context, args)
        } else {
            Box::pin(async { Err(Error::PluginError("No action handler".to_string())) })
        }
    }
}
