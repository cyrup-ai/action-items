# Task 0: Implementation - Cloud Sync Interface Layout System

## Implementation Scope
Implement the main cloud sync interface with a split-panel layout (40% left branding panel, 60% right sync management panel), master control switch, and two-column sync category display system.

## Core Implementation

### 1. Main Layout System
```rust
// Cloud sync interface layout based on examples/ui/flex_layout.rs:25-50
use bevy::prelude::*;

#[derive(Component, Clone, Debug)]
pub struct CloudSyncInterface {
    pub master_sync_enabled: bool,
    pub last_sync_timestamp: Option<DateTime<Utc>>,
    pub sync_in_progress: bool,
    pub selected_categories: HashSet<SyncCategory>,
    pub network_status: NetworkStatus,
}

#[derive(Component, Debug)]
pub struct CloudSyncLeftPanel {
    pub branding_height: f32,
    pub control_height: f32,
    pub status_height: f32,
}

#[derive(Component, Debug)]
pub struct CloudSyncRightPanel {
    pub synced_column_width: f32,
    pub not_synced_column_width: f32,
    pub category_item_height: f32,
}

fn setup_cloud_sync_interface(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Main container - full screen
    commands
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Stretch,
                ..default()
            },
            background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
            ..default()
        })
        .with_children(|parent| {
            // Left Panel - Branding and Master Control (40%)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(40.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(32.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.05, 0.05, 0.05, 1.0).into(),
                    ..default()
                })
                .insert(CloudSyncLeftPanel {
                    branding_height: 120.0,
                    control_height: 80.0,
                    status_height: 60.0,
                })
                .with_children(|left_panel| {
                    // Cloud Sync Branding Section
                    left_panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(120.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::Center,
                                justify_content: JustifyContent::Center,
                                margin: UiRect::bottom(Val::Px(24.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|branding| {
                            // Cloud sync icon
                            branding.spawn(ImageBundle {
                                style: Style {
                                    width: Val::Px(64.0),
                                    height: Val::Px(64.0),
                                    margin: UiRect::bottom(Val::Px(16.0)),
                                    ..default()
                                },
                                image: asset_server.load("icons/cloud_sync.png").into(),
                                ..default()
                            });
                            
                            // Title
                            branding.spawn(TextBundle::from_section(
                                "Cloud Sync",
                                TextStyle {
                                    font: asset_server.load("fonts/Inter-Bold.ttf"),
                                    font_size: 32.0,
                                    color: Color::WHITE,
                                },
                            ));
                        });
                });
            
            // Right Panel - Sync Category Management (60%)
            parent
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(60.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::all(Val::Px(32.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.1, 0.1, 0.1, 1.0).into(),
                    ..default()
                })
                .insert(CloudSyncRightPanel {
                    synced_column_width: 50.0,
                    not_synced_column_width: 50.0,
                    category_item_height: 56.0,
                })
                .with_children(|right_panel| {
                    // Two-column header
                    right_panel
                        .spawn(NodeBundle {
                            style: Style {
                                width: Val::Percent(100.0),
                                height: Val::Px(60.0),
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                margin: UiRect::bottom(Val::Px(24.0)),
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|header| {
                            // Synced column header
                            header.spawn(TextBundle::from_section(
                                "Synced",
                                TextStyle {
                                    font: asset_server.load("fonts/Inter-SemiBold.ttf"),
                                    font_size: 20.0,
                                    color: Color::rgba(0.9, 0.9, 0.9, 1.0),
                                },
                            ));
                        });
                });
        });
}
```

### 2. Master Control Switch System
```rust
// Master toggle switch based on examples/ui/ui_texture_atlas.rs:35-60
#[derive(Component, Debug)]
pub struct MasterSyncToggle {
    pub enabled: bool,
    pub interaction_state: ToggleInteractionState,
    pub animation_progress: f32,
}

#[derive(Debug, PartialEq)]
pub enum ToggleInteractionState {
    Normal,
    Hovered,
    Pressed,
    Disabled,
}

fn setup_master_sync_toggle(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(80.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceBetween,
                padding: UiRect::horizontal(Val::Px(16.0)),
                margin: UiRect::bottom(Val::Px(24.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            background_color: Color::rgba(0.12, 0.12, 0.12, 1.0).into(),
            border_color: Color::rgba(0.3, 0.3, 0.3, 0.5).into(),
            border_radius: BorderRadius::all(Val::Px(12.0)),
            ..default()
        })
        .with_children(|toggle_container| {
            // Toggle label
            toggle_container.spawn(TextBundle::from_section(
                "Enable Cloud Sync",
                TextStyle {
                    font: asset_server.load("fonts/Inter-Medium.ttf"),
                    font_size: 18.0,
                    color: Color::rgba(0.9, 0.9, 0.9, 1.0),
                },
            ));
            
            // Toggle switch
            toggle_container
                .spawn(ButtonBundle {
                    style: Style {
                        width: Val::Px(60.0),
                        height: Val::Px(32.0),
                        border_radius: BorderRadius::all(Val::Px(16.0)),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::FlexStart,
                        padding: UiRect::all(Val::Px(2.0)),
                        ..default()
                    },
                    background_color: Color::rgba(0.0, 0.48, 1.0, 1.0).into(), // iOS blue
                    ..default()
                })
                .insert(MasterSyncToggle {
                    enabled: true,
                    interaction_state: ToggleInteractionState::Normal,
                    animation_progress: 1.0,
                })
                .with_children(|switch| {
                    // Toggle knob
                    switch.spawn(NodeBundle {
                        style: Style {
                            width: Val::Px(28.0),
                            height: Val::Px(28.0),
                            position_type: PositionType::Relative,
                            left: Val::Px(28.0), // Positioned for enabled state
                            ..default()
                        },
                        background_color: Color::WHITE.into(),
                        border_radius: BorderRadius::all(Val::Px(14.0)),
                        ..default()
                    });
                });
        });
}

// Toggle interaction system based on examples/input/mouse_input.rs:45-70
fn handle_master_sync_toggle(
    mut toggle_query: Query<
        (&Interaction, &mut MasterSyncToggle, &mut BackgroundColor),
        (Changed<Interaction>, With<Button>)
    >,
    mut sync_interface: Query<&mut CloudSyncInterface>,
) {
    for (interaction, mut toggle, mut background_color) in toggle_query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                toggle.enabled = !toggle.enabled;
                toggle.interaction_state = ToggleInteractionState::Pressed;
                
                // Update interface state
                if let Ok(mut interface) = sync_interface.get_single_mut() {
                    interface.master_sync_enabled = toggle.enabled;
                }
                
                // Update visual state
                if toggle.enabled {
                    *background_color = Color::rgba(0.0, 0.48, 1.0, 1.0).into(); // iOS blue
                } else {
                    *background_color = Color::rgba(0.3, 0.3, 0.3, 1.0).into(); // Disabled gray
                }
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

### 3. Sync Status Display System
```rust
// Sync status display based on examples/time/time.rs:15-35 and examples/ui/text.rs:25-45
#[derive(Component, Debug)]
pub struct SyncStatusDisplay {
    pub last_update: DateTime<Utc>,
    pub status_text: String,
    pub status_color: Color,
}

fn setup_sync_status_display(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Px(60.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
            ..default()
        })
        .with_children(|status_container| {
            // Status text
            status_container
                .spawn(TextBundle::from_section(
                    "Last Synced Aug 6, 2025 at 6:28 PM",
                    TextStyle {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 14.0,
                        color: Color::rgba(0.7, 0.7, 0.7, 1.0),
                    },
                ))
                .insert(SyncStatusDisplay {
                    last_update: Utc::now(),
                    status_text: "Last Synced Aug 6, 2025 at 6:28 PM".to_string(),
                    status_color: Color::rgba(0.0, 0.8, 0.4, 1.0), // Success green
                });
        });
}

// Real-time status updates based on examples/time/time.rs:50-75
fn update_sync_status_display(
    mut status_query: Query<(&mut Text, &mut SyncStatusDisplay)>,
    sync_interface: Query<&CloudSyncInterface, Changed<CloudSyncInterface>>,
    time: Res<Time>,
) {
    for sync_interface in sync_interface.iter() {
        for (mut text, mut status_display) in status_query.iter_mut() {
            if let Some(last_sync) = sync_interface.last_sync_timestamp {
                let formatted_time = last_sync.format("%b %d, %Y at %I:%M %p").to_string();
                let status_text = if sync_interface.sync_in_progress {
                    "Syncing...".to_string()
                } else {
                    format!("Last Synced {}", formatted_time)
                };
                
                status_display.status_text = status_text.clone();
                text.sections[0].value = status_text;
                
                // Update color based on sync status
                if sync_interface.sync_in_progress {
                    text.sections[0].style.color = Color::rgba(0.0, 0.48, 1.0, 1.0); // Syncing blue
                } else {
                    text.sections[0].style.color = Color::rgba(0.0, 0.8, 0.4, 1.0); // Success green
                }
            }
        }
    }
}
```

### 4. Two-Column Category Layout
```rust
// Two-column layout system based on examples/ui/ui.rs:85-110
#[derive(Component, Debug)]
pub struct SyncCategoryColumn {
    pub column_type: CategoryColumnType,
    pub category_count: usize,
    pub scroll_position: f32,
}

#[derive(Debug, PartialEq)]
pub enum CategoryColumnType {
    Synced,
    NotSynced,
}

fn setup_sync_category_columns(
    parent: &mut ChildBuilder,
    asset_server: &Res<AssetServer>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(85.0), // Remaining space after header
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(24.0),
                ..default()
            },
            ..default()
        })
        .with_children(|columns_container| {
            // Synced Categories Column
            columns_container
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                })
                .insert(SyncCategoryColumn {
                    column_type: CategoryColumnType::Synced,
                    category_count: 10,
                    scroll_position: 0.0,
                })
                .with_children(|synced_column| {
                    // Column header
                    synced_column.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border_bottom: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        border_color: Color::rgba(0.3, 0.3, 0.3, 0.3).into(),
                        ..default()
                    })
                    .with_children(|header| {
                        header.spawn(TextBundle::from_section(
                            "Synced",
                            TextStyle {
                                font: asset_server.load("fonts/Inter-SemiBold.ttf"),
                                font_size: 16.0,
                                color: Color::rgba(0.0, 0.8, 0.4, 1.0), // Success green
                            },
                        ));
                    });
                });
            
            // Not Synced Categories Column
            columns_container
                .spawn(NodeBundle {
                    style: Style {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    },
                    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    ..default()
                })
                .insert(SyncCategoryColumn {
                    column_type: CategoryColumnType::NotSynced,
                    category_count: 4,
                    scroll_position: 0.0,
                })
                .with_children(|not_synced_column| {
                    // Column header
                    not_synced_column.spawn(NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Px(50.0),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            border_bottom: UiRect::bottom(Val::Px(1.0)),
                            ..default()
                        },
                        border_color: Color::rgba(0.3, 0.3, 0.3, 0.3).into(),
                        ..default()
                    })
                    .with_children(|header| {
                        header.spawn(TextBundle::from_section(
                            "Not Synced",
                            TextStyle {
                                font: asset_server.load("fonts/Inter-SemiBold.ttf"),
                                font_size: 16.0,
                                color: Color::rgba(0.8, 0.4, 0.0, 1.0), // Warning orange
                            },
                        ));
                    });
                });
        });
}
```

## Bevy Example References
- **Flex layouts**: `examples/ui/flex_layout.rs:25-50` - Split panel layout with percentage widths
- **UI interactions**: `examples/input/mouse_input.rs:45-70` - Toggle switch interaction handling
- **Time formatting**: `examples/time/time.rs:15-35` - Real-time timestamp display
- **Dynamic text**: `examples/ui/text.rs:25-45` - Status text updates
- **Texture atlases**: `examples/ui/ui_texture_atlas.rs:35-60` - Toggle switch states and icons
- **Multi-column UI**: `examples/ui/ui.rs:85-110` - Two-column category layout system

## Architecture Integration Notes
- **File**: `ui/src/cloud_sync/interface.rs:1-400`
- **Dependencies**: Cloud sync engine, network status monitoring, category management
- **Integration**: Settings persistence, user preferences, sync state management
- **Performance**: Optimized rendering with component-based architecture

## Success Criteria
1. **Split-panel layout** with exact 40/60 proportions maintained across screen sizes
2. **Responsive master toggle** with smooth iOS-style animation transitions
3. **Real-time sync status** updates within 500ms of sync state changes
4. **Two-column category display** with proper spacing and visual hierarchy
5. **Smooth interactions** with hover states and click feedback for all controls
6. **Consistent visual theming** following the launcher's design system
7. **Accessibility support** with proper focus management and keyboard navigation