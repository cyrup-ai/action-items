use action_items_ecs_deno::raycast_types::{
    IsolatedRaycastCommand, IsolatedRaycastExtension, IsolatedRaycastPlugin,
};

#[test]
fn test_isolated_raycast_extension_serialization() {
    let ext = IsolatedRaycastExtension::default();
    let json = serde_json::to_string(&ext).expect("Failed to serialize");
    let deserialized: IsolatedRaycastExtension =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(ext, deserialized);
}

#[test]
fn test_isolated_raycast_command_serialization() {
    let cmd = IsolatedRaycastCommand::default();
    let json = serde_json::to_string(&cmd).expect("Failed to serialize");
    let deserialized: IsolatedRaycastCommand =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(cmd, deserialized);
}

#[test]
fn test_isolated_raycast_plugin_serialization() {
    let plugin = IsolatedRaycastPlugin::default();
    let json = serde_json::to_string(&plugin).expect("Failed to serialize");
    let deserialized: IsolatedRaycastPlugin =
        serde_json::from_str(&json).expect("Failed to deserialize");
    assert_eq!(plugin, deserialized);
}

#[test]
fn test_extension_validation() {
    let mut ext =
        IsolatedRaycastExtension::new("test".to_string(), "Test".to_string(), "/test".to_string());
    assert!(ext.validate().is_ok());

    // Test invalid extension
    ext.id = String::new();
    assert!(ext.validate().is_err());
}

#[test]
fn test_command_validation() {
    let mut cmd = IsolatedRaycastCommand::new("test".to_string(), "Test".to_string());
    assert!(cmd.validate().is_ok());

    // Test invalid command mode
    cmd.mode = "invalid".to_string();
    assert!(cmd.validate().is_err());
}

#[test]
fn test_capability_constants() {
    use action_items_ecs_deno::raycast_types::action_types::*;
    use action_items_ecs_deno::raycast_types::argument_types::*;
    use action_items_ecs_deno::raycast_types::capabilities::*;
    use action_items_ecs_deno::raycast_types::command_modes::*;
    use action_items_ecs_deno::raycast_types::preference_types::*;

    // Test that constants are accessible and have expected values
    assert_eq!(SEARCH, "search");
    assert_eq!(VIEW, "view");
    assert_eq!(TEXT, "text");
    assert_eq!(TEXTFIELD, "textfield");
    assert_eq!(PRIMARY, "primary");
}
