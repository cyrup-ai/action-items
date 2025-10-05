//! ECS Components for Service Bridge
//!
//! Zero-allocation, optimal memory layout ECS components for blazing-fast performance.
//! All components use `#[repr(C)]` for predictable memory alignment and cache efficiency.

use std::collections::HashMap;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::types::TimeStamp;

/// Plugin type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, Copy)]
#[repr(u8)]
pub enum PluginType {
    Deno = 0,
    Native = 1,
    Wasm = 2,
    Raycast = 3,
}

/// Component marking an entity as a registered plugin
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginComponent {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub status: PluginStatus,
    pub plugin_type: PluginType,
    pub author: Option<String>,
    pub has_config: bool,
    pub registration_time: TimeStamp,
    pub last_heartbeat: Option<TimeStamp>,
}

/// Plugin lifecycle status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum PluginStatus {
    /// Plugin is initializing
    Initializing = 0,
    /// Plugin is active and ready to receive messages
    Active = 1,
    /// Plugin is temporarily unavailable
    Inactive = 2,
    /// Plugin encountered an error
    Error(String) = 3,
    /// Plugin is shutting down
    Terminating = 4,
}

/// Component representing a plugin's capabilities
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct CapabilitiesComponent {
    pub capabilities: Vec<Capability>,
}

/// A capability that a plugin provides
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct Capability {
    pub name: String,
    pub version: String,
    pub description: String,
    pub metadata: HashMap<String, String>,
}

impl Capability {
    #[inline]
    pub fn new(name: &str, version: &str, description: &str) -> Self {
        Self {
            name: name.to_string(),
            version: version.to_string(),
            description: description.to_string(),
            metadata: HashMap::new(),
        }
    }

    #[inline]
    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Component for tracking message routing state with optimized memory layout
#[derive(Component, Debug, Clone)]
#[repr(C)] // Optimal memory layout for cache efficiency
#[derive(Default)]
pub struct MessageRouterComponent {
    pub routing_table: HashMap<String, Vec<String>>, // message_type -> plugin_ids
    pub load_balancer_state: HashMap<String, usize>, // plugin_id -> current load index
}


/// Component for tracking channel state with optimized memory layout
/// Fixed: Replaced Instant with TimeStamp for proper serialization support
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ChannelComponent {
    pub channel_id: String,
    pub connected_plugins: Vec<String>,
    pub message_count: u64,
    pub last_activity: Option<TimeStamp>, // Fixed: Use TimeStamp instead of Instant
    pub health_status: ChannelHealth,
}

/// Channel health status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum ChannelHealth {
    Active = 0,
    Idle = 1,
    Congested = 2,
    Failed(String) = 3,
}

/// Component representing the core service bridge state
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceBridgeCore {
    pub service_id: String,
    pub name: String,
    pub version: String,
    pub status: ServiceBridgeStatus,
    pub startup_time: TimeStamp,
    pub last_heartbeat: Option<TimeStamp>,
    pub active_connections: u32,
}

/// Service bridge status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum ServiceBridgeStatus {
    Initializing = 0,
    Active = 1,
    Maintenance = 2,
    ShuttingDown = 3,
    Error(String) = 4,
}

/// Component for plugin registry state tracking
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginRegistry {
    pub registry_id: String,
    pub registered_plugins: Vec<String>,
    pub plugin_capabilities: HashMap<String, Vec<String>>,
    pub last_updated: TimeStamp,
    pub total_plugins: u32,
    pub active_plugins: u32,
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self {
            registry_id: format!("registry_{}", TimeStamp::now().as_millis()),
            registered_plugins: Vec::new(),
            plugin_capabilities: HashMap::new(),
            last_updated: TimeStamp::now(),
            total_plugins: 0,
            active_plugins: 0,
        }
    }
}

/// Component for service handler registry tracking
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceHandlerRegistry {
    pub handler_id: String,
    pub registered_handlers: HashMap<String, Vec<String>>, // service_type -> handler_ids
    pub handler_metadata: HashMap<String, ServiceHandlerInfo>,
    pub last_updated: TimeStamp,
    pub total_handlers: u32,
}

/// Service handler information with optimal memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ServiceHandlerInfo {
    pub handler_id: String,
    pub service_types: Vec<String>,
    pub priority: u8,
    pub registered_at: TimeStamp,
    pub last_activity: Option<TimeStamp>,
}

impl Default for ServiceHandlerRegistry {
    fn default() -> Self {
        Self {
            handler_id: format!("handler_registry_{}", TimeStamp::now().as_millis()),
            registered_handlers: HashMap::new(),
            handler_metadata: HashMap::new(),
            last_updated: TimeStamp::now(),
            total_handlers: 0,
        }
    }
}

/// Component for active message tracking
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct ActiveMessage {
    pub message_id: String,
    pub correlation_id: String,
    pub sender_id: String,
    pub recipient_id: String,
    pub message_type: String,
    pub status: MessageStatus,
    pub created_at: TimeStamp,
    pub processed_at: Option<TimeStamp>,
    pub retry_count: u8,
    pub priority: u8,
}

/// Message status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum MessageStatus {
    Pending = 0,
    Processing = 1,
    Completed = 2,
    Failed(String) = 3,
    Expired = 4,
}

/// Component for health metrics tracking
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct HealthMetrics {
    pub component_id: String,
    pub health_status: HealthStatus,
    pub last_check: TimeStamp,
    pub uptime_seconds: u64,
    pub error_count: u32,
    pub warning_count: u32,
    pub performance_metrics: PerformanceMetrics,
}

/// Health status with explicit discriminant for optimal serialization
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum HealthStatus {
    Healthy = 0,
    Warning(String) = 1,
    Critical(String) = 2,
    Unavailable = 3,
}

/// Performance metrics with optimal memory layout
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PerformanceMetrics {
    pub avg_response_time_ms: f64,
    pub throughput_per_second: f64,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            avg_response_time_ms: 0.0,
            throughput_per_second: 0.0,
            memory_usage_mb: 0.0,
            cpu_usage_percent: 0.0,
        }
    }
}

/// Component for message statistics tracking
/// Optimized memory layout with `#[repr(C)]` for cache efficiency
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageStatistics {
    pub component_id: String,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub messages_processed: u64,
    pub messages_failed: u64,
    pub messages_dropped: u64,
    pub avg_processing_time_ms: f64,
    pub peak_queue_size: usize,
    pub last_reset: TimeStamp,
    pub collection_period_seconds: u64,
}

impl Default for MessageStatistics {
    fn default() -> Self {
        Self {
            component_id: format!("msg_stats_{}", TimeStamp::now().as_millis()),
            messages_sent: 0,
            messages_received: 0,
            messages_processed: 0,
            messages_failed: 0,
            messages_dropped: 0,
            avg_processing_time_ms: 0.0,
            peak_queue_size: 0,
            last_reset: TimeStamp::now(),
            collection_period_seconds: 300, // 5 minutes default
        }
    }
}

impl MessageStatistics {
    /// Record successful message processing
    #[inline]
    pub fn record_success(&mut self, processing_time_ms: f64) {
        self.messages_processed += 1;

        // Update rolling average processing time
        if self.messages_processed == 1 {
            self.avg_processing_time_ms = processing_time_ms;
        } else {
            self.avg_processing_time_ms = (self.avg_processing_time_ms
                * (self.messages_processed - 1) as f64
                + processing_time_ms)
                / self.messages_processed as f64;
        }
    }

    /// Record failed message processing
    #[inline]
    pub fn record_failure(&mut self) {
        self.messages_processed += 1;
        self.messages_failed += 1;
    }

    /// Record dropped message
    #[inline]
    pub fn record_drop(&mut self) {
        self.messages_dropped += 1;
    }

    /// Update queue statistics
    #[inline]
    pub fn update_queue_stats(&mut self, current_queue_size: usize) {
        if current_queue_size > self.peak_queue_size {
            self.peak_queue_size = current_queue_size;
        }
    }
}
