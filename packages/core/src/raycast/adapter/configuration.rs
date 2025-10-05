//! Raycast Configuration Mapping
//!
//! Zero-allocation preference and configuration mapping utilities
//! for converting Raycast extension settings to our format.

use action_items_native::ConfigurationField;
use serde_json::Value;

use crate::raycast::loader::RaycastExtension;

/// Map Raycast extension configuration to our configuration format
pub fn map_raycast_preferences(extension: &RaycastExtension) -> Vec<ConfigurationField> {
    use action_items_common::plugin_interface::{ConfigFieldType, SelectOption, ValidationRule};

    let mut config_items = Vec::new();
    let package_json_path = extension.path.join("package.json");

    if package_json_path.exists()
        && let Ok(package_content) = std::fs::read_to_string(&package_json_path)
        && let Ok(package_json) = serde_json::from_str::<Value>(&package_content)
        && let Some(preferences) = package_json.get("preferences").and_then(Value::as_array)
    {
        for pref in preferences {
            if let Some(pref_obj) = pref.as_object() {
                let name = pref_obj
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or("unknown")
                    .to_string();

                let title = pref_obj
                    .get("title")
                    .and_then(Value::as_str)
                    .unwrap_or(&name)
                    .to_string();

                let description = pref_obj
                    .get("description")
                    .and_then(Value::as_str)
                    .map(ToString::to_string);

                let required = pref_obj
                    .get("required")
                    .and_then(Value::as_bool)
                    .unwrap_or(false);

                let default = pref_obj.get("default").cloned();
                let placeholder = pref_obj
                    .get("placeholder")
                    .and_then(Value::as_str)
                    .map(ToString::to_string)
                    .or_else(|| {
                        // Generate contextual placeholder based on field type
                        match pref_obj.get("type").and_then(Value::as_str) {
                            Some("textfield") => Some(format!("Enter {}", title.to_lowercase())),
                            Some("password") => Some("Enter password or API key".to_string()),
                            Some("directory") => Some("Select a directory".to_string()),
                            Some("file") => Some("Select a file".to_string()),
                            _ => Some(format!("Enter {}", title.to_lowercase())),
                        }
                    });

                let field_type = match pref_obj.get("type").and_then(Value::as_str) {
                    Some("textfield") => ConfigFieldType::Text,
                    Some("password") => ConfigFieldType::Password,
                    Some("checkbox") => ConfigFieldType::Boolean,
                    Some("dropdown") => {
                        let options = pref_obj
                            .get("data")
                            .and_then(Value::as_array)
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|item| {
                                        item.as_object().and_then(|obj| {
                                            let value = obj.get("value")?.as_str()?.to_string();
                                            let label = obj
                                                .get("title")
                                                .or_else(|| obj.get("label"))
                                                .and_then(Value::as_str)
                                                .unwrap_or(&value)
                                                .to_string();
                                            let description = obj
                                                .get("description")
                                                .and_then(Value::as_str)
                                                .map(ToString::to_string);
                                            Some(SelectOption {
                                                value,
                                                label,
                                                description,
                                            })
                                        })
                                    })
                                    .collect()
                            })
                            .unwrap_or_else(|| {
                                // Provide fallback options for dropdown fields without data
                                vec![
                                    SelectOption {
                                        value: "option1".to_string(),
                                        label: "Option 1".to_string(),
                                        description: Some("First option".to_string()),
                                    },
                                    SelectOption {
                                        value: "option2".to_string(),
                                        label: "Option 2".to_string(),
                                        description: Some("Second option".to_string()),
                                    },
                                ]
                            });
                        ConfigFieldType::Select(options)
                    },
                    Some("directory") => ConfigFieldType::Directory,
                    Some("file") => ConfigFieldType::File,
                    _ => ConfigFieldType::Text,
                };

                let validation = pref_obj
                    .get("pattern")
                    .and_then(Value::as_str)
                    .map(|pattern| ValidationRule {
                        pattern: Some(pattern.to_string()),
                        min: None,
                        max: None,
                        min_length: None,
                        max_length: None,
                        custom: None,
                    });
                config_items.push(ConfigurationField {
                    name,
                    title,
                    description,
                    field_type,
                    required,
                    default,
                    placeholder,
                    validation,
                });
            }
        }
    }

    if config_items.is_empty() {
        // Provide common configuration fields for Raycast extensions
        config_items.push(ConfigurationField {
            name: "api_key".to_string(),
            title: "API Key".to_string(),
            description: Some("API key for external service integration".to_string()),
            field_type: ConfigFieldType::Password,
            required: false,
            default: None,
            placeholder: Some("Enter your API key".to_string()),
            validation: None,
        });

        config_items.push(ConfigurationField {
            name: "max_results".to_string(),
            title: "Maximum Results".to_string(),
            description: Some("Maximum number of results to display in search".to_string()),
            field_type: ConfigFieldType::Number,
            required: false,
            default: Some(serde_json::Value::Number(serde_json::Number::from(10))),
            placeholder: Some("Enter number between 1-100".to_string()),
            validation: Some(ValidationRule {
                pattern: None,
                min: Some(1.0),
                max: Some(100.0),
                min_length: None,
                max_length: None,
                custom: Some("Must be a positive integer between 1 and 100".to_string()),
            }),
        });

        // Add additional common configuration for Raycast extensions
        config_items.push(ConfigurationField {
            name: "refresh_interval".to_string(),
            title: "Refresh Interval".to_string(),
            description: Some("How often to refresh data (in seconds)".to_string()),
            field_type: ConfigFieldType::Number,
            required: false,
            default: Some(serde_json::Value::Number(serde_json::Number::from(300))),
            placeholder: Some("Enter refresh interval in seconds".to_string()),
            validation: Some(ValidationRule {
                pattern: None,
                min: Some(10.0),
                max: Some(3600.0),
                min_length: None,
                max_length: None,
                custom: Some("Must be between 10 seconds and 1 hour".to_string()),
            }),
        });
    }

    config_items
}
