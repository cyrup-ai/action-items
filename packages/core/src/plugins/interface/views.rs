// View types are defined in this module

use serde::{Deserialize, Serialize};

use super::config::SelectOption;
use super::action_item::{ItemAction, ActionItem};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandResult {
    None,
    List(Vec<ActionItem>),
    Detail(DetailView),
    Form(FormView),
    Custom(serde_json::Value),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetailView {
    pub markdown: String,
    pub metadata: Option<serde_json::Value>,
    pub actions: Vec<ItemAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormView {
    pub title: String,
    pub fields: Vec<FormField>,
    pub submit_action: ItemAction,
    pub cancel_action: Option<ItemAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormField {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub field_type: FormFieldType,
    pub required: bool,
    pub default: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FormFieldType {
    TextField {
        placeholder: Option<String>,
        multiline: bool,
    },
    PasswordField {
        placeholder: Option<String>,
    },
    Dropdown {
        options: Vec<SelectOption>,
    },
    Checkbox,
    DatePicker,
    ColorPicker,
}
