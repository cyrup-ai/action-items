//! Type definitions for Deno runtime communication
//!
//! Simplified type definitions focusing on configuration only.

use bevy::prelude::Resource;
use serde::{Deserialize, Serialize};

/// Runtime configuration for Deno plugins
#[derive(Debug, Clone, Serialize, Deserialize, Resource)]
pub struct RuntimeConfig {
    pub max_plugins: usize,
    pub timeout_seconds: u64,
    pub enable_network: bool,
    pub enable_filesystem: bool,
    pub memory_limit_mb: Option<u64>,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            max_plugins: 50,
            timeout_seconds: 30,
            enable_network: false,
            enable_filesystem: true,
            memory_limit_mb: Some(512),
        }
    }
}

/// Runtime communication channels (placeholder for future use)
/// Currently unused but maintained for API compatibility
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuntimeChannels {
    // Future: could contain communication channels for plugin-runtime interaction
    _reserved: (),
}
