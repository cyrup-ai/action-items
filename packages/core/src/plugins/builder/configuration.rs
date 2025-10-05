//! Plugin builder configuration methods

use std::sync::Arc;

use action_items_common::plugin_interface::ActionItem;
use serde_json::Value;

use super::plugin_builder::PluginBuilder;
use crate::plugins::interface::{Error, PluginContext};

impl PluginBuilder {
    /// Set plugin version
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.manifest.version = version.into();
        self
    }

    /// Set plugin author
    pub fn author(mut self, author: impl Into<String>) -> Self {
        self.manifest.author = author.into();
        self
    }

    /// Set plugin description
    pub fn description(mut self, desc: impl Into<String>) -> Self {
        self.manifest.description = desc.into();
        self
    }

    /// Add keywords
    pub fn keywords(mut self, keywords: Vec<String>) -> Self {
        self.manifest.keywords = keywords;
        self
    }

    /// Set search handler
    pub fn on_search<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(String, PluginContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<Vec<ActionItem>, Error>> + Send + 'static,
    {
        self.manifest.capabilities.search = true;
        self.search_handler = Some(Arc::new(move |query, ctx| Box::pin(handler(query, ctx))));
        self
    }

    /// Set action handler
    pub fn on_action<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(String, PluginContext, Option<Value>) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.manifest.capabilities.quick_actions = true;
        self.action_handler = Some(Arc::new(move |id, ctx, meta| {
            Box::pin(handler(id, ctx, meta))
        }));
        self
    }

    /// Set refresh handler
    pub fn on_refresh<F, Fut>(mut self, handler: F) -> Self
    where
        F: Fn(PluginContext) -> Fut + Send + Sync + 'static,
        Fut: std::future::Future<Output = Result<(), Error>> + Send + 'static,
    {
        self.manifest.capabilities.background_refresh = true;
        self.refresh_handler = Some(Arc::new(move |ctx| Box::pin(handler(ctx))));
        self
    }
}
