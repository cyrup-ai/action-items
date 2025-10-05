//! Main PluginContext struct and core types

use std::collections::HashMap;

use bevy::prelude::*;
use crossbeam_channel::Sender as CrossbeamSender;
use serde;

use super::services::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, StorageService,
};
use crate::plugins::interface::commands::StorageCommand;

/// Plugin context provided to commands
/// This is a lightweight context that provides access to plugin configuration
/// Actual service operations are performed through Bevy systems and events
#[derive(Clone, serde::Serialize)]
pub struct PluginContext {
    pub plugin_id: String,
    pub config: HashMap<String, serde_json::Value>,
    pub preferences: HashMap<String, serde_json::Value>,
    pub environment: HashMap<String, String>,
    #[serde(skip)]
    pub clipboard: ClipboardAccess,
    #[serde(skip)]
    pub notifications: NotificationService,
    #[serde(skip)]
    pub storage: StorageService, // Direct access for native, tasks for Extism host fns
    #[serde(skip)]
    pub http: HttpClient,
    #[serde(skip)]
    pub cache: CacheService,
    // Sender for storage commands, to be used by Extism host functions via ServiceBridge
    #[serde(skip)]
    pub storage_sender: CrossbeamSender<StorageCommand>,
}

// Re-export from plugin_services for convenience
pub use crate::plugins::services::{PluginCache, StorageDirectory};
