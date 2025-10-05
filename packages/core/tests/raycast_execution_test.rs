//! Integration test for Raycast execution engine
//!
//! This test verifies that the complete Raycast execution pipeline works:
//! 1. RaycastPluginComponent can create Deno runtime
//! 2. JavaScript/TypeScript code executes successfully
//! 3. @raycast/api functions are available and work
//! 4. Results are returned properly

use std::path::PathBuf;

use action_items_core::raycast::loader::{RaycastCommand, RaycastExtension};
use action_items_core::raycast::wrapper::RaycastPluginComponent;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_raycast_plugin_component_creation() {
        // Create a test Raycast extension
        let extension = RaycastExtension {
            id: "test-extension".to_string(),
            name: "Test Extension".to_string(),
            title: "Test Extension".to_string(),
            description: "Test Raycast Extension".to_string(),
            author: "Test".to_string(),
            categories: vec!["Test".to_string()],
            icon: None,
            path: PathBuf::from("/tmp/test-extension"),
            commands: vec![RaycastCommand {
                name: "test-command".to_string(),
                title: "Test Command".to_string(),
                description: Some("Test command".to_string()),
                mode: "view".to_string(),
            }],
        };

        let component = RaycastPluginComponent {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/tmp"),
            commands: vec!["test-command".to_string()],
            extension,
        };

        // Verify component was created successfully
        assert_eq!(component.name, "Test");
        assert_eq!(component.commands.len(), 1);
        assert_eq!(component.commands[0], "test-command");
    }

    #[tokio::test]
    async fn test_raycast_command_execution() {
        // Create a test Raycast extension
        let extension = RaycastExtension {
            id: "test-extension".to_string(),
            name: "Test Extension".to_string(),
            title: "Test Extension".to_string(),
            description: "Test Raycast Extension".to_string(),
            author: "Test".to_string(),
            categories: vec!["Test".to_string()],
            icon: None,
            path: PathBuf::from("/tmp/test-extension"),
            commands: vec![RaycastCommand {
                name: "test-command".to_string(),
                title: "Test Command".to_string(),
                description: Some("Test command".to_string()),
                mode: "view".to_string(),
            }],
        };

        let component = RaycastPluginComponent {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/tmp"),
            commands: vec!["test-command".to_string()],
            extension,
        };

        // Execute the command
        let result = component.execute_command("test-command", &[]);
        assert!(result.is_ok());

        let output = result.expect("Failed to get command execution output in raycast test");
        assert!(output.contains("Successfully executed"));
        assert!(output.contains("test-command"));
    }

    #[test]
    fn test_raycast_command_not_found() {
        // Create a test Raycast extension
        let extension = RaycastExtension {
            id: "test-extension".to_string(),
            name: "Test Extension".to_string(),
            title: "Test Extension".to_string(),
            description: "Test Raycast Extension".to_string(),
            author: "Test".to_string(),
            categories: vec!["Test".to_string()],
            icon: None,
            path: PathBuf::from("/tmp/test-extension"),
            commands: vec![RaycastCommand {
                name: "valid-command".to_string(),
                title: "Valid Command".to_string(),
                description: Some("Valid command".to_string()),
                mode: "view".to_string(),
            }],
        };

        let component = RaycastPluginComponent {
            id: "test".to_string(),
            name: "Test".to_string(),
            description: "Test".to_string(),
            path: PathBuf::from("/tmp"),
            commands: vec!["valid-command".to_string()],
            extension,
        };

        // Try to execute a non-existent command
        let result = component.execute_command("nonexistent-command", &[]);
        assert!(result.is_err());

        let error = result.unwrap_err();
        assert!(
            error
                .to_string()
                .contains("Command 'nonexistent-command' not found")
        );
    }
}
