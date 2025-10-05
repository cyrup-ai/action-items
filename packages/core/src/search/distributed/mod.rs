//! Distributed search functionality for coordinating searches across multiple plugins
//!
//! This module provides systems and types for managing distributed search operations
//! via the service bridge, including query orchestration, response processing,
//! and plugin health monitoring.

pub mod message_handling;
pub mod orchestration;
pub mod plugin_management;
pub mod response_processing;
pub mod result_management;
pub mod types;

// Re-export main types and functions for backward compatibility
pub use orchestration::distributed_search_system;
pub use plugin_management::{
    broadcast_capability_updates, discover_plugins_via_service_bridge, monitor_search_plugin_health,
};
pub use response_processing::{
    handle_search_response_messages, process_distributed_search_responses,
};
pub use types::{DistributedSearchManager, DistributedSearchQuery};
