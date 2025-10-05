//! Type definitions and aliases for plugin builder system

use std::sync::Arc;

use action_items_common::plugin_interface::ActionItem;
use futures::future::BoxFuture;
use serde_json::Value;

use crate::plugins::interface::{Error, PluginContext};

/// Type alias for search handler function signature
pub type SearchHandler = Arc<
    dyn Fn(String, PluginContext) -> BoxFuture<'static, Result<Vec<ActionItem>, Error>>
        + Send
        + Sync,
>;

/// Type alias for action handler function signature
pub type ActionHandler = Arc<
    dyn Fn(String, PluginContext, Option<Value>) -> BoxFuture<'static, Result<(), Error>>
        + Send
        + Sync,
>;

/// Type alias for refresh handler function signature
pub type RefreshHandler =
    Arc<dyn Fn(PluginContext) -> BoxFuture<'static, Result<(), Error>> + Send + Sync>;
