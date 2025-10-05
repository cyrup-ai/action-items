use action_items_common::plugin_interface::{ConfigFieldType, ConfigurationField};

use super::types::ValidationResult;
use crate::config::ConfigValue;

/// Check if color is valid
pub fn is_valid_color(color: &str) -> bool {
    // Hex color
    if color.starts_with('#') && color.len() == 7 {
        return color.chars().skip(1).all(|c| c.is_ascii_hexdigit());
    }

    // RGB color
    if color.starts_with("rgb(") && color.ends_with(')') {
        let inner = &color[4..color.len() - 1];
        let parts: Vec<&str> = inner.split(',').collect();
        if parts.len() == 3 {
            return parts.iter().all(|part| part.trim().parse::<u8>().is_ok());
        }
    }

    // Named colors (basic set)
    matches!(
        color.to_lowercase().as_str(),
        "red"
            | "green"
            | "blue"
            | "yellow"
            | "orange"
            | "purple"
            | "pink"
            | "brown"
            | "black"
            | "white"
            | "gray"
            | "grey"
    )
}

/// Check if date format is valid
pub fn is_valid_date_format(field_type: &ConfigFieldType, date_str: &str) -> bool {
    match field_type {
        ConfigFieldType::Date => {
            // YYYY-MM-DD
            chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
        },
        ConfigFieldType::Time => {
            // HH:MM or HH:MM:SS
            chrono::NaiveTime::parse_from_str(date_str, "%H:%M").is_ok()
                || chrono::NaiveTime::parse_from_str(date_str, "%H:%M:%S").is_ok()
        },
        ConfigFieldType::DateTime => {
            // ISO 8601 format
            chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
                || chrono::NaiveDateTime::parse_from_str(date_str, "%Y-%m-%d %H:%M:%S").is_ok()
        },
        _ => false,
    }
}

/// Get expected date format description
pub fn get_expected_date_format(field_type: &ConfigFieldType) -> String {
    match field_type {
        ConfigFieldType::Date => "YYYY-MM-DD".to_string(),
        ConfigFieldType::Time => "HH:MM or HH:MM:SS".to_string(),
        ConfigFieldType::DateTime => {
            "ISO 8601 (YYYY-MM-DDTHH:MM:SSZ) or YYYY-MM-DD HH:MM:SS".to_string()
        },
        _ => "unknown".to_string(),
    }
}

/// Get human-readable type name for value
pub fn get_value_type_name(value: &ConfigValue) -> &'static str {
    match value {
        ConfigValue::Null => "null",
        ConfigValue::Bool(_) => "boolean",
        ConfigValue::Number(_) => "number",
        ConfigValue::String(_) => "string",
        ConfigValue::Array(_) => "array",
        ConfigValue::Object(_) => "object",
    }
}

/// Generate helpful suggestions for invalid values
pub fn generate_suggestions(
    field: &ConfigurationField,
    value: &ConfigValue,
    result: &mut ValidationResult,
) {
    match &field.field_type {
        ConfigFieldType::Boolean => {
            if value.is_string() {
                result
                    .suggestions
                    .push("Try 'true' or 'false' (without quotes)".to_string());
            }
        },
        ConfigFieldType::Number => {
            if let Some(str_val) = value.as_str()
                && str_val.parse::<f64>().is_ok()
            {
                result
                    .suggestions
                    .push("Remove quotes to make this a number".to_string());
            }
        },
        ConfigFieldType::Select(options) => {
            if !options.is_empty() {
                result.suggestions.push(format!(
                    "Available options: {}",
                    options
                        .iter()
                        .take(5)
                        .map(|o| o.label.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        },
        _ => {},
    }

    // Default value suggestions
    if let Some(default) = &field.default {
        result.suggestions.push(format!("Default value: {default}"));
    }
}
