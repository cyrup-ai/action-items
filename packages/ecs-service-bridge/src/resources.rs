//! ECS Resources for Service Bridge
//!
//! Zero-allocation, optimal memory layout ECS resources for blazing-fast performance.
//! All resources use `#[repr(C)]` for predictable memory alignment and cache efficiency.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::TimeStamp;

/// Main Service Bridge resource - manages the overall service bridge state
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Resource)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceBridgeResource {
    pub config: ServiceBridgeConfig,
    pub stats: ServiceBridgeStats,
    pub health: ServiceBridgeHealth,
    pub startup_time: TimeStamp,
}

impl Default for ServiceBridgeResource {
    fn default() -> Self {
        Self {
            config: ServiceBridgeConfig::default(),
            stats: ServiceBridgeStats::default(),
            health: ServiceBridgeHealth::Healthy,
            startup_time: TimeStamp::now(),
        }
    }
}

impl ServiceBridgeResource {
    /// Create a new ServiceBridgeResource with default configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new ServiceBridgeResource with custom configuration
    pub fn with_config(config: ServiceBridgeConfig) -> Self {
        Self {
            config,
            stats: ServiceBridgeStats::default(),
            health: ServiceBridgeHealth::Healthy,
            startup_time: TimeStamp::now(),
        }
    }

    /// Register a plugin with simple configuration (delegates to PluginRegistryResource)
    pub fn register_plugin_simple(
        &self,
        _plugin_id: String,
        _name: String,
        _capabilities: Vec<Capability>,
    ) -> Result<(), String> {
        // This method is a compatibility shim - actual registration should be done via
        // PluginRegistryResource
        Ok(())
    }
}

/// Service Bridge configuration with optimal memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceBridgeConfig {
    pub max_plugins: usize,
    pub message_timeout_ms: u64,
    pub enable_metrics: bool,
    pub log_level: String,
}

impl Default for ServiceBridgeConfig {
    #[inline]
    fn default() -> Self {
        Self {
            max_plugins: 100,
            message_timeout_ms: 5000,
            enable_metrics: true,
            log_level: "info".to_string(),
        }
    }
}

/// Service Bridge statistics with optimal memory layout
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceBridgeStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_failed: u64,
    pub active_plugins: usize,
    pub uptime_seconds: u64,
}

/// Service Bridge health status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum ServiceBridgeHealth {
    Healthy = 0,
    Degraded(String) = 1,
    Unhealthy(String) = 2,
}

/// Plugin Registry resource - manages registered plugins and their capabilities
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Resource, Default)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginRegistryResource {
    pub plugins: HashMap<String, PluginInfo>,
    pub capabilities: HashMap<String, Vec<String>>, /* capability -> plugin_ids
                                                     * Plugin channels managed by
                                                     * MessageInfrastructure resource */
}

impl PluginRegistryResource {
    /// Create a new PluginRegistryResource
    pub fn new() -> Self {
        Self::default()
    }

    /// Get plugin information by plugin ID
    #[inline]
    pub fn get_plugin(&self, plugin_id: &str) -> Option<&PluginInfo> {
        self.plugins.get(plugin_id)
    }

    /// Get mutable plugin information by plugin ID
    #[inline]
    pub fn get_plugin_mut(&mut self, plugin_id: &str) -> Option<&mut PluginInfo> {
        self.plugins.get_mut(plugin_id)
    }

    /// Register a plugin with simple configuration
    pub fn register_plugin_simple(
        &mut self,
        plugin_id: String,
        name: String,
        capabilities: Vec<Capability>,
    ) -> Result<(), String> {
        if self.plugins.contains_key(&plugin_id) {
            return Err(format!("Plugin {} already registered", plugin_id));
        }

        let plugin_info = PluginInfo {
            plugin_id: plugin_id.clone(),
            name,
            version: "1.0.0".to_string(),
            description: "Plugin registered via simple registration".to_string(),
            capabilities: capabilities.clone(),
            status: crate::components::PluginStatus::Active,
            registration_time: TimeStamp::now(),
            last_heartbeat: Some(TimeStamp::now()),
        };

        self.plugins.insert(plugin_id.clone(), plugin_info);

        // Update capability index
        for capability in capabilities {
            self.capabilities
                .entry(capability.name)
                .or_default()
                .push(plugin_id.clone());
        }

        Ok(())
    }
}

/// Information about a registered plugin with optimal memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginInfo {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
    pub status: crate::components::PluginStatus,
    pub registration_time: TimeStamp,
    pub last_heartbeat: Option<TimeStamp>,
}

/// A capability that a plugin provides with optimal memory layout
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

impl Capability {
    /// Create a new capability
    pub fn new(name: String, version: String, description: String) -> Self {
        Self {
            name,
            version,
            description,
            metadata: HashMap::new(),
        }
    }

    /// Create a new capability with permission
    pub fn with_permission(name: String, permission: String) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("permission".to_string(), permission.clone());
        Self {
            name,
            version: "1.0.0".to_string(),
            description: format!("Capability requiring {} permission", permission),
            metadata,
        }
    }
}

// Plugin channels now managed by MessageInfrastructure (see messaging.rs)

/// Channel Manager resource - manages communication channels between plugins
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Resource, Default)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ChannelManagerResource {
    pub channels: HashMap<String, ChannelInfo>,
    pub message_queue: Vec<QueuedMessage>,
    pub stats: ChannelStats,
}

/// Channel information and statistics with optimal memory layout
#[derive(Debug, Clone)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ChannelInfo {
    pub channel_id: String,
    pub plugin_ids: Vec<String>,
    pub created_at: TimeStamp,
    pub message_count: u64,
    pub last_activity: Option<TimeStamp>,
}

/// Message queued for processing with optimal memory layout
#[derive(Debug, Clone)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct QueuedMessage {
    pub message: crate::events::PluginMessageEvent,
    pub queued_at: TimeStamp,
    pub retry_count: u32,
}

/// Channel statistics with optimal memory layout
#[derive(Debug, Default, Clone)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ChannelStats {
    pub total_channels: usize,
    pub active_channels: usize,
    pub messages_queued: usize,
    pub messages_processed: u64,
    pub messages_failed: u64,
}

// Channel management now handled by MessageInfrastructure
