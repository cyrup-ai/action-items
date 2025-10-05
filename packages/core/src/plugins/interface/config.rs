use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationField {
    pub name: String,
    pub title: String,
    pub description: Option<String>,
    pub field_type: ConfigFieldType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub placeholder: Option<String>,
    pub validation: Option<ValidationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigFieldType {
    Text,
    Password,
    Number,
    Boolean,
    Select(Vec<SelectOption>),
    MultiSelect(Vec<SelectOption>),
    File,
    Directory,
    Color,
    Date,
    Time,
    DateTime,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectOption {
    pub value: String,
    pub label: String,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub pattern: Option<String>,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub custom: Option<String>, // JavaScript expression
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PreferenceField {
    pub key: String,
    pub title: String,
    pub description: Option<String>,
    pub preference_type: PreferenceType,
    pub default: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreferenceType {
    Bool,
    Number { min: Option<f64>, max: Option<f64> },
    String { multiline: bool },
    Enum { options: Vec<String> },
    Hotkey,
    Color,
}
