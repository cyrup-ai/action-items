//! Plugin bridge core types
//!
//! This module provides the bridge layer between the main service_bridge infrastructure 
//! and the plugin interface layer. It re-exports the main ServiceBridge for
//! plugin consumption while maintaining clean architectural separation.

// Re-export the main ServiceBridge from service_bridge
pub use crate::service_bridge::ServiceBridge;

// Re-export SharedServiceBridge from systems module (to avoid duplication)
pub use super::systems::SharedServiceBridge;