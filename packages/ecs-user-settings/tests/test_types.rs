//! Tests for types.rs

use action_items_ecs_user_settings::types::{validate_table_name, parse_record_id, VALID_TABLES};

#[test]
fn test_all_valid_tables_accepted() {
    for table in VALID_TABLES {
        assert!(validate_table_name(table).is_ok());
    }
}

#[test]
fn test_invalid_table_rejected() {
    assert!(validate_table_name("invalid_table").is_err());
    assert!(validate_table_name("users; DROP TABLE").is_err());
}

#[test]
fn test_record_id_construction() {
    let result = parse_record_id("user_preferences", "main");
    assert!(result.is_ok());
    
    let record_id = result.expect("should parse");
    assert_eq!(record_id.to_string(), "user_preferences:main");
}

#[test]
fn test_sql_injection_prevention() {
    // Table injection should be blocked by whitelist
    assert!(parse_record_id("users; DROP TABLE", "test").is_err());
    
    // Key injection is handled by RecordId type safety
    // RecordId wraps dangerous keys in ⟨⟩ brackets to escape them
    let result = parse_record_id("user_preferences", "'; DELETE FROM users--");
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
