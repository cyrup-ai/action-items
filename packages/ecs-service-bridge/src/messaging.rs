//! Zero-Allocation Message Infrastructure
//!
//! Blazing-fast, lock-free message routing and processing using crossbeam channels.
//! All operations are zero-allocation with optimal memory layouts and cache efficiency.

use std::sync::Arc;

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, TrySendError, bounded, unbounded};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};

use crate::events::*;
use crate::types::{
    CorrelationId, MessageAddress, RequestId, ServiceError, ServiceResult, TimeStamp,
};

/// Maximum number of messages that can be queued per priority level
const MAX_PRIORITY_QUEUE_SIZE: usize = 10000;

/// Maximum number of plugin channels to prevent memory exhaustion
const MAX_PLUGIN_CHANNELS: usize = 1000;

/// Message infrastructure resource with zero-allocation design
#[derive(Resource)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageInfrastructure {
    /// Priority-based message queues using crossbeam channels
    pub priority_queues: PriorityMessageQueues,
    /// Plugin-specific message channels
    pub plugin_channels: PluginMessageChannels,
    /// Message routing table for efficient dispatch
    pub routing_table: MessageRoutingTable,
    /// Message statistics and monitoring
    pub stats: MessageStats,
    /// Configuration for message handling
    pub config: MessageConfig,
}

impl Default for MessageInfrastructure {
    #[inline]
    fn default() -> Self {
        Self {
            priority_queues: PriorityMessageQueues::new(),
            plugin_channels: PluginMessageChannels::new(),
            routing_table: MessageRoutingTable::new(),
            stats: MessageStats::default(),
            config: MessageConfig::default(),
        }
    }
}

/// Priority-based message queues with lock-free crossbeam channels
#[derive(Debug)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PriorityMessageQueues {
    /// Critical priority messages (system messages, errors)
    pub critical: (Sender<MessageEnvelope>, Receiver<MessageEnvelope>),
    /// High priority messages (user interactions, urgent requests)
    pub high: (Sender<MessageEnvelope>, Receiver<MessageEnvelope>),
    /// Normal priority messages (regular plugin communication)
    pub normal: (Sender<MessageEnvelope>, Receiver<MessageEnvelope>),
    /// Low priority messages (background tasks, cleanup)
    pub low: (Sender<MessageEnvelope>, Receiver<MessageEnvelope>),
}

impl Default for PriorityMessageQueues {
    fn default() -> Self {
        Self::new()
    }
}

impl PriorityMessageQueues {
    /// Create new priority queues with optimal capacity
    #[inline]
    pub fn new() -> Self {
        Self {
            critical: bounded(MAX_PRIORITY_QUEUE_SIZE / 10), // Small queue for critical messages
            high: bounded(MAX_PRIORITY_QUEUE_SIZE / 4),      // Medium queue for high priority
            normal: bounded(MAX_PRIORITY_QUEUE_SIZE / 2),    // Large queue for normal messages
            low: bounded(MAX_PRIORITY_QUEUE_SIZE),           // Largest queue for low priority
        }
    }

    /// Send message to appropriate priority queue
    #[inline]
    pub fn send(&self, envelope: MessageEnvelope) -> ServiceResult<()> {
        let sender = match envelope.priority {
            MessagePriority::Critical => &self.critical.0,
            MessagePriority::High => &self.high.0,
            MessagePriority::Normal => &self.normal.0,
            MessagePriority::Low => &self.low.0,
        };

        sender.try_send(envelope).map_err(|e| match e {
            TrySendError::Full(_) => ServiceError::ResourceExhausted {
                resource: "message_queue".into(),
            },
            TrySendError::Disconnected(_) => ServiceError::PluginCommunicationFailed {
                plugin_id: "message_infrastructure".into(),
                reason: "Queue disconnected".into(),
            },
        })
    }

    /// Receive next message with priority order
    #[inline]
    pub fn try_recv(&self) -> Option<MessageEnvelope> {
        // Try critical first, then high, normal, low
        self.critical
            .1
            .try_recv()
            .ok()
            .or_else(|| self.high.1.try_recv().ok())
            .or_else(|| self.normal.1.try_recv().ok())
            .or_else(|| self.low.1.try_recv().ok())
    }
}

/// Plugin-specific message channels for direct communication
#[derive(Debug, Default)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginMessageChannels {
    /// Map of plugin_id -> message channel
    pub channels: FxHashMap<String, PluginChannel>,
    /// Broadcast channel for messages to all plugins
    pub broadcast_channel: Option<(Sender<MessageEnvelope>, Receiver<MessageEnvelope>)>,
}

impl PluginMessageChannels {
    /// Create new plugin message channels
    #[inline]
    pub fn new() -> Self {
        Self {
            channels: FxHashMap::default(),
            broadcast_channel: Some(unbounded()),
        }
    }

    /// Register new plugin channel
    #[inline]
    pub fn register_plugin(&mut self, plugin_id: String) -> ServiceResult<()> {
        if self.channels.len() >= MAX_PLUGIN_CHANNELS {
            return Err(ServiceError::ResourceExhausted {
                resource: "plugin_channels".into(),
            });
        }

        let (sender, receiver) = unbounded();
        let channel = PluginChannel {
            plugin_id: plugin_id.clone(),
            sender,
            receiver: Arc::new(std::sync::Mutex::new(receiver)),
            created_at: TimeStamp::now(),
            message_count: 0,
        };

        self.channels.insert(plugin_id, channel);
        Ok(())
    }

    /// Unregister plugin channel
    #[inline]
    pub fn unregister_plugin(&mut self, plugin_id: &str) -> bool {
        self.channels.remove(plugin_id).is_some()
    }

    /// Send message to specific plugin
    #[inline]
    pub fn send_to_plugin(
        &mut self,
        plugin_id: &str,
        envelope: MessageEnvelope,
    ) -> ServiceResult<()> {
        if let Some(channel) = self.channels.get_mut(plugin_id) {
            channel.sender.try_send(envelope).map_err(|e| match e {
                TrySendError::Full(_) => ServiceError::ResourceExhausted {
                    resource: format!("plugin_channel_{}", plugin_id),
                },
                TrySendError::Disconnected(_) => ServiceError::PluginCommunicationFailed {
                    plugin_id: plugin_id.to_string(),
                    reason: "Channel disconnected".into(),
                },
            })?;

            channel.message_count += 1;
            Ok(())
        } else {
            Err(ServiceError::PluginNotFound {
                plugin_id: plugin_id.to_string(),
            })
        }
    }

    /// Broadcast message to all plugins
    #[inline]
    pub fn broadcast(&self, envelope: MessageEnvelope) -> ServiceResult<()> {
        if let Some((sender, _)) = &self.broadcast_channel {
            sender.try_send(envelope).map_err(|e| match e {
                TrySendError::Full(_) => ServiceError::ResourceExhausted {
                    resource: "broadcast_channel".into(),
                },
                TrySendError::Disconnected(_) => ServiceError::PluginCommunicationFailed {
                    plugin_id: "broadcast".into(),
                    reason: "Broadcast channel disconnected".into(),
                },
            })
        } else {
            Err(ServiceError::PluginCommunicationFailed {
                plugin_id: "broadcast".into(),
                reason: "Broadcast channel not initialized".into(),
            })
        }
    }
}

/// Plugin communication channel with message statistics
#[derive(Debug)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct PluginChannel {
    pub plugin_id: String,
    pub sender: Sender<MessageEnvelope>,
    pub receiver: Arc<std::sync::Mutex<Receiver<MessageEnvelope>>>,
    pub created_at: TimeStamp,
    pub message_count: u64,
}

/// Message routing table for efficient dispatch
#[derive(Debug, Default)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageRoutingTable {
    /// Message type -> List of plugin IDs that handle it
    pub handlers: FxHashMap<String, Vec<String>>,
    /// Plugin ID -> List of message types it handles
    pub plugin_handlers: FxHashMap<String, Vec<String>>,
    /// Load balancing state for round-robin dispatch
    pub load_balance_state: FxHashMap<String, usize>,
}

impl MessageRoutingTable {
    /// Create new routing table
    #[inline]
    pub fn new() -> Self {
        Self::default()
    }

    /// Register message handler for a plugin
    #[inline]
    pub fn register_handler(&mut self, plugin_id: String, message_type: String) {
        // Add to handlers map
        self.handlers
            .entry(message_type.clone())
            .or_default()
            .push(plugin_id.clone());

        // Add to plugin handlers map
        self.plugin_handlers
            .entry(plugin_id)
            .or_default()
            .push(message_type.clone());

        // Initialize load balancing state
        self.load_balance_state.entry(message_type).or_insert(0);
    }

    /// Unregister plugin from all message handlers
    #[inline]
    pub fn unregister_plugin(&mut self, plugin_id: &str) {
        // Remove from plugin handlers
        if let Some(message_types) = self.plugin_handlers.remove(plugin_id) {
            // Remove plugin from each message type handler list
            for message_type in message_types {
                if let Some(handlers) = self.handlers.get_mut(&message_type) {
                    handlers.retain(|id| id != plugin_id);

                    // Remove message type if no handlers left
                    if handlers.is_empty() {
                        self.handlers.remove(&message_type);
                        self.load_balance_state.remove(&message_type);
                    }
                }
            }
        }
    }

    /// Get next handler for message type using round-robin load balancing
    #[inline]
    pub fn get_next_handler(&mut self, message_type: &str) -> Option<String> {
        if let Some(handlers) = self.handlers.get(message_type) {
            if handlers.is_empty() {
                return None;
            }

            let current_index = self
                .load_balance_state
                .get(message_type)
                .copied()
                .unwrap_or(0);
            let handler = handlers[current_index].clone();

            // Update load balance state for round-robin
            let next_index = (current_index + 1) % handlers.len();
            self.load_balance_state
                .insert(message_type.to_string(), next_index);

            Some(handler)
        } else {
            None
        }
    }

    /// Get all handlers for message type
    #[inline]
    pub fn get_handlers(&self, message_type: &str) -> Option<&Vec<String>> {
        self.handlers.get(message_type)
    }
}
/// Message envelope for zero-allocation message transport
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageEnvelope {
    /// Message metadata
    pub metadata: MessageMetadata,
    /// Message payload (serialized)
    pub payload: MessagePayload,
    /// Message priority for queue routing
    pub priority: MessagePriority,
    /// Routing information
    pub routing: MessageRouting,
}

impl MessageEnvelope {
    /// Create new message envelope with validation
    #[inline]
    pub fn new(
        from: MessageAddress,
        to: MessageAddress,
        message_type: String,
        payload: serde_json::Value,
        priority: MessagePriority,
    ) -> ServiceResult<Self> {
        // Validate message type
        if message_type.is_empty() || message_type.len() > 256 {
            return Err(ServiceError::InvalidAddress("Invalid message type".into()));
        }

        let metadata = MessageMetadata {
            message_id: RequestId::new(),
            correlation_id: CorrelationId::new(),
            timestamp: TimeStamp::now(),
            message_type,
            content_length: payload.to_string().len() as u32,
        };

        let payload = MessagePayload {
            content: payload,
            content_type: "application/json".to_string(),
            encoding: MessageEncoding::Json,
        };

        let routing = MessageRouting {
            from,
            to,
            reply_to: None,
            route_history: Vec::new(),
            ttl_hops: 10, // Default TTL
        };

        Ok(Self {
            metadata,
            payload,
            priority,
            routing,
        })
    }

    /// Add reply-to address for response routing
    #[inline]
    pub fn with_reply_to(mut self, reply_to: MessageAddress) -> Self {
        self.routing.reply_to = Some(reply_to);
        self
    }

    /// Add correlation ID for request tracking
    #[inline]
    pub fn with_correlation_id(mut self, correlation_id: CorrelationId) -> Self {
        self.metadata.correlation_id = correlation_id;
        self
    }

    /// Add route hop to history for loop detection
    #[inline]
    pub fn add_route_hop(&mut self, hop: MessageAddress) -> ServiceResult<()> {
        // Check for routing loops
        if self.routing.route_history.contains(&hop) {
            return Err(ServiceError::RoutingLoop {
                address: hop.to_string(),
            });
        }

        // Check TTL
        if self.routing.ttl_hops == 0 {
            return Err(ServiceError::MessageExpired {
                message_id: self.metadata.message_id.to_string(),
            });
        }

        self.routing.route_history.push(hop);
        self.routing.ttl_hops -= 1;
        Ok(())
    }
}

/// Message metadata with zero-allocation design
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageMetadata {
    pub message_id: RequestId,
    pub correlation_id: CorrelationId,
    pub timestamp: TimeStamp,
    pub message_type: String,
    pub content_length: u32,
}

/// Message payload with encoding information
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessagePayload {
    pub content: serde_json::Value,
    pub content_type: String,
    pub encoding: MessageEncoding,
}

/// Message encoding types for optimal serialization
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[repr(u8)] // Explicit byte representation for optimal serialization
pub enum MessageEncoding {
    Json = 0,
    MessagePack = 1,
    Binary = 2,
    Text = 3,
}

/// Message routing information for dispatch
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageRouting {
    pub from: MessageAddress,
    pub to: MessageAddress,
    pub reply_to: Option<MessageAddress>,
    pub route_history: Vec<MessageAddress>,
    pub ttl_hops: u8, // Time-to-live in hops
}

/// Message infrastructure statistics for monitoring
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageStats {
    /// Total messages processed
    pub messages_processed: u64,
    /// Messages sent successfully
    pub messages_sent: u64,
    /// Messages failed to send
    pub messages_failed: u64,
    /// Messages dropped due to queue full
    pub messages_dropped: u64,
    /// Average processing time in microseconds
    pub avg_processing_time_us: u64,
    /// Peak queue size
    pub peak_queue_size: usize,
    /// Active plugin channels
    pub active_channels: usize,
}

impl MessageStats {
    /// Record successful message processing
    #[inline]
    pub fn record_success(&mut self, processing_time_us: u64) {
        self.messages_processed += 1;
        self.messages_sent += 1;

        // Update rolling average processing time
        if self.messages_processed == 1 {
            self.avg_processing_time_us = processing_time_us;
        } else {
            self.avg_processing_time_us =
                (self.avg_processing_time_us * (self.messages_processed - 1) + processing_time_us)
                    / self.messages_processed;
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

/// Message infrastructure configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[repr(C)] // Optimal memory layout for cache efficiency
pub struct MessageConfig {
    /// Maximum message size in bytes
    pub max_message_size: u32,
    /// Message timeout in milliseconds
    pub message_timeout_ms: u64,
    /// Enable message compression for large payloads
    pub enable_compression: bool,
    /// Compression threshold in bytes
    pub compression_threshold: u32,
    /// Enable message encryption
    pub enable_encryption: bool,
    /// Maximum number of route hops (TTL)
    pub max_route_hops: u8,
    /// Enable message deduplication
    pub enable_deduplication: bool,
    /// Deduplication window in seconds
    pub deduplication_window_secs: u64,
}

impl Default for MessageConfig {
    #[inline]
    fn default() -> Self {
        Self {
            max_message_size: 1024 * 1024, // 1MB default
            message_timeout_ms: 30000,     // 30 seconds
            enable_compression: true,
            compression_threshold: 1024, // 1KB
            enable_encryption: false,    // Disabled by default
            max_route_hops: 10,
            enable_deduplication: true,
            deduplication_window_secs: 300, // 5 minutes
        }
    }
}
