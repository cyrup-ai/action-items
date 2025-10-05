use crate::config::ConfigValue;

/// Validation result for a configuration field
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub suggestions: Vec<String>,
}

/// Specific validation error with details
#[derive(Debug, Clone)]
pub struct ValidationError {
    pub field_name: String,
    pub error_type: ValidationErrorType,
    pub message: String,
    pub current_value: Option<ConfigValue>,
    pub expected: Option<String>,
}

#[derive(Debug, Clone)]
pub enum ValidationErrorType {
    Required,
    InvalidType,
    OutOfRange,
    PatternMismatch,
    TooShort,
    TooLong,
    InvalidFormat,
    Custom(String),
}
