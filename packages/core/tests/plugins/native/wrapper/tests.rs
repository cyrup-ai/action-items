//! Test implementations for native plugin wrapper

use std::collections::HashMap;

use action_items_common::plugin_interface::{PluginCapabilities, PluginPermissions};
use action_items_common::plugin_interface::{PluginCategory, PluginManifest};
use bevy::app::App;

use action_items_core::plugins::native::wrapper::types::{NativePluginWrapper, PluginComponent};
use action_items_core::plugins::interface::NativePlugin;
use action_items_core::search::SearchIndex;

// Mock plugin for testing
struct MockPlugin {
    manifest: PluginManifest,
}

impl NativePlugin for MockPlugin {
    fn manifest(&self) -> &PluginManifest {
        &self.manifest
    }

    fn initialize(
        &mut self,
        _context: action_items_native::context::PluginContext,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<std::result::Result<(), action_items_native::error::Error>> {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async { Ok(()) })
    }

    fn execute_command(
        &mut self,
        _command_id: String,
        _context: action_items_native::context::PluginContext,
        _args: Option<serde_json::Value>,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<
        std::result::Result<Option<serde_json::Value>, action_items_native::error::Error>,
    > {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async { Ok(None) })
    }

    fn search(
        &self,
        _query: String,
        _context: action_items_native::context::PluginContext,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<
        std::result::Result<Vec<action_items_common::plugin_interface::ActionItem>, action_items_native::Error>,
    > {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async {
            Ok(vec![])
        })
    }

    fn execute_action(
        &mut self,
        _action_id: String,
        _context: action_items_native::context::PluginContext,
        _args: Option<serde_json::Value>,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<
        std::result::Result<Option<serde_json::Value>, action_items_native::error::Error>,
    > {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async { Ok(None) })
    }

    fn background_refresh(
        &mut self,
        _context: action_items_native::context::PluginContext,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<std::result::Result<(), action_items_native::error::Error>> {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async { Ok(()) })
    }

    fn cleanup(
        &mut self,
        _task_pool: &bevy::tasks::AsyncComputeTaskPool,
    ) -> bevy::tasks::Task<std::result::Result<(), action_items_native::error::Error>> {
        bevy::tasks::AsyncComputeTaskPool::get().spawn(async { Ok(()) })
    }
}

#[test]
fn test_native_plugin_wrapper_creation() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PluginManifest {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        homepage: None,
        repository: None,
        icon: None,
        categories: vec![PluginCategory::Utilities],
        keywords: vec!["test".to_string()],
        capabilities: PluginCapabilities {
            search: true,
            network_access: false,
            file_system_access: false,
            ..Default::default()
        },
        permissions: PluginPermissions {
            network_hosts: vec![],
            read_files: vec![],
            write_files: vec![],
            ..Default::default()
        },
        configuration: vec![],
        preferences: vec![],
        commands: vec![],
        actions: vec![],
        dependencies: HashMap::new(),
        environment: HashMap::new(),
        min_launcher_version: "1.0.0".to_string(),
        max_launcher_version: None,
        update_url: None,
        changelog_url: None,
    };

    let mock_plugin = MockPlugin { manifest };
    let wrapper = NativePluginWrapper::new(Box::new(mock_plugin));

    let wrapper = wrapper?;
    assert_eq!(wrapper.metadata().id, "test-plugin");
    assert_eq!(wrapper.metadata().name, "Test Plugin");
    Ok(())
}

#[test]
fn test_plugin_registration() -> Result<(), Box<dyn std::error::Error>> {
    let manifest = PluginManifest {
        id: "test-plugin".to_string(),
        name: "Test Plugin".to_string(),
        version: "1.0.0".to_string(),
        description: "A test plugin".to_string(),
        author: "Test Author".to_string(),
        license: "MIT".to_string(),
        homepage: None,
        repository: None,
        icon: None,
        categories: vec![PluginCategory::Utilities],
        keywords: vec!["test".to_string()],
        capabilities: PluginCapabilities {
            search: true,
            network_access: false,
            file_system_access: false,
            ..Default::default()
        },
        permissions: PluginPermissions {
            network_hosts: vec![],
            read_files: vec![],
            write_files: vec![],
            ..Default::default()
        },
        configuration: vec![],
        preferences: vec![],
        commands: vec![],
        actions: vec![],
        dependencies: HashMap::new(),
        environment: HashMap::new(),
        min_launcher_version: "1.0.0".to_string(),
        max_launcher_version: None,
        update_url: None,
        changelog_url: None,
    };

    let mock_plugin = MockPlugin { manifest };
    let wrapper = NativePluginWrapper::new(Box::new(mock_plugin))?;

    let mut app = App::new();
    app.init_resource::<SearchIndex>();
    app.add_plugins(wrapper);

    // Verify plugin component was added
    let mut query = app.world_mut().query::<&PluginComponent>();
    let plugin_components: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(plugin_components.len(), 1);
    assert_eq!(plugin_components[0].id, "test-plugin");
    Ok(())
}