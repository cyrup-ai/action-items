# Task 2: Implementation - Sync Category Management System

## Implementation Scope
Implement the comprehensive sync category management system with 10 synced categories and 4 non-synced categories, including individual toggle controls, info tooltips, category icons, and selective synchronization logic.

## Core Implementation

### 1. Category Data Models
```rust
// Category management based on examples/ui/ui.rs:15-40 and examples/asset_loading/asset_loading.rs:25-50
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Hash)]
pub enum SyncCategory {
    // Synced Categories
    SearchHistory,
    Aliases,
    Hotkeys,
    ExtensionsAndSettings,
    Quicklinks,
    Snippets,
    RaycastNotes,
    Themes,
    AiChatsPresetsCommands,
    CustomWindowManagement,
    
    // Non-Synced Categories
    CredentialsAndPasswords,
    GeneralAndAdvancedSettings,
    ClipboardHistory,
    ScriptCommands,
}

#[derive(Component, Clone, Debug)]
pub struct SyncCategoryItem {
    pub category: SyncCategory,
    pub display_name: String,
    pub description: String,
    pub icon_path: String,
    pub is_syncable: bool,
    pub is_enabled: bool,
    pub has_info_tooltip: bool,
    pub tooltip_text: Option<String>,
    pub data_size_mb: f32,
    pub last_sync: Option<DateTime<Utc>>,
}

impl SyncCategoryItem {
    fn new_synced(
        category: SyncCategory,
        display_name: &str,
        description: &str,
        icon_path: &str,
    ) -> Self {
        Self {
            category,
            display_name: display_name.to_string(),
            description: description.to_string(),
            icon_path: icon_path.to_string(),
            is_syncable: true,
            is_enabled: true,
            has_info_tooltip: false,
            tooltip_text: None,
            data_size_mb: 0.0,
            last_sync: None,
        }
    }
    
    fn new_non_synced(
        category: SyncCategory,
        display_name: &str,
        description: &str,
        icon_path: &str,
        tooltip_text: &str,
    ) -> Self {
        Self {
            category,
            display_name: display_name.to_string(),
            description: description.to_string(),
            icon_path: icon_path.to_string(),
            is_syncable: false,
            is_enabled: false,
            has_info_tooltip: true,
            tooltip_text: Some(tooltip_text.to_string()),
            data_size_mb: 0.0,
            last_sync: None,
        }
    }
}
```

### 2. Category Data Initialization
```rust
// Category initialization based on examples/asset_loading/asset_loading.rs:75-100
#[derive(Resource)]
pub struct SyncCategoryRegistry {
    pub synced_categories: Vec<SyncCategoryItem>,
    pub non_synced_categories: Vec<SyncCategoryItem>,
    pub category_icons: HashMap<SyncCategory, Handle<Image>>,
    pub sync_states: HashMap<SyncCategory, bool>,
}

impl Default for SyncCategoryRegistry {
    fn default() -> Self {
        let synced_categories = vec![
            SyncCategoryItem::new_synced(
                SyncCategory::SearchHistory,
                "Search History",
                "Synchronize search patterns and frequently accessed items",
                "icons/magnifying_glass.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::Aliases,
                "Aliases",
                "Custom command aliases and shortcuts",
                "icons/text_cursor.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::Hotkeys,
                "Hotkeys",
                "Global hotkey assignments and custom shortcuts",
                "icons/keyboard_shortcut.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::ExtensionsAndSettings,
                "Extensions and Settings",
                "Extension configurations and user preferences",
                "icons/gear_settings.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::Quicklinks,
                "Quicklinks",
                "Custom quicklink collections and URLs",
                "icons/link_chain.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::Snippets,
                "Snippets",
                "Text snippets and code templates",
                "icons/code_snippet.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::RaycastNotes,
                "Raycast Notes",
                "User-created notes and documentation",
                "icons/document_note.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::Themes,
                "Themes",
                "Custom themes and visual preferences",
                "icons/theme_palette.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::AiChatsPresetsCommands,
                "AI Chats, Presets & Commands",
                "AI conversation history and custom AI configurations",
                "icons/ai_sparkle.png"
            ),
            SyncCategoryItem::new_synced(
                SyncCategory::CustomWindowManagement,
                "Custom Window Management Commands",
                "User-defined window management automations",
                "icons/window_layout.png"
            ),
        ];
        
        let non_synced_categories = vec![
            SyncCategoryItem::new_non_synced(
                SyncCategory::CredentialsAndPasswords,
                "Credentials and Passwords",
                "Not synced for security reasons",
                "icons/shield_security.png",
                "Passwords are not synced for security. Use a dedicated password manager."
            ),
            SyncCategoryItem::new_non_synced(
                SyncCategory::GeneralAndAdvancedSettings,
                "General and Advanced Settings",
                "Device-specific configurations",
                "icons/gear_advanced.png",
                "System settings are device-specific and not synced."
            ),
            SyncCategoryItem::new_non_synced(
                SyncCategory::ClipboardHistory,
                "Clipboard History",
                "Not synced for privacy and performance",
                "icons/clipboard.png",
                "Clipboard history is not synced to protect privacy and reduce bandwidth."
            ),
            SyncCategoryItem::new_non_synced(
                SyncCategory::ScriptCommands,
                "Script Commands",
                "Device-specific due to local dependencies",
                "icons/code_script.png",
                "Scripts have local dependencies and file paths, so they're not synced."
            ),
        ];
        
        Self {
            synced_categories,
            non_synced_categories,
            category_icons: HashMap::new(),
            sync_states: HashMap::new(),
        }
    }
}

fn initialize_category_registry(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut registry: ResMut<SyncCategoryRegistry>,
) {
    // Load category icons
    for category in &registry.synced_categories {
        let handle = asset_server.load(&category.icon_path);
        registry.category_icons.insert(category.category.clone(), handle);
        registry.sync_states.insert(category.category.clone(), category.is_enabled);
    }
    
    for category in &registry.non_synced_categories {
        let handle = asset_server.load(&category.icon_path);
        registry.category_icons.insert(category.category.clone(), handle);
        registry.sync_states.insert(category.category.clone(), false);
    }
}
```

### 3. Category List UI System
```rust
// Category list rendering based on examples/ui/ui.rs:125-160
#[derive(Component, Debug)]
pub struct CategoryListContainer {
    pub column_type: CategoryColumnType,
    pub scroll_offset: f32,
    pub item_height: f32,
    pub visible_items: usize,
}

fn setup_synced_categories_list(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    registry: &Res<SyncCategoryRegistry>,
) {
    parent
        .spawn(ScrollingList::default())
        .insert(CategoryListContainer {
            column_type: CategoryColumnType::Synced,
            scroll_offset: 0.0,
            item_height: 56.0,
            visible_items: 10,
        })
        .with_children(|list_container| {
            for category in &registry.synced_categories {
                spawn_category_item(list_container, asset_server, category, true);
            }
        });
}

fn setup_non_synced_categories_list(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    registry: &Res<SyncCategoryRegistry>,
) {
    parent
        .spawn(ScrollingList::default())
        .insert(CategoryListContainer {
            column_type: CategoryColumnType::NotSynced,
            scroll_offset: 0.0,
            item_height: 56.0,
            visible_items: 4,
        })
        .with_children(|list_container| {
            for category in &registry.non_synced_categories {
                spawn_category_item(list_container, asset_server, category, false);
            }
        });
}

fn spawn_category_item(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    category: &SyncCategoryItem,
    is_syncable: bool,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                margin: UiRect::bottom(Val::Px(2.0)),
                ..default()
            },
            background_color: Color::rgba(0.12, 0.12, 0.12, 0.8).into(),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            ..default()
        })
        .insert(category.clone())
        .with_children(|item| {
            // Category icon
            item.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    margin: UiRect::right(Val::Px(12.0)),
                    ..default()
                },
                image: asset_server.load(&category.icon_path).into(),
                ..default()
            });
            
            // Category details container
            item.spawn(NodeBundle {
                style: Style {
                    flex_direction: FlexDirection::Column,
                    flex_grow: 1.0,
                    justify_content: JustifyContent::Center,
                    ..default()
                },
                ..default()
            })
            .with_children(|details| {
                // Category name
                details.spawn(TextBundle::from_section(
                    &category.display_name,
                    TextStyle {
                        font: asset_server.load("fonts/Inter-Medium.ttf"),
                        font_size: 14.0,
                        color: Color::rgba(0.9, 0.9, 0.9, 1.0),
                    },
                ));
                
                // Category description
                details.spawn(TextBundle::from_section(
                    &category.description,
                    TextStyle {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 12.0,
                        color: Color::rgba(0.6, 0.6, 0.6, 1.0),
                    },
                ));
            });
            
            // Right side controls
            if is_syncable {
                // Toggle switch for synced categories
                spawn_category_toggle_switch(item, asset_server, category.is_enabled);
            } else {
                // Info icon for non-synced categories
                spawn_info_icon(item, asset_server, category.tooltip_text.as_deref().unwrap_or(""));
            }
        });
}
```

### 4. Category Toggle Switch System
```rust
// Category toggle switches based on examples/ui/ui_texture_atlas.rs:85-110
#[derive(Component, Debug)]
pub struct CategoryToggleSwitch {
    pub category: SyncCategory,
    pub enabled: bool,
    pub interaction_state: ToggleInteractionState,
}

fn spawn_category_toggle_switch(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    enabled: bool,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(44.0),
                height: Val::Px(24.0),
                border_radius: BorderRadius::all(Val::Px(12.0)),
                align_items: AlignItems::Center,
                justify_content: if enabled { JustifyContent::FlexEnd } else { JustifyContent::FlexStart },
                padding: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            background_color: if enabled { 
                Color::rgba(0.0, 0.48, 1.0, 1.0) 
            } else { 
                Color::rgba(0.3, 0.3, 0.3, 1.0) 
            }.into(),
            ..default()
        })
        .insert(CategoryToggleSwitch {
            category: SyncCategory::SearchHistory, // Will be set properly by parent
            enabled,
            interaction_state: ToggleInteractionState::Normal,
        })
        .with_children(|switch| {
            // Toggle knob
            switch.spawn(NodeBundle {
                style: Style {
                    width: Val::Px(20.0),
                    height: Val::Px(20.0),
                    ..default()
                },
                background_color: Color::WHITE.into(),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            });
        });
}

// Toggle interaction system based on examples/input/mouse_input.rs:125-150
fn handle_category_toggle_interaction(
    mut toggle_query: Query<
        (&Interaction, &mut CategoryToggleSwitch, &mut BackgroundColor, &mut Style),
        (Changed<Interaction>, With<Button>)
    >,
    mut registry: ResMut<SyncCategoryRegistry>,
    mut sync_events: EventWriter<SyncCategoryChangedEvent>,
) {
    for (interaction, mut toggle, mut background_color, mut style) in toggle_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                toggle.enabled = !toggle.enabled;
                
                // Update registry state
                registry.sync_states.insert(toggle.category.clone(), toggle.enabled);
                
                // Update visual state
                if toggle.enabled {
                    *background_color = Color::rgba(0.0, 0.48, 1.0, 1.0).into();
                    style.justify_content = JustifyContent::FlexEnd;
                } else {
                    *background_color = Color::rgba(0.3, 0.3, 0.3, 1.0).into();
                    style.justify_content = JustifyContent::FlexStart;
                }
                
                // Send sync event
                sync_events.send(SyncCategoryChangedEvent {
                    category: toggle.category.clone(),
                    enabled: toggle.enabled,
                    timestamp: Utc::now(),
                });
            }
            Interaction::Hovered => {
                toggle.interaction_state = ToggleInteractionState::Hovered;
            }
            Interaction::None => {
                toggle.interaction_state = ToggleInteractionState::Normal;
            }
        }
    }
}
```

### 5. Info Icon and Tooltip System
```rust
// Info icon and tooltip system based on examples/ui/ui_texture_atlas.rs:135-160
#[derive(Component, Debug)]
pub struct InfoIcon {
    pub tooltip_text: String,
    pub is_hovered: bool,
    pub tooltip_visible: bool,
}

fn spawn_info_icon(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
    tooltip_text: &str,
) {
    parent
        .spawn(ButtonBundle {
            style: Style {
                width: Val::Px(20.0),
                height: Val::Px(20.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            background_color: Color::TRANSPARENT.into(),
            ..default()
        })
        .insert(InfoIcon {
            tooltip_text: tooltip_text.to_string(),
            is_hovered: false,
            tooltip_visible: false,
        })
        .with_children(|icon| {
            icon.spawn(ImageBundle {
                style: Style {
                    width: Val::Px(16.0),
                    height: Val::Px(16.0),
                    ..default()
                },
                image: asset_server.load("icons/info_circle.png").into(),
                ..default()
            });
        });
}

// Tooltip interaction system based on examples/input/mouse_input.rs:175-200
fn handle_info_icon_interaction(
    mut info_query: Query<(&Interaction, &mut InfoIcon), (Changed<Interaction>, With<Button>)>,
    mut tooltip_events: EventWriter<ShowTooltipEvent>,
) {
    for (interaction, mut info_icon) in info_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                if !info_icon.is_hovered {
                    info_icon.is_hovered = true;
                    tooltip_events.send(ShowTooltipEvent {
                        text: info_icon.tooltip_text.clone(),
                        position: Vec2::ZERO, // Will be calculated by tooltip system
                    });
                }
            }
            Interaction::None => {
                if info_icon.is_hovered {
                    info_icon.is_hovered = false;
                    tooltip_events.send(ShowTooltipEvent {
                        text: String::new(), // Empty text hides tooltip
                        position: Vec2::ZERO,
                    });
                }
            }
            _ => {}
        }
    }
}
```

### 6. Sync Event System
```rust
// Sync event system based on examples/ecs/event.rs:25-50
#[derive(Event, Debug, Clone)]
pub struct SyncCategoryChangedEvent {
    pub category: SyncCategory,
    pub enabled: bool,
    pub timestamp: DateTime<Utc>,
}

#[derive(Event, Debug, Clone)]
pub struct ShowTooltipEvent {
    pub text: String,
    pub position: Vec2,
}

#[derive(Event, Debug, Clone)]
pub struct InitiateCategorySyncEvent {
    pub categories: Vec<SyncCategory>,
    pub force_sync: bool,
}

fn process_category_sync_events(
    mut sync_events: EventReader<SyncCategoryChangedEvent>,
    mut initiate_sync: EventWriter<InitiateCategorySyncEvent>,
    registry: Res<SyncCategoryRegistry>,
    sync_interface: Res<CloudSyncInterface>,
) {
    for event in sync_events.read() {
        info!("Category sync state changed: {:?} -> {}", event.category, event.enabled);
        
        // If master sync is enabled and category was just enabled, trigger immediate sync
        if sync_interface.master_sync_enabled && event.enabled {
            initiate_sync.send(InitiateCategorySyncEvent {
                categories: vec![event.category.clone()],
                force_sync: false,
            });
        }
    }
}
```

## Bevy Example References
- **UI list management**: `examples/ui/ui.rs:15-40` - Dynamic category lists with icons
- **Asset loading**: `examples/asset_loading/asset_loading.rs:25-50` - Category icon loading
- **Toggle switches**: `examples/ui/ui_texture_atlas.rs:85-110` - Individual category toggles
- **Input handling**: `examples/input/mouse_input.rs:125-150` - Category toggle interactions
- **Event systems**: `examples/ecs/event.rs:25-50` - Sync state change events
- **Tooltip system**: `examples/ui/ui_texture_atlas.rs:135-160` - Info icon tooltips

## Architecture Integration Notes
- **File**: `ui/src/cloud_sync/category_management.rs:1-600`
- **Dependencies**: Cloud sync engine, settings persistence, asset management
- **Integration**: Main interface, sync status, user preferences
- **Performance**: Efficient list rendering with virtualization for large datasets

## Success Criteria
1. **Complete category list** with all 14 categories (10 synced + 4 non-synced) properly displayed
2. **Individual toggles** working independently for all synced categories
3. **Info tooltips** displaying correctly for non-synced categories with proper text
4. **Visual consistency** with proper icons, spacing, and theme compliance
5. **State persistence** maintaining toggle states across app restarts
6. **Real-time sync** triggered immediately when categories are enabled
7. **Smooth interactions** with proper hover states and click feedback