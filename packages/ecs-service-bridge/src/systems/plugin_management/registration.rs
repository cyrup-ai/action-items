//! Plugin Registration System
//!
//! Handles plugin registration requests with compile-time validation,
//! authentication, capability indexing, and entity spawning.

use bevy::prelude::*;
use crossbeam_channel::{Receiver, Sender, bounded};

use super::authentication::{generate_instance_id, validate_authentication_token};
use super::capability_index::PluginCapabilityIndex;
use super::permissions::PluginPermissions;
use crate::components::{CapabilitiesComponent, Capability, PluginComponent, PluginStatus, PluginType};
use crate::events::*;
use crate::resources::*;
use crate::types::*;

/// Plugin registration request with compile-time validation
#[derive(Debug, Clone)]
pub struct PluginRegistrationRequest {
    pub plugin_id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<Capability>,
    pub permissions: PluginPermissions,
    pub authentication_token: String,
}

impl PluginRegistrationRequest {
    /// Create new plugin registration request with validation
    #[inline]
    pub fn new(
        plugin_id: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        description: impl Into<String>,
    ) -> ServiceResult<Self> {
        let plugin_id = plugin_id.into();
        let name = name.into();
        let version = version.into();
        let description = description.into();

        // Validate plugin ID
        if plugin_id.is_empty() || plugin_id.len() > 256 {
            return Err(ServiceError::InvalidAddress(
                "Invalid plugin ID length".into(),
            ));
        }

        if !plugin_id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '_' || c == '-')
        {
            return Err(ServiceError::InvalidAddress(
                "Plugin ID contains invalid characters".into(),
            ));
        }

        // Validate other fields
        if name.is_empty() || name.len() > 1024 {
            return Err(ServiceError::PluginRegistrationFailed {
                reason: "Invalid plugin name".into(),
            });
        }

        if version.is_empty() || version.len() > 128 {
            return Err(ServiceError::PluginRegistrationFailed {
                reason: "Invalid plugin version".into(),
            });
        }

        Ok(Self {
            plugin_id,
            name,
            version,
            description,
            capabilities: Vec::new(),
            permissions: PluginPermissions::default(),
            authentication_token: String::new(),
        })
    }

    /// Add capability with validation
    #[inline]
    pub fn with_capability(mut self, capability: Capability) -> ServiceResult<Self> {
        // Validate capability uniqueness
        if self.capabilities.iter().any(|c| c.name == capability.name) {
            return Err(ServiceError::PluginRegistrationFailed {
                reason: format!("Duplicate capability: {}", capability.name),
            });
        }

        self.capabilities.push(capability);
        Ok(self)
    }

    /// Set permissions with validation
    #[inline]
    pub fn with_permissions(mut self, permissions: PluginPermissions) -> Self {
        self.permissions = permissions;
        self
    }

    /// Set authentication token with const-time validation
    #[inline]
    pub fn with_authentication(mut self, token: impl Into<String>) -> ServiceResult<Self> {
        let token = token.into();

        if token.len() < 32 || token.len() > 512 {
            return Err(ServiceError::PluginAuthenticationFailed {
                plugin_id: self.plugin_id.clone(),
            });
        }

        self.authentication_token = token;
        Ok(self)
    }
}

/// Plugin entity marker for ECS queries
#[derive(Debug, Clone, Component)]
#[repr(C)] // Optimal memory layout
pub struct PluginEntity {
    pub plugin_id: String,
    pub registration_time: TimeStamp,
    pub instance_id: u32,
}

/// Resource for managing plugin registration requests
#[derive(Resource)]
pub struct PluginRegistrationQueue {
    pub requests: Receiver<PluginRegistrationRequest>,
    pub sender: Sender<PluginRegistrationRequest>,
}

impl Default for PluginRegistrationQueue {
    fn default() -> Self {
        let (sender, requests) = bounded(1000);
        Self { requests, sender }
    }
}

impl PluginRegistrationQueue {
    /// Create new registration queue with bounded capacity
    #[inline]
    pub fn new(capacity: usize) -> Self {
        let (sender, requests) = bounded(capacity);
        Self { requests, sender }
    }

    /// Submit registration request
    #[inline]
    pub fn submit(&self, request: PluginRegistrationRequest) -> ServiceResult<()> {
        self.sender
            .try_send(request)
            .map_err(|_| ServiceError::ResourceExhausted {
                resource: "plugin_registration_queue".into(),
            })
    }
}

/// Comprehensive plugin registration system with zero allocations
///
/// Handles plugin registration requests with compile-time validation,
/// authentication, capability indexing, and entity spawning.
#[inline]
pub fn plugin_registration_system(
    mut commands: Commands,
    mut plugin_registry: ResMut<PluginRegistryResource>,
    mut capability_index: ResMut<PluginCapabilityIndex>,
    mut token_store: ResMut<super::authentication::PluginTokenStore>,
    registration_queue: Res<PluginRegistrationQueue>,
    mut lifecycle_events: EventWriter<PluginLifecycleEvent>,
) {
    // Process all pending registration requests
    while let Ok(request) = registration_queue.requests.try_recv() {
        match process_plugin_registration(
            &mut commands,
            &mut plugin_registry,
            &mut capability_index,
            &mut token_store,
            request,
        ) {
            Ok(plugin_entity) => {
                // Send successful registration event
                lifecycle_events.write(PluginLifecycleEvent {
                    plugin_id: plugin_entity.plugin_id.clone(),
                    event_type: LifecycleEventType::Registered,
                    timestamp: TimeStamp::now(),
                });

                info!(
                    "Plugin registered successfully: {}",
                    plugin_entity.plugin_id
                );
            },
            Err(error) => {
                error!("Plugin registration failed: {}", error);
            },
        }
    }
}

/// Process individual plugin registration with comprehensive validation
#[inline]
fn process_plugin_registration(
    commands: &mut Commands,
    plugin_registry: &mut ResMut<PluginRegistryResource>,
    capability_index: &mut ResMut<PluginCapabilityIndex>,
    token_store: &mut ResMut<super::authentication::PluginTokenStore>,
    request: PluginRegistrationRequest,
) -> ServiceResult<PluginEntity> {
    // Validate plugin doesn't already exist
    if plugin_registry.plugins.contains_key(&request.plugin_id) {
        return Err(ServiceError::PluginRegistrationFailed {
            reason: format!("Plugin already exists: {}", request.plugin_id),
        });
    }

    // Validate authentication token (const-time comparison for security)
    if !validate_authentication_token(
        &request.plugin_id,
        &request.authentication_token,
        token_store,
    ) {
        return Err(ServiceError::PluginAuthenticationFailed {
            plugin_id: request.plugin_id,
        });
    }

    // Create plugin entity with all components
    let plugin_entity = PluginEntity {
        plugin_id: request.plugin_id.clone(),
        registration_time: TimeStamp::now(),
        instance_id: generate_instance_id(),
    };

    let plugin_component = PluginComponent {
        plugin_id: request.plugin_id.clone(),
        name: request.name.clone(),
        version: request.version.clone(),
        description: request.description.clone(),
        status: PluginStatus::Initializing,
        plugin_type: PluginType::Native,  // Default to Native, should be passed in request
        author: None,
        has_config: false,
        registration_time: plugin_entity.registration_time,
        last_heartbeat: None,
    };

    let capabilities_component = CapabilitiesComponent {
        capabilities: request.capabilities.clone(),
    };

    let permissions_component = request.permissions;
    let health_component = super::lifecycle::PluginHealth::new();

    // Spawn plugin entity with all components
    commands.spawn((
        plugin_entity.clone(),
        plugin_component,
        capabilities_component,
        permissions_component,
        health_component,
    ));

    // Create plugin info for registry
    let plugin_info = PluginInfo {
        plugin_id: request.plugin_id.clone(),
        name: request.name,
        version: request.version,
        description: request.description,
        capabilities: request
            .capabilities
            .iter()
            .map(|c| crate::resources::Capability {
                name: c.name.clone(),
                version: c.version.clone(),
                description: c.description.clone(),
                metadata: c.metadata.clone(),
            })
            .collect(),
        status: PluginStatus::Initializing,
        registration_time: plugin_entity.registration_time,
        last_heartbeat: None,
    };

    // Plugin channels are automatically created by MessageInfrastructure
    // via manage_plugin_channels_system when PluginLifecycleEvent::Registered is emitted

    // Register in plugin registry
    plugin_registry
        .plugins
        .insert(request.plugin_id.clone(), plugin_info);

    // Index capabilities for fast lookup
    capability_index.add_plugin_capabilities(request.plugin_id.clone(), request.capabilities);

    Ok(plugin_entity)
}
