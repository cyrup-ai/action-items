//! Tests for plugins/service_bridge_integration/permission_mapper.rs

use ecs_service_bridge::components::Capability;
use action_items_core::plugins::service_bridge_integration::permission_mapper::{
    map_capabilities_to_permissions,
};
use action_items_core::plugins::discovery::DiscoveredPlugin;
use action_items_core::plugins::interface::PermissionType;

#[test]
fn test_clipboard_capability_mapping() {
    let capabilities = vec![Capability {
        name: "clipboard".to_string(),
        version: "1.0.0".to_string(),
        description: "Clipboard access".to_string(),
        metadata: std::collections::HashMap::new(),
    }];

    // Create a mock plugin for testing
    // Note: This would need actual plugin instances in real tests
    // For now, we test the capability mapping logic directly

    let permissions = map_capabilities_to_permissions(&capabilities, &create_mock_plugin());
    assert!(permissions.is_ok());

    let perms = permissions.unwrap();
    assert!(perms.contains(&PermissionType::FullDiskAccess));
}
    use action_items_native::Error;
    use bevy::tasks::{AsyncComputeTaskPool, Task};
    use serde_json::Value;
    use std::collections::HashMap;

    // Create a minimal mock NativePlugin implementation
    struct MockNativePlugin {
        manifest: PluginManifest,
    }

    impl NativePlugin for MockNativePlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }

        fn initialize(
            &mut self,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }

        fn execute_command(
            &mut self,
            _command_id: String,
            _context: PluginContext,
            _args: Option<Value>,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Option<Value>, Error>> {
            task_pool.spawn(async { Ok(None) })
        }


#[test]
fn test_camera_capability_mapping() {
    let capabilities = vec![Capability {
        name: "camera".to_string(),
        version: "1.0.0".to_string(),
        description: "Camera access".to_string(),
        metadata: std::collections::HashMap::new(),
    }];

    let permissions = map_capabilities_to_permissions(&capabilities, &create_mock_plugin());
    assert!(permissions.is_ok());

    let perms = permissions.unwrap();
    assert!(perms.contains(&PermissionType::Camera));
}

#[test]
fn test_multiple_capabilities_mapping() {
    let capabilities = vec![
        Capability {
            name: "camera".to_string(),
            version: "1.0.0".to_string(),
            description: "Camera access".to_string(),
            metadata: std::collections::HashMap::new(),
        },
        Capability {
            name: "microphone".to_string(),
            version: "1.0.0".to_string(),
            description: "Microphone access".to_string(),
            metadata: std::collections::HashMap::new(),
        },
    ];

        fn search(
            &self,
            _query: String,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Vec<ActionItem>, Error>> {
            task_pool.spawn(async { Ok(vec![]) })
        }

        fn execute_action(
            &mut self,
            _action_id: String,
            _context: PluginContext,
            _args: Option<Value>,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Option<Value>, Error>> {
            task_pool.spawn(async { Ok(None) })
        }

        fn background_refresh(
            &mut self,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }

        fn cleanup(
            &mut self,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }

    let permissions = map_capabilities_to_permissions(&capabilities, &create_mock_plugin());
    assert!(permissions.is_ok());

    let perms = permissions.unwrap();
    assert!(perms.contains(&PermissionType::Camera));
    assert!(perms.contains(&PermissionType::Microphone));
}

#[test]
fn test_unknown_capability_handling() {
    let capabilities = vec![Capability {
        name: "unknown_capability".to_string(),
        version: "1.0.0".to_string(),
        description: "Unknown capability".to_string(),
        metadata: std::collections::HashMap::new(),
    }];

    let permissions = map_capabilities_to_permissions(&capabilities, &create_mock_plugin());
    assert!(permissions.is_ok());

    // Should not fail on unknown capabilities, just log and continue
    let perms = permissions.unwrap();
    // Should still have plugin-type specific permissions
    assert!(perms.contains(&PermissionType::FullDiskAccess));
}

// Helper function to create a mock plugin for testing
fn create_mock_plugin() -> DiscoveredPlugin {
    use action_items_core::native_plugin_wrapper::NativePluginWrapper;
    }

    // Create a minimal mock manifest for testing
    let mock_manifest = PluginManifest {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Mock plugin for permission mapping tests".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        homepage: None,
        repository: None,
        icon: None,
        categories: vec![PluginCategory::Utilities],
        keywords: vec![],
        capabilities: PluginCapabilities::default(),
        permissions: PluginPermissions::default(),
        configuration: vec![],
        preferences: vec![],
        commands: vec![],
        actions: vec![],
        dependencies: HashMap::new(),
        environment: HashMap::new(),
        min_launcher_version: "0.1.0".to_string(),
        max_launcher_version: None,
        update_url: None,
        changelog_url: None,
    };

    let mock_plugin = MockNativePlugin {
        manifest: mock_manifest,
    };

    // Create real wrapper using the proper constructor
    let wrapper = NativePluginWrapper::new(Box::new(mock_plugin))
        .expect("Failed to create mock native plugin wrapper");
    
    DiscoveredPlugin::Native(wrapper)
}
    use action_items_core::plugins::interface::{NativePlugin, PluginContext};
    use action_items_common::plugin_interface::{
        PluginManifest, PluginCapabilities, PluginPermissions, ActionItem, PluginCategory
    };
    use action_items_native::Error;
    use bevy::tasks::{AsyncComputeTaskPool, Task};
    use serde_json::Value;
    use std::collections::HashMap;

    // Create a minimal mock NativePlugin implementation
    struct MockNativePlugin {
        manifest: PluginManifest,
    }

    impl NativePlugin for MockNativePlugin {
        fn manifest(&self) -> &PluginManifest {
            &self.manifest
        }

        fn initialize(
            &mut self,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }

        fn execute_command(
            &mut self,
            _command_id: String,
            _context: PluginContext,
            _args: Option<Value>,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Option<Value>, Error>> {
            task_pool.spawn(async { Ok(None) })
        }

        fn search(
            &self,
            _query: String,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Vec<ActionItem>, Error>> {
            task_pool.spawn(async { Ok(vec![]) })
        }

        fn execute_action(
            &mut self,
            _action_id: String,
            _context: PluginContext,
            _args: Option<Value>,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<Option<Value>, Error>> {
            task_pool.spawn(async { Ok(None) })
        }

        fn background_refresh(
            &mut self,
            _context: PluginContext,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }

        fn cleanup(
            &mut self,
            task_pool: &AsyncComputeTaskPool,
        ) -> Task<Result<(), Error>> {
            task_pool.spawn(async { Ok(()) })
        }
    }

    // Create a minimal mock manifest for testing
    let mock_manifest = PluginManifest {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "Mock plugin for permission mapping tests".to_string(),
        author: "Test".to_string(),
        license: "MIT".to_string(),
        homepage: None,
        repository: None,
        icon: None,
        categories: vec![PluginCategory::Utilities],
        keywords: vec![],
        capabilities: PluginCapabilities::default(),
        permissions: PluginPermissions::default(),
        configuration: vec![],
        preferences: vec![],
        commands: vec![],
        actions: vec![],
        dependencies: HashMap::new(),
        environment: HashMap::new(),
        min_launcher_version: "0.1.0".to_string(),
        max_launcher_version: None,
        update_url: None,
        changelog_url: None,
    };

    let mock_plugin = MockNativePlugin {
        manifest: mock_manifest,
    };

    // Create real wrapper using the proper constructor
    let wrapper = NativePluginWrapper::new(Box::new(mock_plugin))
        .expect("Failed to create mock native plugin wrapper");
    
    DiscoveredPlugin::Native(wrapper)
}
