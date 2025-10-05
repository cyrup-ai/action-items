//! Comprehensive test suite for user settings service
//!
//! Tests cover:
//! - Input validation and SQL injection prevention
//! - CRUD operations (read, write, update, delete)
//! - Audit trail and change tracking
//! - Migration from JSON files
//! - Error handling and edge cases

#[cfg(test)]
mod validation_tests {
    use crate::types::{validate_table_name, parse_record_id, VALID_TABLES};
    use crate::error::SettingsError;

    #[test]
    fn test_all_valid_tables_accepted() {
        for table in VALID_TABLES {
            assert!(
                validate_table_name(table).is_ok(),
                "Valid table {} should be accepted",
                table
            );
        }
    }

    #[test]
    fn test_invalid_table_rejected() {
        let invalid_tables = vec![
            "invalid_table",
            "users; DROP TABLE",
            "DROP DATABASE users",
            "../../../etc/passwd",
            "'; DELETE FROM users--",
            "admin' OR '1'='1",
        ];

        for table in invalid_tables {
            assert!(
                validate_table_name(table).is_err(),
                "Invalid table '{}' should be rejected",
                table
            );
        }
    }

    #[test]
    fn test_record_id_construction() {
        let result = parse_record_id("user_preferences", "main");
        assert!(result.is_ok(), "Should parse valid table and key");
        
        let record_id = result.expect("should parse");
        assert_eq!(
            record_id.to_string(),
            "user_preferences:main",
            "RecordId should format correctly"
        );
    }

    #[test]
    fn test_sql_injection_prevention_table() {
        // Table injection should be blocked by whitelist
        let injection_attempts = vec![
            ("users; DROP TABLE settings", "test"),
            ("admin' OR '1'='1", "test"),
            ("'; DELETE FROM users--", "test"),
            ("../../../etc/passwd", "config"),
        ];

        for (table, key) in injection_attempts {
            let result = parse_record_id(table, key);
            assert!(
                result.is_err(),
                "SQL injection attempt with table '{}' should be rejected",
                table
            );

            if let Err(e) = result {
                match e {
                    SettingsError::InvalidValue(_) => {
                        // Expected - table not in whitelist
                    },
                    _ => panic!("Expected InvalidValue error, got {:?}", e),
                }
            }
        }
    }

    #[test]
    fn test_sql_injection_prevention_key() {
        // Key injection is handled by RecordId type safety
        // RecordId wraps dangerous keys in ⟨⟩ brackets to escape them
        let injection_keys = vec![
            "'; DELETE FROM users--",
            "admin' OR '1'='1",
            "test'; DROP TABLE settings; --",
        ];

        for key in injection_keys {
            let result = parse_record_id("user_preferences", key);
            assert!(
                result.is_ok(),
                "RecordId should accept key '{}' (escaping is handled internally)",
                key
            );

            if let Ok(rid) = result {
                let id_str = rid.to_string();
                // RecordId escapes special chars - check for escape markers ⟨⟩
                assert!(
                    id_str.contains("⟨") && id_str.contains("⟩"),
                    "RecordId should escape dangerous keys with ⟨⟩ brackets, got: {}",
                    id_str
                );
            }
        }
    }

    #[test]
    fn test_case_sensitivity() {
        // Table names must match exactly (case-sensitive)
        assert!(parse_record_id("user_preferences", "test").is_ok());
        assert!(parse_record_id("User_Preferences", "test").is_err());
        assert!(parse_record_id("USER_PREFERENCES", "test").is_err());
    }
}

// Integration tests are disabled pending DatabaseService test API
// These tests require a running SurrealDB instance and proper test harness setup
#[cfg(all(test, feature = "integration-tests"))]
mod integration_tests {
    use bevy::prelude::*;
    use uuid::Uuid;
    use surrealdb::Value;
    use std::collections::HashMap;
    use action_items_ecs_surrealdb::DatabaseService;
    
    use crate::events::*;
    use crate::plugin::UserSettingsPlugin;
    use crate::schema::USER_SETTINGS_SCHEMA;

    /// Helper to create test app with database
    async fn create_test_app() -> App {
        let mut app = App::new();
        
        // Create in-memory database for testing
        let db = DatabaseService::new_in_memory()
            .await
            .expect("Failed to create in-memory database");
        
        // Initialize schema
        db.execute_schema(USER_SETTINGS_SCHEMA)
            .await
            .expect("Failed to initialize schema");
        
        app.insert_resource(db);
        app.add_plugins(UserSettingsPlugin);
        
        app
    }

    /// Helper to create test entity
    fn create_test_requester(app: &mut App) -> Entity {
        app.world_mut().spawn_empty().id()
    }

    #[tokio::test]
    async fn test_read_operation() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        // First write a value
        let operation_id = Uuid::new_v4();
        let test_value = serde_json::json!({"theme": "dark", "language": "en"});
        
        app.world_mut().send_event(SettingsWriteRequested {
            operation_id,
            table: "user_preferences".to_string(),
            key: "test_user".to_string(),
            value: Value::from(test_value.clone()),
            requester,
        });

        // Run systems to process write
        app.update();
        app.update();

        // Now read it back
        let read_id = Uuid::new_v4();
        app.world_mut().send_event(SettingsReadRequested {
            operation_id: read_id,
            table: "user_preferences".to_string(),
            key: "test_user".to_string(),
            requester,
        });

        // Run systems to process read
        app.update();
        app.update();

        // Check for completion event
        let mut completed_events = app.world_mut()
            .resource_mut::<Events<SettingsReadCompleted>>()
            .expect("SettingsReadCompleted events should exist");
        
        let events: Vec<_> = completed_events.drain().collect();
        assert_eq!(events.len(), 1, "Should have one read completion event");

        let event = &events[0];
        assert_eq!(event.operation_id, read_id);
        assert!(event.result.is_ok(), "Read should succeed");
        
        if let Ok(Some(value)) = &event.result {
            // Verify the value matches what we wrote
            assert_eq!(value, &Value::from(test_value));
        } else {
            panic!("Expected Some(value) from read");
        }
    }

    #[tokio::test]
    async fn test_write_operation() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        let operation_id = Uuid::new_v4();
        let test_value = serde_json::json!({
            "theme": "dark",
            "font_size": 14,
            "auto_save": true
        });
        
        app.world_mut().send_event(SettingsWriteRequested {
            operation_id,
            table: "user_preferences".to_string(),
            key: "write_test".to_string(),
            value: Value::from(test_value.clone()),
            requester,
        });

        // Run systems
        app.update();
        app.update();

        // Check completion event
        let mut completed_events = app.world_mut()
            .resource_mut::<Events<SettingsWriteCompleted>>()
            .expect("SettingsWriteCompleted events should exist");
        
        let events: Vec<_> = completed_events.drain().collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].result.is_ok(), "Write should succeed");

        // Verify SettingChanged event was emitted
        let mut change_events = app.world_mut()
            .resource_mut::<Events<SettingChanged>>()
            .expect("SettingChanged events should exist");
        
        let changes: Vec<_> = change_events.drain().collect();
        assert_eq!(changes.len(), 1, "Should emit one SettingChanged event");
        assert_eq!(changes[0].table, "user_preferences");
        assert_eq!(changes[0].key, "write_test");
    }

    #[tokio::test]
    async fn test_update_operation() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        // First write initial value
        let write_id = Uuid::new_v4();
        let initial_value = serde_json::json!({
            "theme": "light",
            "font_size": 12,
        });
        
        app.world_mut().send_event(SettingsWriteRequested {
            operation_id: write_id,
            table: "user_preferences".to_string(),
            key: "update_test".to_string(),
            value: Value::from(initial_value),
            requester,
        });

        app.update();
        app.update();

        // Now update with partial fields
        let update_id = Uuid::new_v4();
        let mut update_fields = HashMap::new();
        update_fields.insert("theme".to_string(), Value::from("dark"));
        update_fields.insert("font_size".to_string(), Value::from(16));
        
        app.world_mut().send_event(SettingsUpdateRequested {
            operation_id: update_id,
            table: "user_preferences".to_string(),
            key: "update_test".to_string(),
            fields: update_fields,
            requester,
        });

        app.update();
        app.update();

        // Check completion
        let mut completed_events = app.world_mut()
            .resource_mut::<Events<SettingsUpdateCompleted>>()
            .expect("SettingsUpdateCompleted events should exist");
        
        let events: Vec<_> = completed_events.drain().collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].result.is_ok(), "Update should succeed");

        // Verify SettingChanged event has old_value populated (task 2.2)
        let mut change_events = app.world_mut()
            .resource_mut::<Events<SettingChanged>>()
            .expect("SettingChanged events should exist");
        
        let changes: Vec<_> = change_events.drain().collect();
        // Should have 2 events: write + update
        assert!(changes.len() >= 1, "Should have SettingChanged events");
        
        let update_change = changes.last().expect("Should have update change");
        assert!(
            update_change.old_value.is_some(),
            "Update should have old_value for audit trail (task 2.2)"
        );
    }

    #[tokio::test]
    async fn test_delete_operation() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        // First write a value
        let write_id = Uuid::new_v4();
        let test_value = serde_json::json!({"data": "to_delete"});
        
        app.world_mut().send_event(SettingsWriteRequested {
            operation_id: write_id,
            table: "user_preferences".to_string(),
            key: "delete_test".to_string(),
            value: Value::from(test_value),
            requester,
        });

        app.update();
        app.update();

        // Now delete it
        let delete_id = Uuid::new_v4();
        app.world_mut().send_event(SettingsDeleteRequested {
            operation_id: delete_id,
            table: "user_preferences".to_string(),
            key: "delete_test".to_string(),
            requester,
        });

        app.update();
        app.update();

        // Check completion
        let mut completed_events = app.world_mut()
            .resource_mut::<Events<SettingsDeleteCompleted>>()
            .expect("SettingsDeleteCompleted events should exist");
        
        let events: Vec<_> = completed_events.drain().collect();
        assert_eq!(events.len(), 1);
        
        let result = &events[0].result;
        assert!(result.is_ok(), "Delete should succeed");
        assert_eq!(result.as_ref().unwrap(), &true, "Should return true (existed)");

        // Verify SettingChanged event with old_value and Value::default() (task 2.3)
        let mut change_events = app.world_mut()
            .resource_mut::<Events<SettingChanged>>()
            .expect("SettingChanged events should exist");
        
        let changes: Vec<_> = change_events.drain().collect();
        let delete_change = changes.last().expect("Should have delete change");
        
        assert!(
            delete_change.old_value.is_some(),
            "Delete should have old_value for audit"
        );
        assert_eq!(
            delete_change.new_value,
            Value::default(),
            "Delete should have Value::default() (None) as new_value"
        );
    }

    #[tokio::test]
    async fn test_audit_trail_write() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        // Write a value - should trigger audit trail
        let operation_id = Uuid::new_v4();
        let test_value = serde_json::json!({"audit_test": "value"});
        
        app.world_mut().send_event(SettingsWriteRequested {
            operation_id,
            table: "user_preferences".to_string(),
            key: "audit_write".to_string(),
            value: Value::from(test_value),
            requester,
        });

        app.update();
        app.update();

        // Audit trail system should write to settings_history
        // We'll query settings_history to verify
        let query_id = Uuid::new_v4();
        app.world_mut().send_event(SettingsQueryRequested {
            operation_id: query_id,
            query: "SELECT * FROM settings_history WHERE table = 'user_preferences' AND key = 'audit_write'".to_string(),
            params: None,
            requester,
        });

        app.update();
        app.update();

        // Check audit entries exist
        let mut query_events = app.world_mut()
            .resource_mut::<Events<SettingsQueryCompleted>>()
            .expect("SettingsQueryCompleted events should exist");
        
        let events: Vec<_> = query_events.drain().collect();
        assert_eq!(events.len(), 1);
        
        if let Ok(results) = &events[0].result {
            assert!(
                !results.is_empty(),
                "Audit trail should have recorded the write operation"
            );
        } else {
            panic!("Query failed: {:?}", events[0].result);
        }
    }

    #[tokio::test]
    async fn test_invalid_table_rejected() {
        let mut app = create_test_app().await;
        let requester = create_test_requester(&mut app);

        let operation_id = Uuid::new_v4();
        
        app.world_mut().send_event(SettingsReadRequested {
            operation_id,
            table: "invalid_table; DROP TABLE users".to_string(),
            key: "test".to_string(),
            requester,
        });

        app.update();
        app.update();

        // Should get error event
        let mut completed_events = app.world_mut()
            .resource_mut::<Events<SettingsReadCompleted>>()
            .expect("SettingsReadCompleted events should exist");
        
        let events: Vec<_> = completed_events.drain().collect();
        assert_eq!(events.len(), 1);
        assert!(events[0].result.is_err(), "Invalid table should be rejected");
    }
}

// Migration tests are disabled pending DatabaseService test API
#[cfg(all(test, feature = "integration-tests"))]
mod migration_tests {
    use std::path::PathBuf;
    use tempfile::TempDir;
    use std::fs;
    use serde_json::json;
    
    use action_items_ecs_surrealdb::DatabaseService;
    use crate::migration::*;

    #[tokio::test]
    async fn test_hotkey_migration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path();
        
        // Create mock hotkey preferences JSON
        let hotkey_json = json!({
            "global_hotkey": "Cmd+Space",
            "search_hotkey": "Cmd+K"
        });
        
        let hotkey_path = config_dir.join("hotkey-preferences.json");
        fs::write(&hotkey_path, hotkey_json.to_string())
            .expect("Failed to write test hotkey JSON");
        
        // Create in-memory database
        let db = DatabaseService::new_in_memory()
            .await
            .expect("Failed to create test database");
        
        // Run migration
        let result = migrate_hotkey_preferences(&db, config_dir).await;
        assert!(result.is_ok(), "Migration should succeed");
        
        // Verify backup was created
        let backup_path = hotkey_path.with_extension("json.backup");
        assert!(backup_path.exists(), "Backup file should be created");
    }

    #[tokio::test]
    async fn test_plugin_migration() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path();
        let plugins_dir = config_dir.join("plugins");
        fs::create_dir(&plugins_dir).expect("Failed to create plugins dir");
        
        // Create mock plugin configs
        let plugin1_json = json!({"enabled": true, "timeout": 5000});
        let plugin1_path = plugins_dir.join("test_plugin.json");
        fs::write(&plugin1_path, plugin1_json.to_string())
            .expect("Failed to write plugin JSON");
        
        // Create in-memory database
        let db = DatabaseService::new_in_memory()
            .await
            .expect("Failed to create test database");
        
        // Run migration
        let result = migrate_plugin_configs(&db, config_dir).await;
        assert!(result.is_ok(), "Plugin migration should succeed");
        
        // Verify backup was created
        let backup_path = plugin1_path.with_extension("json.backup");
        assert!(backup_path.exists(), "Plugin backup should be created");
    }

    #[tokio::test]
    async fn test_migration_skip_if_no_files() {
        let temp_dir = TempDir::new().expect("Failed to create temp dir");
        let config_dir = temp_dir.path();
        
        let db = DatabaseService::new_in_memory()
            .await
            .expect("Failed to create test database");
        
        // Should succeed even with no files to migrate
        let result = run_migrations(&db, config_dir).await;
        assert!(result.is_ok(), "Migration should succeed with no files");
    }
}
