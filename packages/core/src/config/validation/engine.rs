use std::collections::HashMap;

use action_items_native::ConfigurationField;
use regex::Regex;

use super::types::{ValidationError, ValidationResult};
use crate::config::ConfigValue;

/// Configuration validation engine
pub struct ValidationEngine {
    compiled_patterns: HashMap<String, Regex>,
}

impl ValidationEngine {
    /// Create new validation engine
    pub fn new() -> Self {
        Self {
            compiled_patterns: HashMap::new(),
        }
    }

    /// Validate a single configuration field
    pub fn validate_field(
        &mut self,
        field: &ConfigurationField,
        value: &Option<ConfigValue>,
    ) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Check if required field is present
        if field.required && value.is_none() {
            result.is_valid = false;
            result.errors.push(ValidationError {
                field_name: field.name.clone(),
                error_type: super::types::ValidationErrorType::Required,
                message: format!("Field '{title}' is required", title = field.title),
                current_value: None,
                expected: Some("non-empty value".to_string()),
            });
            return result;
        }

        // If value is None and not required, it's valid
        let Some(value) = value else {
            return result;
        };

        // Validate based on field type
        super::field_validators::validate_field_type(field, value, &mut result);

        // Apply validation rules
        if let Some(validation) = &field.validation {
            super::rule_validators::validate_rules(self, field, value, validation, &mut result);
        }

        // Generate suggestions for invalid values
        if !result.is_valid {
            super::helpers::generate_suggestions(field, value, &mut result);
        }

        result
    }

    /// Validate all configuration fields for a plugin
    pub fn validate_configuration(
        &mut self,
        fields: &[ConfigurationField],
        values: &HashMap<String, ConfigValue>,
    ) -> HashMap<String, ValidationResult> {
        let mut results = HashMap::new();

        for field in fields {
            let value = values.get(&field.name);
            let result = self.validate_field(field, &value.cloned());
            results.insert(field.name.clone(), result);
        }

        results
    }

    /// Check if all validations pass
    pub fn is_configuration_valid(&self, results: &HashMap<String, ValidationResult>) -> bool {
        results.values().all(|result| result.is_valid)
    }

    /// Get all validation errors
    pub fn get_all_errors(
        &self,
        results: &HashMap<String, ValidationResult>,
    ) -> Vec<ValidationError> {
        results
            .values()
            .flat_map(|result| result.errors.iter())
            .cloned()
            .collect()
    }

    /// Get or compile regex pattern
    pub fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex, regex::Error> {
        if !self.compiled_patterns.contains_key(pattern) {
            let regex = Regex::new(pattern)?;
            self.compiled_patterns.insert(pattern.to_string(), regex);
        }
        // Safe: we just inserted the key if it didn't exist
        self.compiled_patterns.get(pattern).ok_or_else(|| {
            regex::Error::Syntax("Internal error: regex cache inconsistency".to_string())
        })
    }
}

impl Default for ValidationEngine {
    fn default() -> Self {
        Self::new()
    }
}
