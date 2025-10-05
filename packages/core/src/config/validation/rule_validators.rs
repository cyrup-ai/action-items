use action_items_common::plugin_interface::{ConfigurationField, ValidationRule};

use super::engine::ValidationEngine;
use super::types::{ValidationError, ValidationErrorType, ValidationResult};
use crate::config::ConfigValue;

/// Validate field rules
pub fn validate_rules(
    engine: &mut ValidationEngine,
    field: &ConfigurationField,
    value: &ConfigValue,
    rules: &ValidationRule,
    result: &mut ValidationResult,
) {
    // Pattern validation
    if let Some(pattern) = &rules.pattern
        && let Some(str_val) = value.as_str()
    {
        let regex = engine.get_or_compile_regex(pattern);
        match regex {
            Ok(regex) => {
                if !regex.is_match(str_val) {
                    result.is_valid = false;
                    result.errors.push(ValidationError {
                        field_name: field.name.clone(),
                        error_type: ValidationErrorType::PatternMismatch,
                        message: format!("Value '{str_val}' doesn't match required pattern"),
                        current_value: Some(value.clone()),
                        expected: Some(format!("pattern: {pattern}")),
                    });
                }
            },
            Err(e) => {
                result
                    .warnings
                    .push(format!("Invalid regex pattern '{pattern}': {e}"));
            },
        }
    }

    // Numeric range validation
    if let Some(num_val) = value.as_f64() {
        if let Some(min) = rules.min
            && num_val < min
        {
            result.is_valid = false;
            result.errors.push(ValidationError {
                field_name: field.name.clone(),
                error_type: ValidationErrorType::OutOfRange,
                message: format!("Value {num_val} is less than minimum {min}"),
                current_value: Some(value.clone()),
                expected: Some(format!("≥ {min}")),
            });
        }

        if let Some(max) = rules.max
            && num_val > max
        {
            result.is_valid = false;
            result.errors.push(ValidationError {
                field_name: field.name.clone(),
                error_type: ValidationErrorType::OutOfRange,
                message: format!("Value {num_val} is greater than maximum {max}"),
                current_value: Some(value.clone()),
                expected: Some(format!("≤ {max}")),
            });
        }
    }

    // String length validation
    if let Some(str_val) = value.as_str() {
        if let Some(min_length) = rules.min_length
            && str_val.len() < min_length
        {
            result.is_valid = false;
            result.errors.push(ValidationError {
                field_name: field.name.clone(),
                error_type: ValidationErrorType::TooShort,
                message: format!(
                    "Value is too short ({} chars), minimum is {min_length}",
                    str_val.len()
                ),
                current_value: Some(value.clone()),
                expected: Some(format!("at least {min_length} characters")),
            });
        }

        if let Some(max_length) = rules.max_length
            && str_val.len() > max_length
        {
            result.is_valid = false;
            result.errors.push(ValidationError {
                field_name: field.name.clone(),
                error_type: ValidationErrorType::TooLong,
                message: format!(
                    "Value is too long ({} chars), maximum is {max_length}",
                    str_val.len()
                ),
                current_value: Some(value.clone()),
                expected: Some(format!("at most {max_length} characters")),
            });
        }
    }

    // Custom validation (would need JavaScript engine)
    if let Some(custom) = &rules.custom {
        result
            .warnings
            .push(format!("Custom validation '{custom}' not yet implemented"));
    }
}
