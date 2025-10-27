//! Type definitions and validation for user settings
//!
//! This module provides table name validation and RecordId construction
//! to prevent SQL injection attacks. All table names must be in the VALID_TABLES
//! whitelist, and record IDs are constructed using SurrealDB's RecordId type.

use surrealdb::RecordId;
use crate::error::SettingsError;

/// Valid table names for settings storage
///
/// This whitelist prevents SQL injection by ensuring only known tables
/// can be accessed through the settings API.
pub const VALID_TABLES: &[&str] = &[
    "user_preferences",
    "hotkey_settings",
    "plugin_configs",
    "ui_state",
    "ai_settings",
    "cloud_sync_settings",
    "account_settings",
    "organization_settings",
    "advanced_settings",
    "appearance_settings",
    "startup_settings",
    "settings_history",
];

/// Validate that a table name is in the whitelist
///
/// # Arguments
/// * `table` - The table name to validate
///
/// # Returns
/// * `Ok(())` if the table is valid
/// * `Err(SettingsError::InvalidValue)` if the table is not whitelisted
///
/// # Performance
/// Linear search is used as the table count is small (12 items).
/// This function is inlined for hot path optimization.
#[inline]
pub fn validate_table_name(table: &str) -> Result<(), SettingsError> {
    if VALID_TABLES.contains(&table) {
        Ok(())
    } else {
        Err(SettingsError::InvalidValue(format!(
            "Invalid table name: '{}'. Must be one of: {}",
            table,
            VALID_TABLES.join(", ")
        )))
    }
}

/// Parse table and key into a RecordId for type-safe database operations
///
/// This function validates the table name and constructs a RecordId,
/// preventing SQL injection by using SurrealDB's type-safe record addressing.
///
/// # Arguments
/// * `table` - The table name (must be in VALID_TABLES)
/// * `key` - The record key/ID
///
/// # Returns
/// * `Ok(RecordId)` - Type-safe record identifier
/// * `Err(SettingsError::InvalidValue)` - If table is not whitelisted
///
/// # Example
/// ```ignore
/// let record_id = parse_record_id("user_preferences", "theme")?;
/// // record_id can now be safely used with db.select(), db.update(), etc.
/// ```
#[inline]
pub fn parse_record_id(table: &str, key: &str) -> Result<RecordId, SettingsError> {
    // Validate table name first
    validate_table_name(table)?;
    
    // Construct RecordId using SurrealDB's safe parsing
    // RecordId::from((table, key)) creates a valid table:key identifier
    Ok(RecordId::from((table, key)))
}
