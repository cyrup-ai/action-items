use action_items_common::plugin_interface::{ConfigFieldType, ConfigurationField};

use super::helpers;
use super::types::{ValidationError, ValidationErrorType, ValidationResult};
use crate::config::ConfigValue;

/// Validate field type
pub fn validate_field_type(
    field: &ConfigurationField,
    value: &ConfigValue,
    result: &mut ValidationResult,
) {
    match &field.field_type {
        ConfigFieldType::Text | ConfigFieldType::Password => {
            if !value.is_string() {
                add_type_error(field, value, "string", result);
            }
        },
        ConfigFieldType::Number => {
            if !value.is_number() {
                add_type_error(field, value, "number", result);
            }
        },
        ConfigFieldType::Boolean => {
            if !value.is_boolean() {
                add_type_error(field, value, "boolean", result);
            }
        },
        ConfigFieldType::Select(options) => {
            if let Some(str_val) = value.as_str() {
                if !options.iter().any(|opt| opt.value == str_val) {
                    result.is_valid = false;
                    result.errors.push(ValidationError {
                        field_name: field.name.clone(),
                        error_type: ValidationErrorType::InvalidFormat,
                        message: format!(
                            "Value '{str_val}' is not a valid option for '{}'",
                            field.title
                        ),
                        current_value: Some(value.clone()),
                        expected: Some(format!(
                            "one of: {}",
                            options
                                .iter()
                                .map(|o| o.value.as_str())
                                .collect::<Vec<_>>()
                                .join(", ")
                        )),
                    });
                }
            } else {
                add_type_error(field, value, "string (option value)", result);
            }
        },
        ConfigFieldType::MultiSelect(options) => {
            if let Some(array) = value.as_array() {
                for item in array {
                    if let Some(str_val) = item.as_str()
                        && !options.iter().any(|opt| opt.value == str_val)
                    {
                        result.is_valid = false;
                        result.errors.push(ValidationError {
                            field_name: field.name.clone(),
                            error_type: ValidationErrorType::InvalidFormat,
                            message: format!("Value '{str_val}' is not a valid option"),
                            current_value: Some(item.clone()),
                            expected: Some(format!(
                                "one of: {}",
                                options
                                    .iter()
                                    .map(|o| o.value.as_str())
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            )),
                        });
                    }
                }
            } else {
                add_type_error(field, value, "array of strings", result);
            }
        },
        ConfigFieldType::File | ConfigFieldType::Directory => {
            if !value.is_string() {
                add_type_error(field, value, "string (path)", result);
            } else if let Some(path_str) = value.as_str() {
                // Basic path validation
                if path_str.trim().is_empty() {
                    result.warnings.push("Path is empty".to_string());
                }
                // Could add more sophisticated path validation here
            }
        },
        ConfigFieldType::Color => {
            if let Some(color_str) = value.as_str() {
                if !helpers::is_valid_color(color_str) {
                    result.is_valid = false;
                    result.errors.push(ValidationError {
                        field_name: field.name.clone(),
                        error_type: ValidationErrorType::InvalidFormat,
                        message: format!("'{color_str}' is not a valid color format"),
                        current_value: Some(value.clone()),
                        expected: Some("hex (#RRGGBB), rgb(r,g,b), or named color".to_string()),
                    });
                }
            } else {
                add_type_error(field, value, "string (color)", result);
            }
        },
        ConfigFieldType::Date | ConfigFieldType::Time | ConfigFieldType::DateTime => {
            if !value.is_string() {
                add_type_error(field, value, "string (ISO format)", result);
            } else if let Some(date_str) = value.as_str() {
                // Basic ISO date format validation
                if !helpers::is_valid_date_format(&field.field_type, date_str) {
                    result.is_valid = false;
                    result.errors.push(ValidationError {
                        field_name: field.name.clone(),
                        error_type: ValidationErrorType::InvalidFormat,
                        message: format!("'{date_str}' is not a valid date/time format"),
                        current_value: Some(value.clone()),
                        expected: Some(helpers::get_expected_date_format(&field.field_type)),
                    });
                }
            }
        },
    }
}

/// Add type validation error
fn add_type_error(
    field: &ConfigurationField,
    value: &ConfigValue,
    expected_type: &str,
    result: &mut ValidationResult,
) {
    result.is_valid = false;
    result.errors.push(ValidationError {
        field_name: field.name.clone(),
        error_type: ValidationErrorType::InvalidType,
        message: format!(
            "Field '{}' expects {expected_type} but got {}",
            field.title,
            helpers::get_value_type_name(value)
        ),
        current_value: Some(value.clone()),
        expected: Some(expected_type.to_string()),
    });
}
