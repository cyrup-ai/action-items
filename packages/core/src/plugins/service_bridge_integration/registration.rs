use action_items_ecs_permissions::PermissionType;
use ecs_service_bridge::components::PluginStatus;
use ecs_service_bridge::systems::plugin_management::permissions::PluginPermissions;
use ecs_service_bridge::systems::plugin_management::registration::{
    PluginRegistrationQueue, PluginRegistrationRequest,
};
use ecs_service_bridge::types::ServiceError;
use log::info;

use super::capabilities::extract_plugin_capabilities;
use super::components::ServiceBridgeRegistration;
use super::permission_mapper::{
    map_capabilities_to_permissions, validate_permissions_for_platform,
};
use crate::discovery::DiscoveredPlugin;

/// Convert ECS permission types to service bridge PluginPermissions format
///
/// This function translates the comprehensive ECS permission system into the format
/// expected by the service bridge infrastructure.
///
/// # Arguments
/// * `ecs_permissions` - Vector of ECS permission types
///
/// # Returns
/// * `Ok(PluginPermissions)` - Converted service bridge permissions
/// * `Err(ServiceError)` - If conversion fails
fn convert_ecs_permissions_to_plugin_permissions(
    ecs_permissions: &[PermissionType],
) -> Result<PluginPermissions, ServiceError> {
    let mut plugin_permissions = PluginPermissions::default();

    for permission in ecs_permissions {
        match permission {
            // File system permissions
            PermissionType::DesktopFolder
            | PermissionType::DocumentsFolder
            | PermissionType::DownloadsFolder => {
                plugin_permissions = plugin_permissions
                    .with_permission(PluginPermissions::FILE_READ)
                    .with_permission(PluginPermissions::FILE_WRITE);
            },

            // Media permissions
            PermissionType::Camera => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("camera".to_string());
            },

            PermissionType::Microphone => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("microphone".to_string());
            },

            PermissionType::ScreenCapture => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("screen_capture".to_string());
            },

            // Location permission
            PermissionType::Location => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("location".to_string());
            },

            // Network permission
            PermissionType::WiFi => {
                plugin_permissions =
                    plugin_permissions.with_permission(PluginPermissions::NETWORK_ACCESS);
            },

            // System integration permissions
            PermissionType::Accessibility | PermissionType::InputMonitoring => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("system_integration".to_string());
            },

            // Full disk access implies comprehensive permissions
            PermissionType::FullDiskAccess => {
                plugin_permissions = plugin_permissions
                    .with_permission(PluginPermissions::FILE_READ)
                    .with_permission(PluginPermissions::FILE_WRITE)
                    .with_permission(PluginPermissions::STORAGE_READ)
                    .with_permission(PluginPermissions::STORAGE_WRITE)
                    .with_extended_permission("system_integration".to_string());
            },

            // Other permissions - map to extended permissions
            PermissionType::Calendar => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("calendar".to_string());
            },
            PermissionType::Contacts => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("contacts".to_string());
            },
            PermissionType::Reminders => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("reminders".to_string());
            },
            PermissionType::Photos | PermissionType::PhotosAdd => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("photos".to_string());
            },
            PermissionType::Bluetooth => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("bluetooth".to_string());
            },
            PermissionType::AppleEvents => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("apple_events".to_string());
            },
            PermissionType::SpeechRecognition => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("speech_recognition".to_string());
            },
            PermissionType::DeveloperTools => {
                plugin_permissions =
                    plugin_permissions.with_extended_permission("developer_tools".to_string());
            },

            // Platform-specific permissions that don't have direct service bridge equivalents
            _ => {
                tracing::debug!(
                    "ECS permission {:?} doesn't have direct service bridge equivalent, mapping \
                     to extended permission",
                    permission
                );
                plugin_permissions = plugin_permissions
                    .with_extended_permission(format!("{:?}", permission).to_lowercase());
            },
        }
    }

    Ok(plugin_permissions)
}

/// Register a plugin with the service bridge using proper registration queue
pub fn register_plugin_with_service_bridge(
    registration_queue: &PluginRegistrationQueue,
    plugin: &DiscoveredPlugin,
) -> Result<ServiceBridgeRegistration, ServiceError> {
    let (plugin_id, plugin_name, plugin_version, plugin_description) = match plugin {
        DiscoveredPlugin::Native(wrapper) => {
            let metadata = wrapper.metadata();
            (
                metadata.id.clone(),
                metadata.name.clone(),
                metadata.manifest.version.clone(),
                metadata.manifest.description.clone(),
            )
        },
        DiscoveredPlugin::Extism(wrapper) => {
            let metadata = wrapper.metadata();
            (
                metadata.id.clone(),
                metadata.name.clone(),
                metadata.version.clone(),
                metadata.description.clone(),
            )
        },
        DiscoveredPlugin::Raycast(wrapper) => {
            let metadata = wrapper.metadata();
            (
                metadata.id.clone(),
                metadata.name.clone(),
                metadata.version.clone(),
                metadata.description.clone(),
            )
        },
        DiscoveredPlugin::Deno(wrapper) => {
            let metadata = wrapper.metadata();
            (
                metadata.id.clone(),
                metadata.name.clone(),
                metadata.manifest.version.clone(),
                metadata.manifest.description.clone(),
            )
        },
    };

    // Extract capabilities from the plugin
    let capabilities = extract_plugin_capabilities(plugin)?;

    // Create proper plugin registration request
    let mut registration_request = PluginRegistrationRequest::new(
        plugin_id.clone(),
        plugin_name,
        plugin_version,
        plugin_description,
    )?;

    // Add capabilities to the registration request
    for capability in &capabilities {
        registration_request = registration_request.with_capability(capability.clone())?;
    }

    // Map plugin capabilities to ECS permission requirements
    let required_permissions =
        map_capabilities_to_permissions(&capabilities, plugin).map_err(|e| {
            ServiceError::Internal {
                reason: format!("Failed to map plugin permissions: {}", e),
            }
        })?;

    // Validate permissions are supported on current platform
    validate_permissions_for_platform(&required_permissions).map_err(|e| {
        ServiceError::Internal {
            reason: format!("Platform validation failed: {}", e),
        }
    })?;

    // Convert ECS permissions to service bridge permissions
    let plugin_permissions = convert_ecs_permissions_to_plugin_permissions(&required_permissions)?;
    registration_request = registration_request.with_permissions(plugin_permissions);

    // Submit the registration request to the queue
    // The service bridge systems will handle channel creation automatically
    registration_queue.submit(registration_request)?;

    info!(
        "Submitted plugin '{}' registration with {} capabilities to service bridge queue",
        plugin_id,
        capabilities.len()
    );

    // Return registration component with plugin_id as channel (real channels are managed
    // internally)
    Ok(ServiceBridgeRegistration {
        plugin_id: plugin_id.clone(),
        channel: plugin_id, // Channel ID is the plugin_id - infrastructure handles the rest
        capabilities,
        status: PluginStatus::Initializing, // Will be updated by lifecycle events
    })
}
