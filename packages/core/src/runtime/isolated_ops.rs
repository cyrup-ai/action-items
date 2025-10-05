/// Completely isolated Deno async operations module
///
/// This module is intentionally separated from all other crate modules to avoid
/// any possibility of extism::Error type inference bleeding into the #[op2(async)] macros.
/// All error handling uses only deno_core::JsError which implements JsErrorClass.
///
/// NOTE: Operations are currently disabled due to extism::Error JsErrorClass trait bound
/// issues. This module serves as a placeholder for future Deno operations when the trait bound
/// issue is resolved.
use serde::{Deserialize, Serialize};

/// Action Item data structure isolated from main crate types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IsolatedActionItem {
    pub id: String,
    pub title: String,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub priority: String,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}
