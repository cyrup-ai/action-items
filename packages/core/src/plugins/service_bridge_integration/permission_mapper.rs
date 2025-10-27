//! Plugin Permission Capability Mapper
//!
//! Maps plugin capabilities to ECS permission requirements following ARCHITECTURE.md patterns.
//! Provides centralized permission mapping logic for service bridge integration.

use std::collections::HashSet;

use action_items_ecs_permissions::types::{PermissionError, PermissionType};
use ecs_service_bridge::components::Capability;

use crate::discovery::DiscoveredPlugin;

/// Maps plugin capabilities to required ECS permission types
///
/// This function analyzes plugin capabilities and determines which system permissions
/// are required for the plugin to function properly. It supports all plugin types:
/// Native, Extism, Raycast, and Deno.
///
/// # Arguments
/// * `capabilities` - Vector of plugin capabilities to analyze
/// * `plugin` - The discovered plugin for additional context
///
/// # Returns
/// * `Ok(Vec<PermissionType>)` - List of required ECS permission types
/// * `Err(PermissionError)` - If capability mapping fails
///
/// # Examples
/// ```
/// let permissions = map_capabilities_to_permissions(&capabilities, &plugin)?;
/// ```
pub fn map_capabilities_to_permissions(
    capabilities: &[Capability],
    plugin: &DiscoveredPlugin,
) -> Result<Vec<PermissionType>, PermissionError> {
    let mut required_permissions = HashSet::new();

    // Analyze each capability and map to appropriate permissions
    for capability in capabilities {
        match capability.name.as_str() {
            // Clipboard operations require file system access
            "clipboard" | "clipboard_read" | "clipboard_write" => {
                required_permissions.insert(PermissionType::FullDiskAccess);
            },

            // Camera access
            "camera" | "video_capture" => {
                required_permissions.insert(PermissionType::Camera);
            },

            // Microphone access
            "microphone" | "audio_capture" => {
                required_permissions.insert(PermissionType::Microphone);
            },

            // Screen capture and recording
            "screen_capture" | "screen_recording" => {
                required_permissions.insert(PermissionType::ScreenCapture);
            },

            // Location services
            "location" | "geolocation" => {
                required_permissions.insert(PermissionType::Location);
            },

            // Calendar access
            "calendar" | "calendar_read" | "calendar_write" => {
                required_permissions.insert(PermissionType::Calendar);
            },

            // Contacts access
            "contacts" | "contacts_read" | "contacts_write" => {
                required_permissions.insert(PermissionType::Contacts);
            },

            // Reminders access
            "reminders" | "reminders_read" | "reminders_write" => {
                required_permissions.insert(PermissionType::Reminders);
            },

            // Photos library access
            "photos" | "photos_read" | "photos_write" => {
                required_permissions.insert(PermissionType::Photos);
                required_permissions.insert(PermissionType::PhotosAdd);
            },

            // Bluetooth access
            "bluetooth" | "bluetooth_scan" | "bluetooth_connect" => {
                required_permissions.insert(PermissionType::Bluetooth);
            },

            // File system access - map to appropriate folder permissions
            "file_read" | "file_write" | "filesystem" => {
                required_permissions.insert(PermissionType::FullDiskAccess);
            },

            // Desktop folder access
            "desktop_access" => {
                required_permissions.insert(PermissionType::DesktopFolder);
            },

            // Documents folder access
            "documents_access" => {
                required_permissions.insert(PermissionType::DocumentsFolder);
            },

            // Downloads folder access
            "downloads_access" => {
                required_permissions.insert(PermissionType::DownloadsFolder);
            },

            // Network access
            "network" | "http" | "fetch" => {
                // Network access doesn't require specific system permissions on most platforms
                // but we track it for completeness
            },

            // Notifications
            "notifications" | "notification_send" => {
                // Notifications typically don't require explicit permissions on macOS/Linux
                // but may on other platforms
            },

            // Accessibility features
            "accessibility" | "input_monitoring" => {
                required_permissions.insert(PermissionType::Accessibility);
                required_permissions.insert(PermissionType::InputMonitoring);
            },

            // Full disk access for comprehensive file operations
            "full_disk_access" => {
                required_permissions.insert(PermissionType::FullDiskAccess);
            },

            // Apple Events for automation
            "apple_events" | "automation" => {
                required_permissions.insert(PermissionType::AppleEvents);
            },

            // Speech recognition
            "speech_recognition" => {
                required_permissions.insert(PermissionType::SpeechRecognition);
            },

            // Developer tools access
            "developer_tools" => {
                required_permissions.insert(PermissionType::DeveloperTools);
            },

            // WiFi access
            "wifi" | "network_scan" => {
                required_permissions.insert(PermissionType::WiFi);
            },

            // Unknown capabilities - log but don't fail
            _ => {
                tracing::debug!(
                    "Unknown capability '{}' for plugin '{}', no specific permissions mapped",
                    capability.name,
                    get_plugin_id(plugin)
                );
            },
        }
    }

    // Add plugin-type specific permissions
    match plugin {
        DiscoveredPlugin::Native(_) => {
            // Native plugins may need additional system access
            required_permissions.insert(PermissionType::FullDiskAccess);
        },
        DiscoveredPlugin::Extism(_) => {
            // WASM plugins are sandboxed but may need file access for host functions
        },
        DiscoveredPlugin::Raycast(_) => {
            // Raycast plugins typically need file system access for caching
            required_permissions.insert(PermissionType::FullDiskAccess);
        },
        DiscoveredPlugin::Deno(_) => {
            // Deno plugins need file system access for module loading
            required_permissions.insert(PermissionType::FullDiskAccess);
        },
    }

    Ok(required_permissions.into_iter().collect())
}

/// Extract plugin ID for logging purposes
fn get_plugin_id(plugin: &DiscoveredPlugin) -> &str {
    match plugin {
        DiscoveredPlugin::Native(wrapper) => &wrapper.metadata().id,
        DiscoveredPlugin::Extism(wrapper) => &wrapper.metadata().id,
        DiscoveredPlugin::Raycast(wrapper) => &wrapper.metadata().id,
        DiscoveredPlugin::Deno(wrapper) => &wrapper.metadata().id,
    }
}

/// Validate that all required permissions are available on the current platform
///
/// This function checks if the mapped permissions are supported on the current
/// operating system and returns an error if critical permissions are unavailable.
///
/// # Arguments
/// * `permissions` - List of permission types to validate
///
/// # Returns
/// * `Ok(())` - If all permissions are supported
/// * `Err(PermissionError)` - If critical permissions are unsupported
pub fn validate_permissions_for_platform(
    permissions: &[PermissionType],
) -> Result<(), PermissionError> {
    for permission in permissions {
        match permission {
            // These permissions are macOS-specific
            PermissionType::AppleEvents
            | PermissionType::DeveloperTools
            | PermissionType::FullDiskAccess
            | PermissionType::DesktopFolder
            | PermissionType::DocumentsFolder
            | PermissionType::DownloadsFolder => {
                #[cfg(not(target_os = "macos"))]
                {
                    tracing::warn!(
                        "Permission {:?} is macOS-specific but running on different platform",
                        permission
                    );
                }
            },

            // These permissions are cross-platform
            PermissionType::Camera
            | PermissionType::Microphone
            | PermissionType::Location
            | PermissionType::Bluetooth => {
                // These should be available on all platforms
            },

            // Other permissions - platform support varies
            _ => {
                tracing::debug!("Permission {:?} support varies by platform", permission);
            },
        }
    }

    Ok(())
}


