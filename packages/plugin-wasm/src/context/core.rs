//! Main PluginContext struct and core types using modern event-driven architecture

use std::collections::HashMap;

use bevy::prelude::*;
use crossbeam_channel::Sender as CrossbeamSender;
use serde;

use super::events::{StorageReadRequest, StorageWriteRequest};
use super::services::{
    CacheService, ClipboardAccess, HttpClient, NotificationService, StorageService,
};

/// Plugin context provided to commands using modern event-driven architecture
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
    pub storage: StorageService, // Direct access for native, events for Extism host fns
    #[serde(skip)]
    pub http: HttpClient,
    #[serde(skip)]
    pub cache: CacheService,
    // Modern event-driven senders for zero-allocation communication
    #[serde(skip)]
    pub storage_read_sender: CrossbeamSender<StorageReadRequest>,
    #[serde(skip)]
    pub storage_write_sender: CrossbeamSender<StorageWriteRequest>,
}
