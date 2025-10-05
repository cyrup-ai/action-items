# Cloud Sync Menu Specification

## Overview
The Cloud Sync Menu provides comprehensive control over data synchronization across Apple devices. This interface manages selective synchronization of user data categories, ensuring seamless continuity while maintaining user privacy and security preferences.

## Layout Architecture
- **Base Layout**: Cloud Sync tab active in primary navigation
- **Split Layout**: Left branding panel (40%) and right sync management panel (60%)
- **Two-Column Sync Display**: "Synced" and "Not Synced" categories
- **Central Control**: Prominent toggle switch for overall sync control

## Left Panel - Cloud Sync Branding

### Visual Identity Section
- **Icon**: Cloud with bidirectional arrows indicating sync capability
- **Title**: "Cloud Sync" in large, prominent typography
- **Description**: "Enable cloud sync to keep settings and data synchronized across your Apple devices."
- **Purpose**: Clear explanation of cloud sync functionality and benefits

### Master Control Switch
- **Control Type**: Large iOS-style toggle switch
- **Current State**: Enabled (blue active state)
- **Functionality**: Master control for entire cloud synchronization system
- **Visual Feedback**: Prominent blue color indicates active sync status

### Sync Status Display
- **Status Information**: "Last Synced Aug 6, 2025 at 6:28 PM"
- **Purpose**: Real-time sync status and timestamp information
- **Format**: Human-readable timestamp with full date and time
- **Dynamic Updates**: Real-time updates reflecting latest sync activity

## Right Panel - Sync Category Management

### Synced Data Categories

#### User Behavior Data
1. **Search History**
   - **Icon**: Magnifying glass
   - **Purpose**: Synchronize search patterns and frequently accessed items
   - **Privacy Consideration**: Optional due to personal usage patterns

2. **Aliases**
   - **Icon**: Text cursor/alias symbol
   - **Purpose**: Custom command aliases and shortcuts
   - **Sync Benefit**: Consistent quick access across devices

#### System Configuration
3. **Hotkeys**
   - **Icon**: Keyboard shortcut symbol
   - **Purpose**: Global hotkey assignments and custom shortcuts
   - **Cross-Device**: Ensures consistent keyboard workflows

4. **Extensions and Settings**
   - **Icon**: Gear/settings symbol
   - **Purpose**: Extension configurations and user preferences
   - **Complexity**: Comprehensive settings synchronization

#### Content and Customization
5. **Quicklinks**
   - **Icon**: Link chain symbol
   - **Purpose**: Custom quicklink collections and URLs
   - **Productivity**: Shared quick access to frequently used resources

6. **Snippets**
   - **Icon**: Code snippet symbol
   - **Purpose**: Text snippets and code templates
   - **Workflow**: Shared productivity snippets across devices

7. **Raycast Notes**
   - **Icon**: Note/document symbol
   - **Purpose**: User-created notes and documentation
   - **Content Sync**: Personal knowledge base synchronization

8. **Themes**
   - **Icon**: Theme/palette symbol
   - **Purpose**: Custom themes and visual preferences
   - **Personalization**: Consistent visual experience across devices

#### Advanced Features
9. **AI Chats, Presets & Commands**

## Bevy Implementation Details

### Cloud Sync Component Architecture

```rust
use bevy::{prelude::*, utils::HashMap};

// Cloud sync menu components
#[derive(Component, Reflect)]
pub struct CloudSyncMenu;

#[derive(Component, Reflect)]
pub struct SyncMasterToggle {
    pub is_enabled: bool,
    pub animation_progress: f32,
}

#[derive(Component, Reflect)]
pub struct SyncStatusDisplay {
    pub last_sync_time: Option<f64>,
    pub sync_status: SyncStatus,
    pub total_synced_items: u32,
}

#[derive(Component, Reflect)]
pub struct SyncCategoryToggle {
    pub category: DataCategory,
    pub is_enabled: bool,
    pub item_count: u32,
    pub last_updated: Option<f64>,
}

#[derive(Component, Reflect)]
pub struct CloudSyncBranding {
    pub title: String,
    pub description: String,
    pub icon_path: String,
}

// Data structure definitions
#[derive(Clone, Reflect, PartialEq, Hash)]
pub enum DataCategory {
    SearchHistory,
    Aliases,
    Hotkeys,
    ExtensionsAndSettings,
    Quicklinks,
    Snippets,
    RaycastNotes,
    Themes,
    AiChatsPresetsCommands,
    Calendar,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum SyncStatus {
    InProgress,
    Completed,
    Failed(String),
    Disabled,
    NeverSynced,
}

#[derive(Clone, Reflect)]
pub struct SyncCategoryInfo {
    pub category: DataCategory,
    pub display_name: String,
    pub description: String,
    pub icon_path: String,
    pub is_synced: bool,
    pub item_count: u32,
    pub privacy_sensitive: bool,
    pub size_estimate_mb: f32,
}

// Sync progress tracking
#[derive(Component, Reflect)]
pub struct SyncProgressIndicator {
    pub category: DataCategory,
    pub progress_percentage: f32,
    pub is_active: bool,
}
```

### Resource Management for Cloud Sync

```rust
// Global cloud sync state
#[derive(Resource, Reflect)]
pub struct CloudSyncState {
    pub is_enabled: bool,
    pub sync_categories: HashMap<DataCategory, SyncCategoryState>,
    pub last_full_sync: Option<f64>,
    pub apple_id_connected: bool,
    pub sync_device_count: u32,
    pub total_storage_used_mb: f32,
    pub storage_limit_mb: f32,
}

#[derive(Clone, Reflect)]
pub struct SyncCategoryState {
    pub is_enabled: bool,
    pub item_count: u32,
    pub last_sync: Option<f64>,
    pub sync_status: SyncStatus,
    pub size_mb: f32,
    pub conflict_count: u32,
}

// Cloud service configuration
#[derive(Resource, Reflect)]
pub struct CloudSyncConfiguration {
    pub sync_interval_minutes: u32,
    pub auto_sync_enabled: bool,
    pub wifi_only: bool,
    pub background_sync_enabled: bool,
    pub conflict_resolution: ConflictResolution,
    pub encryption_enabled: bool,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum ConflictResolution {
    Manual,
    KeepLocal,
    KeepRemote,
    MergeWhenPossible,
}

// Device information for multi-device sync
#[derive(Clone, Reflect)]
pub struct SyncedDevice {
    pub device_id: String,
    pub device_name: String,
    pub device_type: String,
    pub last_sync: Option<f64>,
    pub is_current_device: bool,
    pub os_version: String,
    pub app_version: String,
}

#[derive(Resource, Reflect)]
pub struct SyncDeviceRegistry {
    pub connected_devices: Vec<SyncedDevice>,
    pub primary_device: Option<String>,
    pub sync_conflicts: Vec<SyncConflict>,
}

#[derive(Clone, Reflect)]
pub struct SyncConflict {
    pub conflict_id: String,
    pub category: DataCategory,
    pub item_name: String,
    pub local_version: SyncItemVersion,
    pub remote_version: SyncItemVersion,
    pub conflict_type: ConflictType,
}

#[derive(Clone, Reflect)]
pub struct SyncItemVersion {
    pub content_hash: String,
    pub last_modified: f64,
    pub device_origin: String,
    pub size_bytes: u32,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum ConflictType {
    ContentMismatch,
    DeletedLocally,
    DeletedRemotely,
    VersionMismatch,
}
```

### Event System for Cloud Sync

```rust
// Cloud sync events
#[derive(Event, Reflect)]
pub enum CloudSyncEvent {
    // Master sync control
    MasterSyncToggled(bool),
    SyncInitiated,
    SyncCompleted(SyncResult),
    SyncFailed(String),
    
    // Category-specific events
    CategorySyncToggled(DataCategory, bool),
    CategorySyncStarted(DataCategory),
    CategorySyncCompleted(DataCategory, u32), // item count
    CategorySyncFailed(DataCategory, String),
    
    // Conflict resolution
    ConflictDetected(SyncConflict),
    ConflictResolved(String, ConflictResolution),
    ConflictIgnored(String),
    
    // Device management
    DeviceConnected(String),
    DeviceDisconnected(String),
    DeviceRenamed(String, String),
}

#[derive(Event, Reflect)]
pub struct SyncProgressUpdate {
    pub category: Option<DataCategory>,
    pub progress_percentage: f32,
    pub current_item: Option<String>,
    pub items_synced: u32,
    pub total_items: u32,
}

#[derive(Event, Reflect)]
pub struct SyncStorageUpdate {
    pub used_mb: f32,
    pub limit_mb: f32,
    pub category_breakdown: HashMap<DataCategory, f32>,
}

#[derive(Clone, Reflect)]
pub struct SyncResult {
    pub success: bool,
    pub categories_synced: Vec<DataCategory>,
    pub total_items: u32,
    pub conflicts_resolved: u32,
    pub errors: Vec<String>,
    pub duration_seconds: f32,
}
```

### System Architecture for Cloud Sync

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum CloudSyncSystems {
    Input,
    SyncEngine,
    ConflictResolution,
    DeviceManagement,
    StorageManagement,
    StateUpdate,
    Animation,
    Rendering,
}

impl Plugin for CloudSyncPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<CloudSyncState>()
            .init_resource::<CloudSyncConfiguration>()
            .init_resource::<SyncDeviceRegistry>()
            .init_resource::<SyncEngine>()
            
            // Events
            .add_event::<CloudSyncEvent>()
            .add_event::<SyncProgressUpdate>()
            .add_event::<SyncStorageUpdate>()
            
            // System ordering
            .configure_sets(Update, (
                CloudSyncSystems::Input,
                CloudSyncSystems::SyncEngine,
                CloudSyncSystems::ConflictResolution,
                CloudSyncSystems::DeviceManagement,
                CloudSyncSystems::StorageManagement,
                CloudSyncSystems::StateUpdate,
                CloudSyncSystems::Animation,
                CloudSyncSystems::Rendering,
            ).chain())
            
            // Systems
            .add_systems(Startup, (
                setup_cloud_sync_menu,
                initialize_sync_state,
                register_sync_categories,
            ))
            
            .add_systems(Update, (
                handle_sync_toggle_interactions,
                handle_category_toggle_interactions,
                handle_sync_initiation,
            ).in_set(CloudSyncSystems::Input))
            
            .add_systems(Update, (
                process_sync_operations,
                monitor_sync_progress,
                handle_sync_completion,
            ).in_set(CloudSyncSystems::SyncEngine))
            
            .add_systems(Update, (
                detect_sync_conflicts,
                process_conflict_resolutions,
                update_conflict_status,
            ).in_set(CloudSyncSystems::ConflictResolution))
            
            .add_systems(Update, (
                discover_connected_devices,
                monitor_device_status,
                handle_device_changes,
            ).in_set(CloudSyncSystems::DeviceManagement))
            
            .add_systems(Update, (
                monitor_storage_usage,
                calculate_category_sizes,
                enforce_storage_limits,
            ).in_set(CloudSyncSystems::StorageManagement))
            
            .add_systems(Update, (
                update_cloud_sync_state,
                persist_sync_preferences,
                sync_timestamp_updates,
            ).in_set(CloudSyncSystems::StateUpdate))
            
            .add_systems(Update, (
                animate_sync_progress_indicators,
                animate_toggle_switches,
                animate_status_changes,
            ).in_set(CloudSyncSystems::Animation))
            
            .add_systems(Update, (
                update_sync_status_display,
                update_category_counts,
                update_progress_indicators,
            ).in_set(CloudSyncSystems::Rendering));
    }
}
```

### Layout Implementation with Sync Category Management

```rust
fn setup_cloud_sync_menu(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    sync_state: Res<CloudSyncState>,
) {
    // Root container with split layout
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(1200.0),
            max_height: Val::Px(800.0),
            flex_direction: FlexDirection::Row,
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        CloudSyncMenu,
    )).with_children(|parent| {
        
        // Left panel - Branding and master control (40%)
        parent.spawn((
            Node {
                width: Val::Percent(40.0),
                max_width: Val::Px(480.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(32.0)),
                row_gap: Val::Px(24.0),
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        )).with_children(|left_parent| {
            spawn_cloud_sync_branding(left_parent, &asset_server, &sync_state);
        });
        
        // Right panel - Sync category management (60%)
        parent.spawn((
            Node {
                width: Val::Percent(60.0),
                max_width: Val::Px(720.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column,
                flex_grow: 0.0,
                padding: UiRect::all(Val::Px(24.0)),
                row_gap: Val::Px(16.0),
                overflow: Overflow::clip_y(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        )).with_children(|right_parent| {
            spawn_sync_categories_section(right_parent, &asset_server, &sync_state);
        });
    });
}

fn spawn_cloud_sync_branding(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sync_state: &CloudSyncState,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_height: Val::Px(400.0),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_grow: 0.0,
            row_gap: Val::Px(20.0),
            ..default()
        },
        CloudSyncBranding {
            title: "Cloud Sync".to_string(),
            description: "Enable cloud sync to keep settings and data synchronized across your Apple devices.".to_string(),
            icon_path: "icons/cloud_sync.png".to_string(),
        },
    )).with_children(|branding_parent| {
        
        // Cloud sync icon
        branding_parent.spawn((
            ImageNode::new(asset_server.load("icons/cloud_sync.png")),
            Node {
                width: Val::Px(80.0),
                height: Val::Px(80.0),
                max_width: Val::Px(80.0),
                max_height: Val::Px(80.0),
                flex_grow: 0.0,
                ..default()
            },
        ));
        
        // Title and description
        branding_parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Auto,
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                flex_grow: 0.0,
                row_gap: Val::Px(8.0),
                ..default()
            },
        )).with_children(|text_parent| {
            
            // Title
            text_parent.spawn((
                Text::new("Cloud Sync"),
                TextFont {
                    font: asset_server.load("fonts/Inter-Bold.ttf"),
                    font_size: 28.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
            ));
            
            // Description
            text_parent.spawn((
                Text::new("Enable cloud sync to keep settings and data synchronized across your Apple devices."),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 14.0,
                    ..default()
                },
                TextColor(Color::srgb(0.7, 0.7, 0.7)),
                TextLayout::new_with_justify(JustifyText::Center),
                Node {
                    max_width: Val::Px(300.0),
                    ..default()
                },
            ));
        });
        
        // Master sync toggle
        spawn_master_sync_toggle(branding_parent, asset_server, sync_state.is_enabled);
        
        // Sync status display
        if let Some(last_sync) = sync_state.last_full_sync {
            spawn_sync_status_display(branding_parent, asset_server, last_sync);
        }
    });
}

fn spawn_sync_categories_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    sync_state: &CloudSyncState,
) {
    let sync_categories = vec![
        SyncCategoryInfo {
            category: DataCategory::SearchHistory,
            display_name: "Search History".to_string(),
            description: "Recent searches and query patterns".to_string(),
            icon_path: "icons/search.png".to_string(),
            is_synced: true,
            item_count: 156,
            privacy_sensitive: true,
            size_estimate_mb: 2.3,
        },
        SyncCategoryInfo {
            category: DataCategory::Aliases,
            display_name: "Aliases".to_string(),
            description: "Custom command shortcuts".to_string(),
            icon_path: "icons/alias.png".to_string(),
            is_synced: true,
            item_count: 23,
            privacy_sensitive: false,
            size_estimate_mb: 0.1,
        },
        SyncCategoryInfo {
            category: DataCategory::Hotkeys,
            display_name: "Hotkeys".to_string(),
            description: "Global keyboard shortcuts".to_string(),
            icon_path: "icons/keyboard.png".to_string(),
            is_synced: true,
            item_count: 18,
            privacy_sensitive: false,
            size_estimate_mb: 0.05,
        },
        SyncCategoryInfo {
            category: DataCategory::ExtensionsAndSettings,
            display_name: "Extensions and Settings".to_string(),
            description: "Extension configurations and preferences".to_string(),
            icon_path: "icons/extensions.png".to_string(),
            is_synced: true,
            item_count: 42,
            privacy_sensitive: false,
            size_estimate_mb: 1.8,
        },
        // Add more categories as needed...
    ];
    
    // Section title
    parent.spawn((
        Text::new("Sync Categories"),
        TextFont {
            font: asset_server.load("fonts/Inter-Bold.ttf"),
            font_size: 18.0,
            ..default()
        },
        TextColor(Color::srgb(1.0, 1.0, 1.0)),
        Node {
            margin: UiRect::bottom(Val::Px(16.0)),
            ..default()
        },
    ));
    
    // Categories container with scroll
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_height: Val::Px(600.0),
            flex_direction: FlexDirection::Column,
            flex_grow: 0.0,
            row_gap: Val::Px(8.0),
            overflow: Overflow::clip_y(),
            ..default()
        },
    )).with_children(|categories_parent| {
        
        for category_info in sync_categories {
            spawn_sync_category_row(
                categories_parent,
                asset_server,
                &category_info,
                sync_state.is_enabled,
            );
        }
    });
}

fn spawn_sync_category_row(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    category_info: &SyncCategoryInfo,
    master_sync_enabled: bool,
) {
    parent.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Px(60.0),
            max_height: Val::Px(60.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            flex_grow: 0.0,
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        BorderColor::all(Color::srgb(0.2, 0.2, 0.2)),
        BorderRadius::all(Val::Px(6.0)),
    )).with_children(|row_parent| {
        
        // Left side - Icon and category info
        row_parent.spawn((
            Node {
                width: Val::Auto,
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(12.0),
                flex_grow: 1.0,
                ..default()
            },
        )).with_children(|left_parent| {
            
            // Category icon
            left_parent.spawn((
                ImageNode::new(asset_server.load(&category_info.icon_path)),
                Node {
                    width: Val::Px(24.0),
                    height: Val::Px(24.0),
                    max_width: Val::Px(24.0),
                    max_height: Val::Px(24.0),
                    flex_grow: 0.0,
                    ..default()
                },
            ));
            
            // Category name and details
            left_parent.spawn((
                Node {
                    width: Val::Auto,
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    flex_grow: 1.0,
                    row_gap: Val::Px(2.0),
                    ..default()
                },
            )).with_children(|info_parent| {
                
                // Category name
                info_parent.spawn((
                    Text::new(&category_info.display_name),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Medium.ttf"),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(if master_sync_enabled { 
                        Color::srgb(1.0, 1.0, 1.0) 
                    } else { 
                        Color::srgb(0.5, 0.5, 0.5) 
                    }),
                ));
                
                // Item count and size
                info_parent.spawn((
                    Text::new(&format!("{} items · {:.1} MB", 
                        category_info.item_count, 
                        category_info.size_estimate_mb
                    )),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.6, 0.6, 0.6)),
                ));
            });
        });
        
        // Right side - Toggle switch
        spawn_category_toggle_switch(
            row_parent, 
            asset_server, 
            category_info.category.clone(),
            category_info.is_synced && master_sync_enabled
        );
    });
}

fn spawn_master_sync_toggle(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    is_enabled: bool,
) {
    parent.spawn((
        Button,
        Node {
            width: Val::Px(80.0),
            height: Val::Px(46.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        },
        BackgroundColor(if is_enabled { 
            Color::srgb(0.0, 0.48, 1.0) 
        } else { 
            Color::srgb(0.3, 0.3, 0.3) 
        }),
        BorderRadius::all(Val::Px(23.0)),
        SyncMasterToggle {
            is_enabled,
            animation_progress: if is_enabled { 1.0 } else { 0.0 },
        },
    )).with_children(|toggle_parent| {
        
        // Toggle circle
        toggle_parent.spawn((
            Node {
                width: Val::Px(38.0),
                height: Val::Px(38.0),
                position_type: PositionType::Absolute,
                left: if is_enabled { Val::Px(38.0) } else { Val::Px(4.0) },
                top: Val::Px(4.0),
                ..default()
            },
            BackgroundColor(Color::srgb(1.0, 1.0, 1.0)),
            BorderRadius::all(Val::Px(19.0)),
        ));
    });
}
```

### Testing Strategy for Cloud Sync

```rust
#[cfg(test)]
mod cloud_sync_tests {
    use super::*;
    
    #[test]
    fn test_cloud_sync_initialization() {
        let mut app = setup_test_app();
        
        // Initialize with test sync state
        let mut sync_categories = HashMap::new();
        sync_categories.insert(DataCategory::SearchHistory, SyncCategoryState {
            is_enabled: true,
            item_count: 100,
            last_sync: Some(1000.0),
            sync_status: SyncStatus::Completed,
            size_mb: 2.5,
            conflict_count: 0,
        });
        
        let test_sync_state = CloudSyncState {
            is_enabled: true,
            sync_categories,
            last_full_sync: Some(1000.0),
            apple_id_connected: true,
            sync_device_count: 3,
            total_storage_used_mb: 15.8,
            storage_limit_mb: 100.0,
        };
        
        app.world_mut().insert_resource(test_sync_state);
        app.update();
        
        // Verify cloud sync menu was created
        let sync_menu_count = app.world().query::<&CloudSyncMenu>().iter(app.world()).count();
        assert_eq!(sync_menu_count, 1);
        
        // Verify master toggle was created
        let toggle_count = app.world().query::<&SyncMasterToggle>().iter(app.world()).count();
        assert_eq!(toggle_count, 1);
    }
    
    #[test]
    fn test_sync_category_toggle() {
        let mut app = setup_test_app();
        
        // Send category toggle event
        app.world_mut().resource_mut::<Events<CloudSyncEvent>>()
            .write(CloudSyncEvent::CategorySyncToggled(DataCategory::SearchHistory, false));
        
        app.update();
        
        // Verify toggle event was processed
        let events: Vec<_> = app.world()
            .resource::<Events<CloudSyncEvent>>()
            .get_reader()
            .read(app.world().resource::<Events<CloudSyncEvent>>())
            .collect();
        
        assert!(!events.is_empty());
    }
    
    #[test]
    fn test_master_sync_disable() {
        let mut app = setup_test_app();
        
        // Create sync toggle in enabled state
        let toggle_entity = app.world_mut().spawn((
            SyncMasterToggle {
                is_enabled: true,
                animation_progress: 1.0,
            },
        )).id();
        
        // Send master toggle event
        app.world_mut().resource_mut::<Events<CloudSyncEvent>>()
            .write(CloudSyncEvent::MasterSyncToggled(false));
        
        app.update();
        
        // Verify toggle state would be updated
        // (In real implementation, this would update the component)
    }
}
   - **Icon**: AI/sparkle symbol
   - **Purpose**: AI conversation history and custom AI configurations
   - **Intelligence**: Personalized AI behavior and history

10. **Custom Window Management Commands**
    - **Icon**: Window layout symbol
    - **Purpose**: User-defined window management automations
    - **Advanced**: Power-user customizations and workflows

### Non-Synced Data Categories

#### Security-Sensitive Data
1. **Credentials and Passwords**
   - **Icon**: Shield with info indicator
   - **Status**: Deliberately not synced for security
   - **Info Icon**: Explains security rationale
   - **Alternative**: Recommend dedicated password managers

2. **General and Advanced Settings**
   - **Icon**: Gear with info indicator
   - **Status**: Device-specific configurations
   - **Info Icon**: Explains device-specific nature
   - **Reasoning**: Hardware and system-specific settings

#### System-Specific Data
3. **Clipboard History**
   - **Icon**: Clipboard symbol
   - **Status**: Not synced (privacy/performance reasons)
   - **Privacy**: Prevents sensitive clipboard data synchronization
   - **Performance**: Avoids bandwidth usage for frequent clipboard changes

4. **Script Commands**
   - **Icon**: Code/script symbol
   - **Status**: Device-specific due to local dependencies
   - **Technical**: Local file paths and system-specific scripts
   - **Complexity**: Device-specific execution environments

## Functional Requirements

### Sync Engine Architecture
- **Selective Synchronization**: Granular control over individual data categories
- **Conflict Resolution**: Intelligent handling of concurrent modifications across devices
- **Bandwidth Optimization**: Efficient delta synchronization to minimize data usage
- **Offline Support**: Graceful handling of offline scenarios with sync queue management

### Security and Privacy Framework
- **End-to-End Encryption**: All synchronized data encrypted in transit and at rest
- **Device Authentication**: Secure device verification and authorization
- **Data Minimization**: Only selected categories synchronized based on user consent
- **Audit Trail**: Comprehensive logging of sync activities for security review

### Real-time Synchronization
- **Live Updates**: Real-time synchronization of changes across connected devices
- **Change Detection**: Intelligent detection of data modifications requiring sync
- **Push Notifications**: Optional notifications for successful sync completion
- **Status Monitoring**: Continuous monitoring of sync health and connectivity

### Error Handling and Recovery
- **Sync Failure Recovery**: Automatic retry mechanisms with exponential backoff
- **Data Integrity**: Verification of synchronized data integrity and consistency
- **Rollback Capability**: Safe rollback mechanisms for problematic sync operations
- **User Notification**: Clear communication of sync errors and resolution steps

## Bevy Implementation Examples

### Toggle Switch Implementation
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Toggle switch states and animations
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Toggle interaction handling

### Two-Column Layout System
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Flexible column layouts with proper spacing
- Reference: `./docs/bevy/examples/ui/ui.rs` - Multi-column content organization

### Category List Management
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dynamic list rendering with icons and labels
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Icon loading for categories

### Sync Status Display
- Reference: `./docs/bevy/examples/time/time.rs` - Timestamp formatting and display
- Reference: `./docs/bevy/examples/ui/text.rs` - Dynamic text updates for sync status

### Info Icon System
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Info icon management and hover states
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Info tooltip trigger handling

### Cloud Icon Animation
- Reference: `./docs/bevy/examples/animation/animated_fox.rs` - Sync progress animations
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Cloud icon state management

### Settings Persistence
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Sync preferences serialization
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Background sync operations

## State Management Requirements

### Sync State Tracking
- **Category States**: Individual tracking of sync status for each data category
- **Master State**: Overall sync enable/disable state management
- **Progress Tracking**: Real-time tracking of sync progress and completion
- **Error States**: Comprehensive error state management and user feedback

### Data Consistency Management
- **Version Control**: Tracking data versions across synchronized devices
- **Conflict Detection**: Real-time detection of data conflicts between devices
- **Merge Strategies**: Intelligent merge strategies for conflicting data
- **Consistency Validation**: Continuous validation of data consistency across devices

### Network State Management
- **Connectivity Monitoring**: Real-time monitoring of network connectivity
- **Sync Queue Management**: Intelligent queuing of sync operations during offline periods
- **Bandwidth Management**: Adaptive sync behavior based on network conditions
- **Retry Logic**: Sophisticated retry mechanisms for failed sync operations

## Security Architecture

### Data Protection Framework
- **Encryption Standards**: Industry-standard encryption for all synchronized data
- **Key Management**: Secure key generation, rotation, and management
- **Access Control**: Fine-grained access control for synchronized data categories
- **Privacy Preservation**: Minimal data collection with explicit user consent

### Authentication and Authorization
- **Device Registration**: Secure device registration and verification process
- **Multi-Device Management**: Secure management of multiple synchronized devices
- **Access Revocation**: Immediate access revocation for compromised devices
- **Session Management**: Secure session handling for sync operations

### Compliance and Auditing
- **Data Residency**: Compliance with data residency requirements and regulations
- **Audit Logging**: Comprehensive audit logging for security and compliance
- **Privacy Controls**: User control over data sharing and synchronization scope
- **Regulatory Compliance**: Compliance with GDPR, CCPA, and other privacy regulations

## Performance Optimization

### Sync Efficiency
- **Delta Synchronization**: Efficient synchronization of only changed data
- **Compression**: Data compression for bandwidth optimization
- **Batching**: Intelligent batching of sync operations for efficiency
- **Prioritization**: Smart prioritization of sync operations based on user usage

### Resource Management
- **Memory Optimization**: Efficient memory usage for sync operations
- **CPU Utilization**: Optimized CPU usage for background sync operations
- **Network Optimization**: Intelligent network usage with respect for user bandwidth
- **Battery Optimization**: Power-efficient sync operations for mobile devices

### Scalability Considerations
- **Load Balancing**: Distributed sync infrastructure for scalability
- **Caching Strategy**: Intelligent caching of frequently accessed synchronized data
- **Rate Limiting**: Appropriate rate limiting to prevent system overload
- **Monitoring**: Comprehensive monitoring of sync system performance

## Error Handling and Recovery

### Sync Operation Failures
- **Network Failures**: Graceful handling of network connectivity issues
- **Server Failures**: Robust handling of server-side sync failures
- **Data Corruption**: Detection and recovery from data corruption issues
- **Conflict Resolution**: Intelligent resolution of data conflicts between devices

### User Experience Recovery
- **Clear Error Messaging**: User-friendly error messages with actionable resolution steps
- **Automatic Recovery**: Automatic recovery mechanisms for common sync issues
- **Manual Recovery**: User-controlled recovery options for complex sync problems
- **Support Integration**: Seamless integration with support channels for unresolved issues

### Data Integrity Protection
- **Backup Systems**: Comprehensive backup systems for synchronized data
- **Integrity Validation**: Continuous validation of synchronized data integrity
- **Rollback Mechanisms**: Safe rollback mechanisms for corrupted sync operations
- **Data Recovery**: Reliable data recovery options for catastrophic failures