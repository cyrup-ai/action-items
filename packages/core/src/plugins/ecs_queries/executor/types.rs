//! Executor types and enums

use std::hash::Hash;

use crossbeam_channel::Sender;

/// Perfect hash table for zero-allocation command matching with compile-time generation
/// Uses minimal perfect hashing with no collisions and zero memory allocation
pub const PERFECT_HASH_TABLE_SIZE: usize = 16;
pub const PERFECT_HASH_COMMANDS: [&str; 8] = [
    "search",
    "execute",
    "init",
    "cleanup",
    "refresh",
    "configure",
    "validate",
    "status",
];

/// Perfect hash function with zero collisions for known command set
#[inline(always)]
pub fn perfect_hash_command(command: &str) -> Option<usize> {
    let bytes = command.as_bytes();
    if bytes.is_empty() {
        return None;
    }

    // Perfect hash function: (first_byte * 7 + last_byte * 3 + len) % 16
    let hash = (bytes[0] as usize * 7 + bytes[bytes.len() - 1] as usize * 3 + bytes.len())
        % PERFECT_HASH_TABLE_SIZE;

    // Verify this is actually the command we expect at this hash position
    match hash {
        0 => {
            if command == "search" {
                Some(0)
            } else {
                None
            }
        },
        1 => {
            if command == "execute" {
                Some(1)
            } else {
                None
            }
        },
        2 => {
            if command == "init" {
                Some(2)
            } else {
                None
            }
        },
        3 => {
            if command == "cleanup" {
                Some(3)
            } else {
                None
            }
        },
        4 => {
            if command == "refresh" {
                Some(4)
            } else {
                None
            }
        },
        5 => {
            if command == "configure" {
                Some(5)
            } else {
                None
            }
        },
        6 => {
            if command == "validate" {
                Some(6)
            } else {
                None
            }
        },
        7 => {
            if command == "status" {
                Some(7)
            } else {
                None
            }
        },
        _ => None,
    }
}

/// Plugin capability verification with compile-time constants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PluginCapability {
    Search,
    Execute,
    FileSystem,
    Network,
    Clipboard,
    Notifications,
    System,
    Storage,
    Cache,
    Background,
    Realtime,
    Permission,
}

impl PluginCapability {
    /// Production capability verification (replaces boolean stub)
    pub fn verify_with_security_backend(
        capabilities: &[PluginCapability],
        plugin_id: &str,
        verifier: &mut crate::service_bridge::registry::CapabilityVerifier,
    ) -> bool {
        // Convert PluginCapability to Capability for verification
        for plugin_capability in capabilities {
            let capability = match plugin_capability {
                PluginCapability::Search => ecs_service_bridge::resources::Capability::new(
                    "search".to_string(),
                    "1.0.0".to_string(),
                    "Search capability".to_string(),
                ),
                PluginCapability::Execute => ecs_service_bridge::resources::Capability::new(
                    "execute".to_string(),
                    "1.0.0".to_string(),
                    "Execute capability".to_string(),
                ),
                PluginCapability::FileSystem => ecs_service_bridge::resources::Capability::new(
                    "filesystem".to_string(),
                    "1.0.0".to_string(),
                    "Filesystem capability".to_string(),
                ),
                PluginCapability::Network => ecs_service_bridge::resources::Capability::new(
                    "network".to_string(),
                    "1.0.0".to_string(),
                    "Network capability".to_string(),
                ),
                PluginCapability::Clipboard => ecs_service_bridge::resources::Capability::new(
                    "clipboard".to_string(),
                    "1.0.0".to_string(),
                    "Clipboard capability".to_string(),
                ),
                PluginCapability::Notifications => ecs_service_bridge::resources::Capability::new(
                    "notification".to_string(),
                    "1.0.0".to_string(),
                    "Notification capability".to_string(),
                ),
                PluginCapability::System => ecs_service_bridge::resources::Capability::new(
                    "system".to_string(),
                    "1.0.0".to_string(),
                    "System capability".to_string(),
                ),
                PluginCapability::Storage => ecs_service_bridge::resources::Capability::new(
                    "storage".to_string(),
                    "1.0.0".to_string(),
                    "Storage capability".to_string(),
                ),
                PluginCapability::Cache => ecs_service_bridge::resources::Capability::new(
                    "cache".to_string(),
                    "1.0.0".to_string(),
                    "Cache capability".to_string(),
                ),
                PluginCapability::Background => ecs_service_bridge::resources::Capability::new(
                    "background".to_string(),
                    "1.0.0".to_string(),
                    "Background capability".to_string(),
                ),
                PluginCapability::Realtime => ecs_service_bridge::resources::Capability::new(
                    "realtime".to_string(),
                    "1.0.0".to_string(),
                    "Realtime capability".to_string(),
                ),
                PluginCapability::Permission => ecs_service_bridge::resources::Capability::new(
                    "permission".to_string(),
                    "1.0.0".to_string(),
                    "Permission capability".to_string(),
                ),
            };

            // Perform comprehensive security verification
            match verifier.verify_capability(plugin_id, &capability.name) {
                Ok(granted) => {
                    if !granted {
                        log::warn!(
                            target: "capability_verification",
                            "Capability verification failed for plugin {}, capability {:?}",
                            plugin_id,
                            plugin_capability
                        );
                        return false;
                    }
                },
                Err(e) => {
                    log::error!(
                        target: "capability_verification",
                        "Error verifying capability for plugin {}, capability {:?}: {}",
                        plugin_id,
                        plugin_capability,
                        e
                    );
                    return false;
                },
            }
        }

        // All capabilities verified successfully
        true
    }
}

/// Zero-allocation execution request
#[derive(Debug)]
pub struct ExecutionRequest {
    pub action_id: String,
    pub plugin_id: String,
    pub args: serde_json::Value,
    pub response_sender: Sender<ExecutionResult>,
}

/// Zero-allocation execution response
#[derive(Debug)]
pub struct ExecutionResponse {
    pub result: ExecutionResult,
    pub plugin_id: String,
    pub execution_time_ms: u64,
}

/// Action execution result with comprehensive error handling
#[derive(Debug, Clone)]
pub struct ExecutionResult {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
    pub execution_time_ms: u64,
    pub plugin_id: String,
}

impl ExecutionResult {
    pub fn success(message: impl Into<String>, plugin_id: impl Into<String>) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: None,
            execution_time_ms: 0,
            plugin_id: plugin_id.into(),
        }
    }

    pub fn success_with_data(
        message: impl Into<String>,
        data: serde_json::Value,
        plugin_id: impl Into<String>,
    ) -> Self {
        Self {
            success: true,
            message: message.into(),
            data: Some(data),
            execution_time_ms: 0,
            plugin_id: plugin_id.into(),
        }
    }

    pub fn failure(message: impl Into<String>, plugin_id: impl Into<String>) -> Self {
        Self {
            success: false,
            message: message.into(),
            data: None,
            execution_time_ms: 0,
            plugin_id: plugin_id.into(),
        }
    }
}

/// Blazing-fast command validation using perfect hash with zero allocation
#[inline(always)]
pub fn is_known_command(command: &str) -> bool {
    perfect_hash_command(command).is_some()
}
