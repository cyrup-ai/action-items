//! JSON parsing and package.json processing
//!
//! This module provides comprehensive metadata parsing for Raycast extensions,
//! including package.json processing, command extraction, and preference handling.

use std::path::Path;

use tokio::fs;

use super::indexer::StringInterner;
use super::types::*;
use super::validator::{validate_file_size, validate_string_field};

/// Metadata parser for extension processing
pub struct MetadataParser<'a> {
    pub interner: &'a StringInterner,
}

impl<'a> MetadataParser<'a> {
    /// Create a new metadata parser
    pub fn new(interner: &'a StringInterner) -> Self {
        Self { interner }
    }

    /// Parse extension from package.json content
    pub fn parse_extension(
        &self,
        package_json: &str,
        path: &Path,
    ) -> Result<IsolatedRaycastExtension, RaycastDiscoveryError> {
        // Parse JSON
        let package: serde_json::Value =
            serde_json::from_str(package_json).map_err(|e| create_parsing_error(path, e))?;

        // Extract basic extension information
        let name = package
            .get(fields::NAME)
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                create_invalid_structure_error(
                    &path.to_string_lossy(),
                    fields::NAME,
                    "string",
                    "missing or not a string",
                )
            })?;

        let title = package
            .get(fields::TITLE)
            .and_then(|v| v.as_str())
            .unwrap_or(name);

        let description = package
            .get(fields::DESCRIPTION)
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let author = package
            .get(fields::AUTHOR)
            .and_then(|v| v.as_str())
            .unwrap_or(UNKNOWN_AUTHOR);

        // Extract categories
        let categories = extract_categories(&package);

        // Extract icon
        let icon = package
            .get(fields::ICON)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract version
        let version = package
            .get(fields::VERSION)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        // Extract keywords
        let keywords = extract_keywords(&package);

        // Extract commands
        let commands = extract_commands(&package, self.interner)?;

        // Extract extension preferences
        let preferences = extract_extension_preferences(&package, self.interner)?;

        // Create extension with validated fields
        let extension = IsolatedRaycastExtension {
            id: validate_string_field(name),
            name: validate_string_field(name),
            title: validate_string_field(title),
            description: validate_string_field(description),
            author: self.interner.intern_author(author).into_owned(),
            categories,
            icon,
            path: path.to_string_lossy().into_owned(),
            commands,
            version,
            keywords,
            metadata: std::collections::HashMap::new(),
            preferences,
        };

        Ok(extension)
    }

    /// Parse package.json from file
    pub async fn parse_package_json(
        &self,
        path: &Path,
    ) -> Result<serde_json::Value, RaycastDiscoveryError> {
        // Validate file size first
        let metadata = fs::metadata(path)
            .await
            .map_err(|e| handle_file_read_error(e, path, path.parent().unwrap_or(path)))?;

        validate_file_size(metadata.len(), path)?;

        // Read and parse file
        let content = fs::read_to_string(path)
            .await
            .map_err(|e| handle_file_read_error(e, path, path.parent().unwrap_or(path)))?;

        serde_json::from_str(&content).map_err(|e| create_parsing_error(path, e))
    }
}

/// Extract categories from package.json
fn extract_categories(package: &serde_json::Value) -> Vec<String> {
    match package.get(fields::CATEGORIES).and_then(|v| v.as_array()) {
        Some(categories) => categories
            .iter()
            .filter_map(|v| v.as_str())
            .map(validate_string_field)
            .collect(),
        None => Vec::new(),
    }
}

/// Extract keywords from package.json
fn extract_keywords(package: &serde_json::Value) -> Vec<String> {
    match package.get(fields::KEYWORDS).and_then(|v| v.as_array()) {
        Some(keywords) => keywords
            .iter()
            .filter_map(|v| v.as_str())
            .map(validate_string_field)
            .collect(),
        None => Vec::new(),
    }
}

/// Extract commands from package.json
pub fn extract_commands(
    package: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<IsolatedRaycastCommand>, RaycastDiscoveryError> {
    let commands_array = match package.get(fields::COMMANDS).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut commands = Vec::with_capacity(commands_array.len());

    for cmd in commands_array {
        let name = match cmd.get(fields::NAME).and_then(|v| v.as_str()) {
            Some(n) => validate_string_field(n),
            None => continue, // Skip commands without names
        };

        let title = match cmd.get(fields::TITLE).and_then(|v| v.as_str()) {
            Some(t) => validate_string_field(t),
            None => name.clone(), // Use name as fallback title
        };

        let description = cmd
            .get(fields::DESCRIPTION)
            .and_then(|v| v.as_str())
            .map(validate_string_field);

        let mode = match cmd.get(fields::MODE).and_then(|v| v.as_str()) {
            Some(m) => interner.intern_mode(m).into_owned(),
            None => DEFAULT_MODE.to_string(),
        };

        let subtitle = cmd
            .get("subtitle")
            .and_then(|v| v.as_str())
            .map(validate_string_field);

        let keywords = extract_command_keywords(cmd);
        let arguments = extract_command_arguments(cmd, interner)?;
        let preferences = extract_command_preferences(cmd, interner)?;

        commands.push(IsolatedRaycastCommand {
            name,
            title,
            description,
            mode,
            subtitle,
            keywords,
            arguments,
            preferences,
        });
    }

    commands.shrink_to_fit();
    Ok(commands)
}

/// Extract command keywords
fn extract_command_keywords(cmd: &serde_json::Value) -> Vec<String> {
    match cmd.get(fields::KEYWORDS).and_then(|v| v.as_array()) {
        Some(keywords) => keywords
            .iter()
            .filter_map(|v| v.as_str())
            .map(validate_string_field)
            .collect(),
        None => Vec::new(),
    }
}

/// Extract command arguments with complete field parsing
#[inline]
pub fn extract_command_arguments(
    cmd: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<IsolatedCommandArgument>, RaycastDiscoveryError> {
    let arguments_array = match cmd.get("arguments").and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut arguments = Vec::with_capacity(arguments_array.len());

    for arg in arguments_array {
        let name = match arg.get(fields::NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip arguments without names
        };

        let placeholder = arg
            .get(fields::PLACEHOLDER)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = arg
            .get(fields::REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        let argument_type = match arg.get(fields::TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "text".to_string(), // Default type
        };

        arguments.push(IsolatedCommandArgument {
            name,
            placeholder,
            required,
            argument_type,
        });
    }

    arguments.shrink_to_fit();
    Ok(arguments)
}

/// Extract command preferences with complete field parsing
#[inline]
pub fn extract_command_preferences(
    cmd: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<IsolatedCommandPreference>, RaycastDiscoveryError> {
    let preferences_array = match cmd.get(fields::PREFERENCES).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut preferences = Vec::with_capacity(preferences_array.len());

    for pref in preferences_array {
        let name = match pref.get(fields::NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip preferences without names
        };

        let title = match pref.get(fields::TITLE).and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => name.clone(), // Use name as fallback title
        };

        let description = pref
            .get(fields::DESCRIPTION)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let preference_type = match pref.get(fields::TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "textfield".to_string(), // Default type
        };

        let default_value = pref
            .get(fields::DEFAULT)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = pref
            .get(fields::REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        preferences.push(IsolatedCommandPreference {
            name,
            title,
            description,
            preference_type,
            default_value,
            required,
        });
    }

    preferences.shrink_to_fit();
    Ok(preferences)
}

/// Extract extension-level preferences with complete field parsing
#[inline]
pub fn extract_extension_preferences(
    package: &serde_json::Value,
    interner: &StringInterner,
) -> Result<Vec<IsolatedExtensionPreference>, RaycastDiscoveryError> {
    let preferences_array = match package.get(fields::PREFERENCES).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return Ok(Vec::new()),
    };

    let mut preferences = Vec::with_capacity(preferences_array.len());

    for pref in preferences_array {
        let name = match pref.get(fields::NAME).and_then(|v| v.as_str()) {
            Some(n) => n.to_string(),
            None => continue, // Skip preferences without names
        };

        let title = match pref.get(fields::TITLE).and_then(|v| v.as_str()) {
            Some(t) => t.to_string(),
            None => name.clone(), // Use name as fallback title
        };

        let description = pref
            .get(fields::DESCRIPTION)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let preference_type = match pref.get(fields::TYPE).and_then(|v| v.as_str()) {
            Some(t) => interner.intern_type(t).into_owned(),
            None => "textfield".to_string(), // Default type
        };

        let default_value = pref
            .get(fields::DEFAULT)
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());

        let required = pref
            .get(fields::REQUIRED)
            .and_then(|v| v.as_bool())
            .unwrap_or_default();

        preferences.push(IsolatedExtensionPreference {
            name,
            title,
            description,
            preference_type,
            default_value,
            required,
        });
    }

    preferences.shrink_to_fit();
    Ok(preferences)
}

// Error creation functions

#[cold]
#[inline(never)]
fn create_invalid_structure_error(
    path: &str,
    field: &'static str,
    expected: &'static str,
    found: &str,
) -> RaycastDiscoveryError {
    RaycastDiscoveryError::InvalidStructure {
        path: path.to_string(),
        field: field.to_string(),
        expected: expected.to_string(),
        found: found.to_string(),
    }
}

#[cold]
#[inline(never)]
fn create_parsing_error(file_path: &Path, source: serde_json::Error) -> RaycastDiscoveryError {
    RaycastDiscoveryError::Parsing {
        file_path: file_path.to_string_lossy().into_owned(),
        line: None,   // serde_json doesn't provide line info in newer versions
        column: None, // serde_json doesn't provide column info in newer versions
        message: source.to_string(),
    }
}

#[cold]
#[inline(never)]
fn handle_file_read_error(
    error: std::io::Error,
    package_json_path: &Path,
    extension_path: &Path,
) -> RaycastDiscoveryError {
    match error.kind() {
        std::io::ErrorKind::NotFound => RaycastDiscoveryError::MissingFiles {
            extension_path: extension_path.to_string_lossy().into_owned(),
            missing_files: vec![PACKAGE_JSON.to_string()],
        },
        std::io::ErrorKind::PermissionDenied => RaycastDiscoveryError::PermissionDenied {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
        },
        _ => RaycastDiscoveryError::Filesystem {
            path: package_json_path.to_string_lossy().into_owned(),
            operation: "read_file".to_string(),
            source: error.to_string(),
        },
    }
}

/// Metadata parsing trait for testability
pub trait MetadataParsing {
    fn parse_package_json(
        &self,
        content: &str,
        path: &Path,
    ) -> Result<serde_json::Value, RaycastDiscoveryError>;
    fn extract_extension(
        &self,
        package: &serde_json::Value,
        path: &Path,
    ) -> Result<IsolatedRaycastExtension, RaycastDiscoveryError>;
    fn validate_structure(
        &self,
        extension: &IsolatedRaycastExtension,
    ) -> Result<(), RaycastValidationError>;
}

impl<'a> MetadataParsing for MetadataParser<'a> {
    fn parse_package_json(
        &self,
        content: &str,
        path: &Path,
    ) -> Result<serde_json::Value, RaycastDiscoveryError> {
        serde_json::from_str(content).map_err(|e| create_parsing_error(path, e))
    }

    fn extract_extension(
        &self,
        package: &serde_json::Value,
        path: &Path,
    ) -> Result<IsolatedRaycastExtension, RaycastDiscoveryError> {
        // This would use the package value directly instead of re-parsing
        // For now, we'll use a simplified approach
        let json_str = serde_json::to_string(package).map_err(|e| create_parsing_error(path, e))?;
        self.parse_extension(&json_str, path)
    }

    fn validate_structure(
        &self,
        extension: &IsolatedRaycastExtension,
    ) -> Result<(), RaycastValidationError> {
        if extension.name.is_empty() {
            return Err(RaycastValidationError::MissingRequired {
                field: "name".to_string(),
                context: "extension".to_string(),
            });
        }
        Ok(())
    }
}
