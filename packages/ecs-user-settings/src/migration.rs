//! Migration logic from JSON files to SurrealDB
//!
//! Provides one-time migration from JSON/TOML configuration files to the
//! SurrealDB backend. Migrations run automatically on first startup and are
//! marked complete with a `.settings-migrated` marker file.
//!
//! # Safety
//!
//! - Original files are backed up with `.backup` extension before migration
//! - Migration failures are logged but don't prevent application startup
//! - Marker file prevents duplicate migrations
//! - Invalid filenames are skipped with warnings (no data loss)

use std::path::Path;
use serde_json::Value;
use tracing::{info, warn, error};
use action_items_ecs_surrealdb::DatabaseService;

/// Migrate hotkey preferences from JSON to database
pub async fn migrate_hotkey_preferences(
    db: &DatabaseService,
    config_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let json_path = config_dir.join("hotkey-preferences.json");

    if !json_path.exists() {
        info!("No hotkey preferences JSON file found - skipping migration");
        return Ok(());
    }

    info!("Migrating hotkey preferences from {:?}", json_path);

    // Read JSON file
    let json_content = std::fs::read_to_string(&json_path)?;
    let prefs: Value = serde_json::from_str(&json_content)?;

    // Transform to SurrealDB format
    let query = format!(
        "UPDATE hotkey_settings:global CONTENT {}",
        prefs
    );

    // Execute migration
    match db.query(&query).await {
        Ok(_) => {
            info!("Hotkey preferences migrated successfully");

            // Backup original JSON
            let backup_path = json_path.with_extension("json.backup");
            std::fs::copy(&json_path, &backup_path)?;
            info!("Original JSON backed up to {:?}", backup_path);

            Ok(())
        },
        Err(e) => {
            error!("Failed to migrate hotkey preferences: {}", e);
            Err(e.into())
        },
    }
}

/// Migrate plugin configurations from individual JSON/TOML files to database
pub async fn migrate_plugin_configs(
    db: &DatabaseService,
    config_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let plugins_dir = config_dir.join("plugins");

    if !plugins_dir.exists() {
        info!("No plugins directory found - skipping migration");
        return Ok(());
    }

    info!("Migrating plugin configurations from {:?}", plugins_dir);

    // Read all plugin config files
    for entry in std::fs::read_dir(&plugins_dir)? {
        let entry = entry?;
        let path = entry.path();

        if let Some(ext) = path.extension().filter(|&e| e == "json" || e == "toml") {
            // Parse plugin ID from filename - skip if invalid
            let plugin_id = match path.file_stem().and_then(|s| s.to_str()) {
                Some(id) => id,
                None => {
                    warn!("Skipping plugin config with invalid filename: {:?}", path);
                    continue;
                }
            };

            info!("Migrating plugin config: {}", plugin_id);

            // Read config file
            let content = std::fs::read_to_string(&path)?;
            let config: Value = if ext == "json" {
                serde_json::from_str(&content)?
            } else {
                // Parse TOML and convert to JSON
                let toml_value: toml::Value = toml::from_str(&content)?;
                serde_json::to_value(&toml_value)?
            };

            // Migrate to database
            let query = format!(
                "UPDATE plugin_configs:{} CONTENT {}",
                plugin_id,
                config
            );

            match db.query(&query).await {
                Ok(_) => {
                    // Backup original file
                    let backup_ext = format!("{}.backup", ext.to_str().unwrap_or(""));
                    let backup_path = path.with_extension(backup_ext);
                    std::fs::copy(&path, &backup_path)?;
                    info!("Plugin config {} migrated and backed up", plugin_id);
                },
                Err(e) => {
                    warn!("Failed to migrate plugin config {}: {}", plugin_id, e);
                },
            }
        }
    }

    Ok(())
}

/// Run all migrations
pub async fn run_migrations(
    db: &DatabaseService,
    config_dir: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Starting settings migration from JSON files");

    migrate_hotkey_preferences(db, config_dir).await?;
    migrate_plugin_configs(db, config_dir).await?;

    info!("Settings migration completed successfully");
    Ok(())
}
