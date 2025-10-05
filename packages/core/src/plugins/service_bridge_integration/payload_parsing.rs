//! Payload parsing utilities for service bridge message translation
//!
//! Converts JSON string values to proper ECS service enum types using
//! established codebase patterns.

use action_items_ecs_clipboard::ClipboardFormat;
use action_items_ecs_permissions::PermissionType;

#[derive(Debug, thiserror::Error)]
pub enum PayloadParseError {
    #[error("Unsupported clipboard format: {0}")]
    UnsupportedClipboardFormat(String),
    #[error("Unsupported permission type: {0}")]
    UnsupportedPermissionType(String),
    #[error("Invalid clipboard data: {0}")]
    InvalidClipboardData(String),
}

/// Convert JSON format string to ClipboardFormat enum using established pattern
pub fn parse_clipboard_format(format_str: &str) -> Result<ClipboardFormat, PayloadParseError> {
    match format_str.to_lowercase().as_str() {
        "text" => Ok(ClipboardFormat::Text),
        "html" => Ok(ClipboardFormat::Html),
        #[cfg(feature = "image-data")]
        "image" => Ok(ClipboardFormat::Image),
        "files" => Ok(ClipboardFormat::Files),
        unknown => Err(PayloadParseError::UnsupportedClipboardFormat(unknown.to_string())),
    }
}

/// Convert JSON permission string to PermissionType enum
pub fn parse_permission_type(permission_str: &str) -> Result<PermissionType, PayloadParseError> {
    match permission_str.to_lowercase().as_str() {
        "camera" => Ok(PermissionType::Camera),
        "microphone" => Ok(PermissionType::Microphone),
        "accessibility" => Ok(PermissionType::Accessibility),
        "screen_capture" => Ok(PermissionType::ScreenCapture),
        "input_monitoring" => Ok(PermissionType::InputMonitoring),
        "full_disk_access" => Ok(PermissionType::FullDiskAccess),
        "location" => Ok(PermissionType::Location),
        "contacts" => Ok(PermissionType::Contacts),
        "calendar" => Ok(PermissionType::Calendar),
        "reminders" => Ok(PermissionType::Reminders),
        "wifi" => Ok(PermissionType::WiFi),
        // Add more mappings as needed
        unknown => Err(PayloadParseError::UnsupportedPermissionType(unknown.to_string())),
    }
}

/// Validate and sanitize clipboard data
pub fn validate_clipboard_data(data: &str) -> Result<String, PayloadParseError> {
    // Basic validation - prevent oversized data
    if data.len() > 1024 * 1024 { // 1MB limit
        return Err(PayloadParseError::InvalidClipboardData("Data too large".to_string()));
    }

    // Basic sanitization - remove null bytes and control chars except newlines/tabs
    let sanitized: String = data.chars()
        .filter(|&c| c >= ' ' || c == '\n' || c == '\t' || c == '\r')
        .collect();

    Ok(sanitized)
}