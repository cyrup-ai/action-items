use std::path::PathBuf;

use bevy::app::App;
use action_items_core::raycast::wrapper::{RaycastPluginWrapper, RaycastPluginComponent};
use action_items_core::raycast::loader::{RaycastCommand, RaycastExtension};
use action_items_core::search::SearchIndex;

fn create_test_extension() -> RaycastExtension {
    RaycastExtension {
        id: "test-extension-id".to_string(),
        name: "test-extension".to_string(),
        title: "Test Extension".to_string(),
        description: "A test Raycast extension".to_string(),
        author: "Test Author".to_string(),
        categories: vec!["test".to_string(), "raycast".to_string()],
        icon: None,
        path: PathBuf::from("/test/path"),
        commands: vec![RaycastCommand {
            name: "test-command".to_string(),
            title: "Test Command".to_string(),
            description: Some("Test command description".to_string()),
            mode: "view".to_string(),
        }],
    }
}

#[test]
fn test_raycast_plugin_wrapper_creation() -> Result<(), Box<dyn std::error::Error>> {
    let extension = create_test_extension();
    let wrapper = RaycastPluginWrapper::new(extension)?;

    assert_eq!(wrapper.metadata().id, "raycast:test-extension");
    assert_eq!(wrapper.metadata().name, "test-extension");
    assert_eq!(wrapper.metadata().description, "A test Raycast extension");
    Ok(())
}

#[test]
fn test_raycast_plugin_registration() -> Result<(), Box<dyn std::error::Error>> {
    let extension = create_test_extension();
    let wrapper = RaycastPluginWrapper::new(extension)?;

    let mut app = App::new();
    app.init_resource::<SearchIndex>();
    app.add_plugins(wrapper);

    // Verify plugin component was added
    let mut query = app.world_mut().query::<&RaycastPluginComponent>();
    let plugin_components: Vec<_> = query.iter(app.world()).collect();
    assert_eq!(plugin_components.len(), 1);
    assert_eq!(plugin_components[0].id, "raycast:test-extension");
    assert_eq!(plugin_components[0].name, "test-extension");
    Ok(())
}

#[test]
fn test_raycast_plugin_search_index_integration() -> Result<(), Box<dyn std::error::Error>> {
    let extension = create_test_extension();
    let wrapper = RaycastPluginWrapper::new(extension)?;

    let mut app = App::new();
    app.init_resource::<SearchIndex>();
    app.add_plugins(wrapper);

    // Verify items were added to search index
    let search_index = app
        .world()
        .get_resource::<SearchIndex>()
        .ok_or("SearchIndex should be available")?;
    let results = search_index.search("test");

    // Should find at least the main extension and one command
    assert!(!results.is_empty());
    assert!(
        results
            .iter()
            .any(|item| item.id.starts_with("raycast:test-extension"))
    );
    Ok(())
}