# TASK7.7: Settings Panel - Advanced

**Status**: Not Started  
**Estimated Time**: 6-8 hours  
**Priority**: High  
**Dependencies**: TASK7.0-INFRASTRUCTURE.md, TASK7.C-COMPONENTS.md

---

## Objective

Implement the Advanced settings panel with power-user configuration options including window positioning behavior, keyboard navigation, input source management, search sensitivity tuning, and hyper key configuration. This panel provides fine-grained control over application behavior for advanced users who want to customize every aspect of the launcher's operation.

---

## Dependencies

**MUST complete first:**
1. ✅ TASK7.0-INFRASTRUCTURE.md - Settings modal, tabs, entity pre-allocation
2. ✅ TASK7.C-COMPONENTS.md - Toggle, Dropdown, Slider components

**Required systems:**
- `SettingsUIEntities` resource (from TASK7.0)
- `SettingControl` component (from TASK7.C)
- Form control spawning functions (from TASK7.C)
- Event handlers for database I/O (from TASK7.0)

---

## Screenshot Reference

![Advanced Menu](/Volumes/samsung_t9/action-items/spec/screenshots/Advanced_Menu.png)

**Visual Structure:**

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│ Show Raycast on      [Screen containing mouse    ▼]   │
│                                                         │
│ Pop to Root Search   [After 90 seconds           ▼] ℹ️│
│                                                         │
│ Escape Key Behavior  [Navigate back or close     ▼] ℹ️│
│                                                         │
│ Auto-switch Input    [U.S.                       ▼] ℹ️│
│ Source                                                  │
│                                                         │
│ Navigation Bindings  [macOS Standard (^N,^P...)  ▼] ℹ️│
│                                                         │
│ Page Navigation Keys [Square Brackets            ▼] ℹ️│
│                                                         │
│ Root Search          [─────────●──────────────]     ℹ️│
│ Sensitivity          Low      Medium        High      │
│                                                         │
│ ──────────────────────────────────────────────────────│
│                                                         │
│ Hyper Key            [-                          ▼] ℹ️│
│                                                         │
│ ☑ Replace occurrences of ^ \  ⌥ ⌘ with ↑             │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## Database Schema

### Table: `advanced_settings`

```sql
DEFINE TABLE advanced_settings SCHEMALESS;

-- Window Positioning
DEFINE FIELD show_on_screen ON TABLE advanced_settings TYPE string DEFAULT "screen_with_mouse";
  -- Values: "screen_with_mouse", "active_screen", "primary_screen"

-- Navigation Behavior
DEFINE FIELD pop_to_root_after ON TABLE advanced_settings TYPE string DEFAULT "90_seconds";
  -- Values: "never", "5_seconds", "30_seconds", "90_seconds", "5_minutes"

DEFINE FIELD escape_key_behavior ON TABLE advanced_settings TYPE string DEFAULT "navigate_back";
  -- Values: "navigate_back", "close_window", "minimize"

-- Input Source Management
DEFINE FIELD auto_switch_input_source ON TABLE advanced_settings TYPE string DEFAULT "us";
  -- Input source identifier (e.g., "com.apple.keylayout.US")

DEFINE FIELD input_source_enabled ON TABLE advanced_settings TYPE bool DEFAULT false;

-- Keyboard Navigation
DEFINE FIELD navigation_bindings ON TABLE advanced_settings TYPE string DEFAULT "macos_standard";
  -- Values: "macos_standard", "vim", "emacs", "custom"

DEFINE FIELD page_navigation_keys ON TABLE advanced_settings TYPE string DEFAULT "square_brackets";
  -- Values: "square_brackets", "curly_brackets", "angle_brackets", "disabled"

-- Search Behavior
DEFINE FIELD root_search_sensitivity ON TABLE advanced_settings TYPE float DEFAULT 0.5;
  -- Range: 0.0 (low) to 1.0 (high)

-- Hyper Key
DEFINE FIELD hyper_key_enabled ON TABLE advanced_settings TYPE bool DEFAULT false;
DEFINE FIELD hyper_key_mapping ON TABLE advanced_settings TYPE string;
  -- Values: "caps_lock", "right_option", "right_command", "f19", "disabled"

-- Symbol Display
DEFINE FIELD use_arrow_for_modifiers ON TABLE advanced_settings TYPE bool DEFAULT false;
  -- Replace ^ \ ⌥ ⌘ with ↑ when displaying
```

---

## Component Structure

### Components

```rust
use bevy::prelude::*;

/// Marker component for the Advanced panel root entity
#[derive(Component, Debug)]
pub struct AdvancedPanel;

/// Component for info icon buttons
#[derive(Component, Debug)]
pub struct AdvancedInfoButton {
    pub setting_id: String,
    pub info_text: String,
}

/// Component for Root Search Sensitivity slider
#[derive(Component, Debug)]
pub struct SearchSensitivitySlider {
    pub value: f32, // 0.0 to 1.0
}

impl SearchSensitivitySlider {
    pub fn label(&self) -> &'static str {
        if self.value < 0.33 {
            "Low"
        } else if self.value < 0.67 {
            "Medium"
        } else {
            "High"
        }
    }
}
```

### Resources

```rust
/// Resource tracking all entities in the Advanced panel
#[derive(Resource)]
pub struct AdvancedPanelEntities {
    pub panel_root: Entity,
    
    // Window positioning
    pub show_on_screen_dropdown: Entity,
    
    // Navigation behavior
    pub pop_to_root_dropdown: Entity,
    pub pop_to_root_info: Entity,
    pub escape_behavior_dropdown: Entity,
    pub escape_behavior_info: Entity,
    
    // Input source
    pub input_source_dropdown: Entity,
    pub input_source_info: Entity,
    
    // Keyboard navigation
    pub navigation_bindings_dropdown: Entity,
    pub navigation_bindings_info: Entity,
    pub page_nav_keys_dropdown: Entity,
    pub page_nav_keys_info: Entity,
    
    // Search sensitivity
    pub search_sensitivity_slider: Entity,
    pub search_sensitivity_info: Entity,
    pub sensitivity_label: Entity,
    
    // Hyper key
    pub hyper_key_dropdown: Entity,
    pub hyper_key_info: Entity,
    pub arrow_modifiers_toggle: Entity,
}
```

### Events

```rust
/// Event sent when user clicks info button
#[derive(Event, Debug)]
pub struct AdvancedSettingInfoRequested {
    pub setting_id: String,
    pub info_text: String,
}

/// Event sent when search sensitivity changes
#[derive(Event, Debug)]
pub struct SearchSensitivityChanged {
    pub value: f32,
}
```

---

## Implementation Details

### System 1: Setup Advanced Panel Entities

**Purpose**: Pre-allocate all Advanced panel UI entities during initialization

```rust
pub fn setup_advanced_panel(
    mut commands: Commands,
    settings_entities: Res<SettingsUIEntities>,
    asset_server: Res<AssetServer>,
) {
    let content_area = settings_entities.content_area;
    
    // Create panel root
    let panel_root = commands.spawn((
        AdvancedPanel,
        UiLayout::window()
            .size((Rl(100.0), Rl(100.0)))
            .pos((Rl(0.0), Rl(0.0)))
            .pack(),
        Visibility::Hidden,
        Name::new("AdvancedPanel"),
    )).id();
    
    commands.entity(content_area).add_child(panel_root);
    
    let mut y_offset = 30.0;
    
    // ═══════════════════════════════════════════════════════════
    // SHOW RAYCAST ON
    // ═══════════════════════════════════════════════════════════
    
    let (show_on_label, show_on_screen_dropdown) = create_labeled_dropdown(
        &mut commands,
        "Show Action Items on",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "show_on_screen".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Screen containing mouse".to_string(),
                    "Active screen".to_string(),
                    "Primary screen".to_string(),
                ],
            },
        },
        "Screen containing mouse",
        y_offset,
        None, // No info button
    );
    
    commands.entity(panel_root).push_children(&[show_on_label, show_on_screen_dropdown]);
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // POP TO ROOT SEARCH
    // ═══════════════════════════════════════════════════════════
    
    let (pop_root_label, pop_to_root_dropdown, pop_to_root_info) = create_labeled_dropdown(
        &mut commands,
        "Pop to Root Search",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "pop_to_root_after".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Never".to_string(),
                    "After 5 seconds".to_string(),
                    "After 30 seconds".to_string(),
                    "After 90 seconds".to_string(),
                    "After 5 minutes".to_string(),
                ],
            },
        },
        "After 90 seconds",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "pop_to_root".to_string(),
            info_text: "Automatically return to the root search screen after the specified time of inactivity.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[pop_root_label, pop_to_root_dropdown]);
    if let Some(info) = pop_to_root_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // ESCAPE KEY BEHAVIOR
    // ═══════════════════════════════════════════════════════════
    
    let (escape_label, escape_behavior_dropdown, escape_behavior_info) = create_labeled_dropdown(
        &mut commands,
        "Escape Key Behavior",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "escape_key_behavior".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Navigate back or close window".to_string(),
                    "Always close window".to_string(),
                    "Minimize window".to_string(),
                ],
            },
        },
        "Navigate back or close window",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "escape_behavior".to_string(),
            info_text: "Control what happens when you press the Escape key: navigate back through search, close the window immediately, or minimize.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[escape_label, escape_behavior_dropdown]);
    if let Some(info) = escape_behavior_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // AUTO-SWITCH INPUT SOURCE
    // ═══════════════════════════════════════════════════════════
    
    let (input_source_label, input_source_dropdown, input_source_info) = create_labeled_dropdown(
        &mut commands,
        "Auto-switch Input Source",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "auto_switch_input_source".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Disabled".to_string(),
                    "U.S.".to_string(),
                    "ABC".to_string(),
                    "Pinyin".to_string(),
                    "Hiragana".to_string(),
                ],
            },
        },
        "U.S.",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "input_source".to_string(),
            info_text: "Automatically switch to the selected keyboard input source when Action Items opens. Useful for multilingual users.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[input_source_label, input_source_dropdown]);
    if let Some(info) = input_source_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // NAVIGATION BINDINGS
    // ═══════════════════════════════════════════════════════════
    
    let (nav_bindings_label, navigation_bindings_dropdown, navigation_bindings_info) = create_labeled_dropdown(
        &mut commands,
        "Navigation Bindings",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "navigation_bindings".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "macOS Standard (^N, ^P, ^F, ^B)".to_string(),
                    "Vim (j, k, h, l)".to_string(),
                    "Emacs (C-n, C-p, C-f, C-b)".to_string(),
                ],
            },
        },
        "macOS Standard (^N, ^P, ^F, ^B)",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "navigation_bindings".to_string(),
            info_text: "Choose keyboard shortcuts for navigating through search results: macOS standard, Vim-style, or Emacs-style bindings.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[nav_bindings_label, navigation_bindings_dropdown]);
    if let Some(info) = navigation_bindings_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // PAGE NAVIGATION KEYS
    // ═══════════════════════════════════════════════════════════
    
    let (page_nav_label, page_nav_keys_dropdown, page_nav_keys_info) = create_labeled_dropdown(
        &mut commands,
        "Page Navigation Keys",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "page_navigation_keys".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "Square Brackets [ ]".to_string(),
                    "Curly Brackets { }".to_string(),
                    "Angle Brackets < >".to_string(),
                    "Disabled".to_string(),
                ],
            },
        },
        "Square Brackets",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "page_navigation".to_string(),
            info_text: "Select which keys to use for quick page navigation through long lists of results.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[page_nav_label, page_nav_keys_dropdown]);
    if let Some(info) = page_nav_keys_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // ROOT SEARCH SENSITIVITY SLIDER
    // ═══════════════════════════════════════════════════════════
    
    let sensitivity_label = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new("Root Search Sensitivity"),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        Name::new("SensitivityLabel"),
    )).id();
    
    // Slider container
    let slider_container = commands.spawn((
        UiLayout::window()
            .size((Rl(45.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
        Name::new("SliderContainer"),
    )).id();
    
    // Slider track
    let slider_track = commands.spawn((
        UiLayout::window()
            .size((Rl(100.0), Ab(4.0)))
            .pos((Rl(0.0), Ab(18.0)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
        Name::new("SliderTrack"),
    )).id();
    
    // Slider handle
    let search_sensitivity_slider = commands.spawn((
        SearchSensitivitySlider { value: 0.5 },
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "root_search_sensitivity".to_string(),
            control_type: ControlType::Slider {
                min: 0.0,
                max: 1.0,
                step: 0.01,
            },
        },
        UiLayout::window()
            .size((Ab(20.0), Ab(20.0)))
            .pos((Rl(50.0), Ab(10.0)))
            .anchor(Anchor::CenterLeft)
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.7, 1.0, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Pickable::default(),
        Interaction::None,
        Name::new("SliderHandle"),
    )).id();
    
    commands.entity(slider_container).push_children(&[slider_track, search_sensitivity_slider]);
    
    // Slider labels (Low, Medium, High)
    let low_label = commands.spawn((
        UiLayout::window()
            .size((Ab(40.0), Ab(20.0)))
            .pos((Rl(0.0), Ab(30.0)))
            .pack(),
        Text::new("Low"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    
    let medium_label = commands.spawn((
        UiLayout::window()
            .size((Ab(60.0), Ab(20.0)))
            .pos((Rl(50.0), Ab(30.0)))
            .anchor(Anchor::TopCenter)
            .pack(),
        Text::new("Medium"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    
    let high_label = commands.spawn((
        UiLayout::window()
            .size((Ab(40.0), Ab(20.0)))
            .pos((Rl(100.0), Ab(30.0)))
            .anchor(Anchor::TopRight)
            .pack(),
        Text::new("High"),
        UiTextSize::from(Em(0.85)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
    )).id();
    
    commands.entity(slider_container).push_children(&[low_label, medium_label, high_label]);
    
    // Info button for sensitivity
    let search_sensitivity_info = commands.spawn((
        AdvancedInfoButton {
            setting_id: "search_sensitivity".to_string(),
            info_text: "Adjust how strict the search matching is. Low sensitivity shows more results (fuzzy matching), high sensitivity shows only exact matches.".to_string(),
        },
        UiLayout::window()
            .size((Ab(24.0), Ab(24.0)))
            .pos((Rl(90.0), Ab(y_offset + 3.0)))
            .pack(),
        UiColor::from(Color::srgba(0.4, 0.4, 0.45, 1.0)),
        UiHover::new(),
        UiClicked::new(),
        Text::new("ℹ️"),
        UiTextSize::from(Em(0.9)),
        Pickable::default(),
        Interaction::None,
    )).id();
    
    commands.entity(panel_root).push_children(&[
        sensitivity_label,
        slider_container,
        search_sensitivity_info,
    ]);
    
    y_offset += 90.0;
    
    // Separator line
    let separator = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(1.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.3, 0.35, 1.0)),
    )).id();
    
    commands.entity(panel_root).add_child(separator);
    y_offset += 40.0;
    
    // ═══════════════════════════════════════════════════════════
    // HYPER KEY
    // ═══════════════════════════════════════════════════════════
    
    let (hyper_key_label, hyper_key_dropdown, hyper_key_info) = create_labeled_dropdown(
        &mut commands,
        "Hyper Key",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "hyper_key_mapping".to_string(),
            control_type: ControlType::Dropdown {
                options: vec![
                    "-".to_string(), // Disabled
                    "Caps Lock".to_string(),
                    "Right Option".to_string(),
                    "Right Command".to_string(),
                    "F19".to_string(),
                ],
            },
        },
        "-",
        y_offset,
        Some(AdvancedInfoButton {
            setting_id: "hyper_key".to_string(),
            info_text: "Map a physical key to act as all modifiers simultaneously (⌃⇧⌥⌘). Useful for creating unique global shortcuts.".to_string(),
        }),
    );
    
    commands.entity(panel_root).push_children(&[hyper_key_label, hyper_key_dropdown]);
    if let Some(info) = hyper_key_info {
        commands.entity(panel_root).add_child(info);
    }
    y_offset += 60.0;
    
    // ═══════════════════════════════════════════════════════════
    // ARROW MODIFIERS TOGGLE
    // ═══════════════════════════════════════════════════════════
    
    let arrow_modifiers_toggle = create_labeled_toggle(
        &mut commands,
        "Replace occurrences of ^ \\ ⌥ ⌘ with ↑",
        SettingControl {
            table: "advanced_settings".to_string(),
            field: "use_arrow_for_modifiers".to_string(),
            control_type: ControlType::Toggle,
        },
        false,
        y_offset,
    );
    
    commands.entity(panel_root).add_child(arrow_modifiers_toggle);
    
    // Store entities in resource
    commands.insert_resource(AdvancedPanelEntities {
        panel_root,
        show_on_screen_dropdown,
        pop_to_root_dropdown,
        pop_to_root_info,
        escape_behavior_dropdown,
        escape_behavior_info,
        input_source_dropdown,
        input_source_info,
        navigation_bindings_dropdown,
        navigation_bindings_info,
        page_nav_keys_dropdown,
        page_nav_keys_info,
        search_sensitivity_slider,
        search_sensitivity_info,
        sensitivity_label,
        hyper_key_dropdown,
        hyper_key_info,
        arrow_modifiers_toggle,
    });
    
    info!("✅ Pre-allocated Advanced panel UI entities");
}

// Helper functions

fn create_labeled_dropdown(
    commands: &mut Commands,
    label: &str,
    control: SettingControl,
    default_value: &str,
    y_offset: f32,
    info_button: Option<AdvancedInfoButton>,
) -> (Entity, Entity, Option<Entity>) {
    let label_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(30.0), Ab(30.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let dropdown = spawn_dropdown_menu(commands, control, default_value.to_string());
    
    let dropdown_container = commands.spawn((
        UiLayout::window()
            .size((Rl(48.0), Ab(40.0)))
            .pos((Rl(40.0), Ab(y_offset - 5.0)))
            .pack(),
    )).id();
    
    commands.entity(dropdown_container).add_child(dropdown);
    
    // Info button if provided
    let info_entity = info_button.map(|info| {
        commands.spawn((
            info,
            UiLayout::window()
                .size((Ab(24.0), Ab(24.0)))
                .pos((Rl(90.0), Ab(y_offset + 3.0)))
                .pack(),
            UiColor::from(Color::srgba(0.4, 0.4, 0.45, 1.0)),
            UiHover::new(),
            UiClicked::new(),
            Text::new("ℹ️"),
            UiTextSize::from(Em(0.9)),
            Pickable::default(),
            Interaction::None,
        )).id()
    });
    
    (label_entity, dropdown_container, info_entity)
}

fn create_labeled_toggle(
    commands: &mut Commands,
    label: &str,
    control: SettingControl,
    default_value: bool,
    y_offset: f32,
) -> Entity {
    let container = commands.spawn((
        UiLayout::window()
            .size((Rl(90.0), Ab(35.0)))
            .pos((Rl(5.0), Ab(y_offset)))
            .pack(),
    )).id();
    
    let label_entity = commands.spawn((
        UiLayout::window()
            .size((Rl(80.0), Ab(30.0)))
            .pos((Rl(0.0), Ab(5.0)))
            .pack(),
        Text::new(label),
        UiTextSize::from(Em(1.0)),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
    )).id();
    
    let toggle = spawn_toggle_switch(commands, control, default_value);
    
    let toggle_container = commands.spawn((
        UiLayout::window()
            .size((Ab(44.0), Ab(24.0)))
            .pos((Rl(0.0), Ab(10.0)))
            .pack(),
    )).id();
    
    commands.entity(toggle_container).add_child(toggle);
    commands.entity(container).push_children(&[label_entity, toggle_container]);
    
    container
}
```

### System 2: Load Advanced Settings

**Purpose**: Load advanced settings from database when panel visible

```rust
pub fn load_advanced_settings(
    mut panel_query: Query<&Visibility, (With<AdvancedPanel>, Changed<Visibility>)>,
    mut read_events: EventWriter<SettingsReadRequested>,
    panel_entities: Res<AdvancedPanelEntities>,
) {
    for visibility in panel_query.iter() {
        if *visibility == Visibility::Visible {
            // Load advanced settings
            read_events.send(SettingsReadRequested {
                operation_id: Uuid::new_v4(),
                table: "advanced_settings".to_string(),
                query: "SELECT * FROM advanced_settings LIMIT 1".to_string(),
                requester: panel_entities.panel_root,
            });
            
            info!("📖 Loading Advanced panel settings from database");
        }
    }
}
```

### System 3: Handle Search Sensitivity Slider

**Purpose**: Update slider value when user drags handle

```rust
pub fn handle_search_sensitivity_slider(
    mut slider_query: Query<
        (&mut SearchSensitivitySlider, &Interaction, &mut UiLayout),
        Changed<Interaction>
    >,
    mut sensitivity_events: EventWriter<SearchSensitivityChanged>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
) {
    for (mut slider, interaction, mut layout) in slider_query.iter_mut() {
        if *interaction == Interaction::Pressed && mouse_input.pressed(MouseButton::Left) {
            // Get mouse position relative to slider track
            if let Some(window) = windows.iter().next() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Calculate new slider value based on cursor position
                    // This is simplified - actual implementation would need proper coordinate mapping
                    let new_value = (cursor_pos.x / window.width()).clamp(0.0, 1.0);
                    
                    if (slider.value - new_value).abs() > 0.01 {
                        slider.value = new_value;
                        
                        // Update slider handle position
                        // layout.pos = /* calculate new position based on value */;
                        
                        sensitivity_events.send(SearchSensitivityChanged {
                            value: new_value,
                        });
                        
                        info!("🔍 Search sensitivity changed: {:.2} ({})", 
                              new_value, 
                              slider.label());
                    }
                }
            }
        }
    }
}
```

### System 4: Save Sensitivity Changes

**Purpose**: Save slider changes to database

```rust
pub fn save_sensitivity_changes(
    mut sensitivity_events: EventReader<SearchSensitivityChanged>,
    mut write_events: EventWriter<SettingsWriteRequested>,
) {
    for event in sensitivity_events.read() {
        write_events.send(SettingsWriteRequested {
            operation_id: Uuid::new_v4(),
            table: "advanced_settings".to_string(),
            field: "root_search_sensitivity".to_string(),
            value: json!(event.value),
        });
    }
}
```

### System 5: Handle Info Buttons

**Purpose**: Show tooltips when info icons clicked

```rust
pub fn handle_advanced_info_buttons(
    buttons: Query<
        (&AdvancedInfoButton, &Interaction, &UiClicked),
        Changed<Interaction>
    >,
    mut info_events: EventWriter<AdvancedSettingInfoRequested>,
) {
    for (button, interaction, clicked) in buttons.iter() {
        if *interaction == Interaction::Pressed && clicked.progress > 0.9 {
            info_events.send(AdvancedSettingInfoRequested {
                setting_id: button.setting_id.clone(),
                info_text: button.info_text.clone(),
            });
            
            info!("ℹ️ Info requested for: {}", button.setting_id);
            
            // TODO: Show tooltip or modal with info_text
        }
    }
}
```

---

## Plugin Definition

```rust
pub struct AdvancedPanelPlugin;

impl Plugin for AdvancedPanelPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_event::<AdvancedSettingInfoRequested>()
            .add_event::<SearchSensitivityChanged>()
            .add_systems(Startup, setup_advanced_panel)
            .add_systems(Update, (
                load_advanced_settings,
                handle_search_sensitivity_slider,
                save_sensitivity_changes,
                handle_advanced_info_buttons,
            ).chain());
    }
}
```

---

## Acceptance Criteria

1. ✅ All dropdown controls render and save correctly
2. ✅ Info buttons show helpful explanations
3. ✅ Search sensitivity slider works with Low/Medium/High labels
4. ✅ Hyper key configuration dropdown functional
5. ✅ Arrow modifiers toggle saves preference
6. ✅ All settings load from database on panel open
7. ✅ All settings save to database on change
8. ✅ Performance targets met (load < 50ms, interactions < 16ms)
9. ✅ NO STUBS in implementation
10. ✅ Tests pass with 100% success
11. ✅ Follows architecture patterns from TASK7.0 and TASK7.C

---

## Implementation Notes

**DO NOT:**
- ❌ Apply settings changes without user confirmation for destructive options
- ❌ Hardcode keyboard layout options

**DO:**
- ✅ Detect available input sources from system
- ✅ Validate sensitivity values (0.0-1.0 range)
- ✅ Show visual feedback during slider interaction
- ✅ Apply settings immediately (no "Apply" button needed)

---

## Estimated Time Breakdown

- UI setup with all controls: 2 hours
- Slider implementation with interaction: 2 hours
- Dropdown controls and info buttons: 1.5 hours
- Database integration: 1 hour
- Testing and polish: 1.5 hours

**Total: 6-8 hours**

**Ready for code review** ✅
