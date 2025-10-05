# TASK7.C: Reusable Form Control Library

**Status:** 🔴 NOT STARTED  
**Estimated Effort:** 10-12 hours  
**Priority:** HIGH (All panels depend on these controls)  

## Dependencies

**Blocked By:**
- **TASK-0**: User Settings Plugin (must be complete)
- **TASK7.0**: Settings Infrastructure (must be complete)

**Enables:**
- TASK7.1: General Settings Panel
- TASK7.2: Extensions Panel
- TASK7.3: AI Settings Panel
- TASK7.4: Cloud Sync Panel
- TASK7.5: Account Settings Panel
- TASK7.6: Organizations Panel
- TASK7.7: Advanced Settings Panel
- TASK7.8: About Panel

## Objective

Build a comprehensive library of production-quality, reusable form controls that all settings panels will use. Each control must be fully functional, animated, accessible, and integrated with the database event system.

**What this includes:**
- 10 reusable form control types
- Spawn helper functions for each control
- Update systems (for database responses)
- Interaction systems (for user input)
- Visual states (idle, hover, active, disabled)
- Smooth animations
- Database integration

**Design principles:**
- **Reusable**: Easy to spawn with a single function call
- **Composable**: Controls can be combined in layouts
- **Type-safe**: Strong typing for control values
- **Reactive**: Automatically emit events on change
- **Performant**: Smooth 60fps animations
- **Accessible**: Keyboard navigation support

## Acceptance Criteria

### Toggle Switch
- [ ] Renders as track + animated handle
- [ ] Smooth animation between on/off states
- [ ] Responds to click to toggle
- [ ] Visual states: idle, hover, active, disabled
- [ ] Emits `SettingsWriteRequested` on change
- [ ] Updates from database response
- [ ] Keyboard support (Space/Enter to toggle)

### Dropdown Menu
- [ ] Renders as button with current value + arrow
- [ ] Opens menu on click with all options
- [ ] Closes menu on selection or outside click
- [ ] Keyboard navigation (arrows, Enter, Escape)
- [ ] Search/filter when typing (optional)
- [ ] Emits `SettingsWriteRequested` on selection
- [ ] Updates from database response

### Slider
- [ ] Renders as track with draggable handle
- [ ] Snap to positions or continuous
- [ ] Labeled positions (Low/Medium/High)
- [ ] Responds to drag and click
- [ ] Keyboard support (Arrow keys)
- [ ] Emits `SettingsWriteRequested` on change
- [ ] Updates from database response

### Hotkey Recorder
- [ ] Displays current hotkey
- [ ] Enters recording mode on click
- [ ] Captures key combination
- [ ] Shows visual feedback during recording
- [ ] Detects conflicts with system/existing hotkeys
- [ ] Cancels on Escape
- [ ] Emits `SettingsWriteRequested` on save
- [ ] Updates from database response

### Text Input
- [ ] Single-line text input
- [ ] Cursor and selection support
- [ ] Validation (min/max length, regex)
- [ ] Error state display
- [ ] Placeholder text
- [ ] Emits `SettingsWriteRequested` on blur/Enter
- [ ] Updates from database response

### Radio Group
- [ ] Renders multiple exclusive options
- [ ] Visual previews (for window mode)
- [ ] Single selection enforcement
- [ ] Keyboard navigation
- [ ] Emits `SettingsWriteRequested` on selection
- [ ] Updates from database response

### Button Group
- [ ] Renders multiple buttons (for text size)
- [ ] Single selection highlight
- [ ] Visual feedback on hover/click
- [ ] Keyboard navigation
- [ ] Emits `SettingsWriteRequested` on selection
- [ ] Updates from database response

### Checkbox
- [ ] Simple checkbox with label
- [ ] Checked/unchecked states
- [ ] Visual feedback
- [ ] Keyboard support (Space)
- [ ] Emits `SettingsWriteRequested` on change
- [ ] Updates from database response

### Action Button
- [ ] Standard clickable button
- [ ] Visual states: idle, hover, active, disabled
- [ ] Icon + text support
- [ ] Emits custom event on click
- [ ] Animated interactions

### Info Icon
- [ ] Small icon button
- [ ] Shows tooltip on hover
- [ ] Tooltip positioned intelligently
- [ ] Dismisses on click outside

## Component Definitions

```rust
use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use serde_json::Value;

/// Component identifying a setting control
#[derive(Component, Debug, Clone)]
pub struct SettingControl {
    /// Database table this control modifies
    pub table: String,
    /// Database field this control modifies
    pub field: String,
    /// Type of control (determines behavior)
    pub control_type: ControlType,
}

/// Control type variants
#[derive(Debug, Clone)]
pub enum ControlType {
    Toggle,
    Dropdown { options: Vec<String> },
    Slider { min: f32, max: f32, step: f32 },
    HotkeyRecorder,
    TextInput { validation: Option<InputValidation> },
    RadioGroup { options: Vec<String> },
    ButtonGroup { options: Vec<String> },
    Checkbox,
}

/// Input validation rules
#[derive(Debug, Clone)]
pub struct InputValidation {
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub regex: Option<String>,
    pub error_message: String,
}

/// Current value of a control (for updates from database)
#[derive(Component, Debug)]
pub struct ControlValue {
    pub value: Value,
}

/// Slider-specific state
#[derive(Component, Debug)]
pub struct SliderValue {
    pub current: f32,
    pub min: f32,
    pub max: f32,
}

/// Dropdown-specific state
#[derive(Component, Debug)]
pub struct DropdownOptions {
    pub options: Vec<String>,
    pub selected_index: usize,
}

/// Dropdown menu entity (spawned when dropdown opens)
#[derive(Component)]
pub struct DropdownMenu {
    pub parent_dropdown: Entity,
}

/// Hotkey recorder state
#[derive(Component, Debug)]
pub struct HotkeyRecorderState {
    pub is_recording: bool,
    pub current_modifiers: Vec<String>,
    pub current_key: Option<String>,
}

/// Info tooltip component
#[derive(Component)]
pub struct InfoTooltip {
    pub text: String,
    pub visible: bool,
}
```

## Control Implementation: Toggle Switch

```rust
/// Spawn a toggle switch control
pub fn spawn_toggle_switch(
    commands: &mut Commands,
    setting_control: SettingControl,
    initial_value: bool,
) -> Entity {
    // Track (background)
    let track = commands.spawn((
        UiLayout::window()
            .size((Ab(44.0), Ab(24.0)))
            .pack(),
        UiColor::from(if initial_value {
            Color::srgba(0.3, 0.5, 0.8, 1.0)  // Blue when on
        } else {
            Color::srgba(0.25, 0.25, 0.28, 1.0)  // Gray when off
        }),
        Name::new("ToggleTrack"),
    )).id();

    // Handle (animated circle)
    let handle = commands.spawn((
        setting_control,
        ControlValue { value: json!(initial_value) },
        UiLayout::window()
            .size((Ab(20.0), Ab(20.0)))
            .pos((Ab(if initial_value { 22.0 } else { 2.0 }), Ab(2.0)))
            .pack(),
        UiColor::from(Color::srgba(0.9, 0.9, 0.95, 1.0)),
        UiClicked::new()
            .forward_speed(15.0)
            .backward_speed(12.0),
        UiHover::new()
            .forward_speed(10.0),
        Pickable::default(),
        Interaction::None,
        Name::new("ToggleHandle"),
    )).id();
    commands.entity(handle).set_parent(track);

    handle
}

/// System: Handle toggle switch clicks
pub fn handle_toggle_clicks(
    mut toggle_query: Query<
        (
            Entity,
            &SettingControl,
            &Interaction,
            &mut ControlValue,
            &mut UiLayout,
        ),
        (Changed<Interaction>, With<UiClicked>),
    >,
    mut track_query: Query<&mut UiColor>,
    mut write_events: EventWriter<SettingsWriteRequested>,
    mut change_events: EventWriter<SettingChanged>,
) {
    for (entity, control, interaction, mut value, mut layout) in toggle_query.iter_mut() {
        if matches!(control.control_type, ControlType::Toggle) {
            if *interaction == Interaction::Pressed {
                // Toggle value
                let old_value = value.value.clone();
                let new_value = !value.value.as_bool().unwrap_or(false);
                value.value = json!(new_value);

                // Update position (2.0 = off, 22.0 = on)
                if let Some(pos) = layout.pos.as_mut() {
                    pos.0 = if new_value { Ab(22.0) } else { Ab(2.0) };
                }

                // Update track color
                if let Ok(parent) = track_query.get_mut(/* get parent track */) {
                    // Update color logic
                }

                // Emit database write event
                write_events.send(SettingsWriteRequested {
                    operation_id: Uuid::new_v4(),
                    table: control.table.clone(),
                    field: control.field.clone(),
                    value: json!(new_value),
                    requester: entity,
                });

                // Emit change event
                change_events.send(SettingChanged {
                    table: control.table.clone(),
                    field: control.field.clone(),
                    old_value: Some(old_value),
                    new_value: json!(new_value),
                });
            }
        }
    }
}

/// System: Update toggles from database responses
pub fn update_toggles_from_database(
    mut response_events: EventReader<SettingsReadResponse>,
    mut toggle_query: Query<(
        &SettingControl,
        &mut ControlValue,
        &mut UiLayout,
    )>,
) {
    for response in response_events.read() {
        if !response.success {
            continue;
        }

        let data = response.data.as_ref().unwrap();
        if let Some(rows) = data.as_array() {
            if let Some(row) = rows.first() {
                for (control, mut value, mut layout) in toggle_query.iter_mut() {
                    if matches!(control.control_type, ControlType::Toggle) 
                        && control.table == response.table {
                        
                        if let Some(field_value) = row.get(&control.field) {
                            if let Some(bool_value) = field_value.as_bool() {
                                value.value = json!(bool_value);
                                
                                // Update position
                                if let Some(pos) = layout.pos.as_mut() {
                                    pos.0 = if bool_value { Ab(22.0) } else { Ab(2.0) };
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
```

## Control Implementation: Dropdown

```rust
/// Spawn a dropdown control
pub fn spawn_dropdown(
    commands: &mut Commands,
    setting_control: SettingControl,
    options: Vec<String>,
    initial_value: &str,
) -> Entity {
    let selected_index = options.iter()
        .position(|opt| opt == initial_value)
        .unwrap_or(0);

    let dropdown = commands.spawn((
        setting_control,
        DropdownOptions { options: options.clone(), selected_index },
        ControlValue { value: json!(initial_value) },
        UiLayout::window()
            .size((Ab(280.0), Ab(32.0)))
            .pack(),
        UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0)),
        UiHover::new().forward_speed(10.0),
        UiClicked::new().forward_speed(15.0),
        Text::new(initial_value),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("Dropdown"),
    )).id();

    // Arrow icon
    commands.spawn((
        Text::new("▼"),
        UiTextSize::from(Em(0.7)),
        UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
        UiLayout::window()
            .pos((Rl(95.0), Rl(50.0)))
            .anchor(Anchor::CenterRight)
            .pack(),
        Name::new("DropdownArrow"),
    )).set_parent(dropdown);

    dropdown
}

/// System: Handle dropdown clicks (open menu)
pub fn handle_dropdown_open(
    mut commands: Commands,
    mut dropdown_query: Query<
        (
            Entity,
            &DropdownOptions,
            &Interaction,
            &GlobalTransform,
        ),
        (Changed<Interaction>, With<SettingControl>),
    >,
) {
    for (entity, options, interaction, transform) in dropdown_query.iter() {
        if *interaction == Interaction::Pressed {
            // Spawn dropdown menu below the dropdown button
            let menu_entity = spawn_dropdown_menu(
                &mut commands,
                entity,
                &options.options,
                options.selected_index,
                transform.translation(),
            );

            info!("Opened dropdown menu");
        }
    }
}

fn spawn_dropdown_menu(
    commands: &mut Commands,
    parent_dropdown: Entity,
    options: &[String],
    selected_index: usize,
    position: Vec3,
) -> Entity {
    // Menu container
    let menu = commands.spawn((
        DropdownMenu { parent_dropdown },
        UiLayout::window()
            .size((Ab(280.0), Ab(options.len() as f32 * 32.0)))
            .pos((Ab(position.x), Ab(position.y - 32.0)))  // Below dropdown
            .pack(),
        UiColor::from(Color::srgba(0.18, 0.18, 0.22, 1.0)),
        Name::new("DropdownMenu"),
    )).id();

    // Menu options
    for (idx, option) in options.iter().enumerate() {
        let option_entity = commands.spawn((
            DropdownMenuOption {
                parent_dropdown,
                option_index: idx,
            },
            UiLayout::window()
                .size((Rl(100.0), Ab(32.0)))
                .pos((Rl(0.0), Ab(idx as f32 * 32.0)))
                .pack(),
            UiColor::from(if idx == selected_index {
                Color::srgba(0.3, 0.5, 0.8, 1.0)  // Highlight selected
            } else {
                Color::srgba(0.18, 0.18, 0.22, 1.0)
            }),
            UiHover::new().forward_speed(12.0),
            Text::new(option),
            UiTextSize::from(Em(0.95)),
            Pickable::default(),
            Interaction::None,
            Name::new(format!("DropdownOption_{}", idx)),
        )).id();
        commands.entity(option_entity).set_parent(menu);
    }

    menu
}

#[derive(Component)]
struct DropdownMenuOption {
    parent_dropdown: Entity,
    option_index: usize,
}

/// System: Handle dropdown menu option selection
pub fn handle_dropdown_selection(
    mut commands: Commands,
    mut option_query: Query<
        (&DropdownMenuOption, &Interaction, &Text),
        Changed<Interaction>,
    >,
    mut dropdown_query: Query<(
        &SettingControl,
        &mut DropdownOptions,
        &mut ControlValue,
        &mut Text,
    )>,
    menu_query: Query<Entity, With<DropdownMenu>>,
    mut write_events: EventWriter<SettingsWriteRequested>,
) {
    for (option, interaction, option_text) in option_query.iter() {
        if *interaction == Interaction::Pressed {
            // Update parent dropdown
            if let Ok((control, mut options, mut value, mut text)) = 
                dropdown_query.get_mut(option.parent_dropdown) {
                
                let old_value = value.value.clone();
                let new_value = option_text.to_string();
                
                options.selected_index = option.option_index;
                value.value = json!(&new_value);
                *text = Text::new(&new_value);

                // Emit database write
                write_events.send(SettingsWriteRequested {
                    operation_id: Uuid::new_v4(),
                    table: control.table.clone(),
                    field: control.field.clone(),
                    value: json!(new_value),
                    requester: option.parent_dropdown,
                });

                // Close menu
                for menu_entity in menu_query.iter() {
                    commands.entity(menu_entity).despawn_recursive();
                }
            }
        }
    }
}
```

## Control Implementation: Slider

```rust
/// Spawn a slider control
pub fn spawn_slider(
    commands: &mut Commands,
    setting_control: SettingControl,
    min: f32,
    max: f32,
    step: f32,
    initial_value: f32,
    labels: Vec<&str>,
) -> Entity {
    // Track
    let track = commands.spawn((
        UiLayout::window()
            .size((Ab(320.0), Ab(4.0)))
            .pack(),
        UiColor::from(Color::srgba(0.25, 0.25, 0.28, 1.0)),
        Name::new("SliderTrack"),
    )).id();

    // Handle
    let handle_pos = ((initial_value - min) / (max - min)) * 320.0;
    let handle = commands.spawn((
        setting_control,
        SliderValue { current: initial_value, min, max },
        ControlValue { value: json!(initial_value) },
        UiLayout::window()
            .size((Ab(16.0), Ab(16.0)))
            .pos((Ab(handle_pos), Ab(-6.0)))
            .pack(),
        UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0)),
        UiClicked::new().forward_speed(8.0),
        UiHover::new().forward_speed(12.0),
        Pickable::default(),
        Interaction::None,
        SliderDragging { is_dragging: false },
        Name::new("SliderHandle"),
    )).id();
    commands.entity(handle).set_parent(track);

    // Labels
    let label_count = labels.len();
    for (idx, label_text) in labels.iter().enumerate() {
        let x_pos = (idx as f32 / (label_count - 1) as f32) * 320.0;
        commands.spawn((
            Text::new(*label_text),
            UiTextSize::from(Em(0.85)),
            UiColor::from(Color::srgba(0.6, 0.6, 0.65, 1.0)),
            UiLayout::window()
                .pos((Ab(x_pos), Ab(12.0)))
                .anchor(Anchor::TopCenter)
                .pack(),
        )).set_parent(track);
    }

    handle
}

#[derive(Component)]
struct SliderDragging {
    is_dragging: bool,
}

/// System: Handle slider dragging
pub fn handle_slider_drag(
    mut slider_query: Query<(
        Entity,
        &SettingControl,
        &Interaction,
        &mut SliderValue,
        &mut SliderDragging,
        &mut UiLayout,
        &mut ControlValue,
    )>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    windows: Query<&Window>,
    mut write_events: EventWriter<SettingsWriteRequested>,
) {
    for (entity, control, interaction, mut slider, mut dragging, mut layout, mut value) in slider_query.iter_mut() {
        // Start dragging
        if *interaction == Interaction::Pressed && mouse_button.pressed(MouseButton::Left) {
            dragging.is_dragging = true;
        }

        // End dragging
        if !mouse_button.pressed(MouseButton::Left) {
            if dragging.is_dragging {
                // Emit final value on release
                write_events.send(SettingsWriteRequested {
                    operation_id: Uuid::new_v4(),
                    table: control.table.clone(),
                    field: control.field.clone(),
                    value: json!(slider.current),
                    requester: entity,
                });
            }
            dragging.is_dragging = false;
        }

        // Update position while dragging
        if dragging.is_dragging {
            if let Ok(window) = windows.get_single() {
                if let Some(cursor_pos) = window.cursor_position() {
                    // Calculate new slider value based on cursor position
                    // (simplified - actual implementation needs track bounds)
                    let normalized = (cursor_pos.x / 320.0).clamp(0.0, 1.0);
                    slider.current = slider.min + normalized * (slider.max - slider.min);
                    value.value = json!(slider.current);

                    // Update handle position
                    if let Some(pos) = layout.pos.as_mut() {
                        pos.0 = Ab(normalized * 320.0);
                    }
                }
            }
        }
    }
}
```

## Control Implementation: Hotkey Recorder

```rust
/// Spawn a hotkey recorder control
pub fn spawn_hotkey_recorder(
    commands: &mut Commands,
    setting_control: SettingControl,
    initial_hotkey: &str,
) -> Entity {
    commands.spawn((
        setting_control,
        HotkeyRecorderState {
            is_recording: false,
            current_modifiers: vec![],
            current_key: None,
        },
        ControlValue { value: json!(initial_hotkey) },
        UiLayout::window()
            .size((Ab(200.0), Ab(32.0)))
            .pack(),
        UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0)),
        UiHover::new().forward_speed(10.0),
        UiClicked::new().forward_speed(15.0),
        Text::new(initial_hotkey),
        UiTextSize::from(Em(0.95)),
        Pickable::default(),
        Interaction::None,
        Name::new("HotkeyRecorder"),
    )).id()
}

/// System: Handle hotkey recorder click (enter recording mode)
pub fn handle_hotkey_recorder_click(
    mut recorder_query: Query<
        (&Interaction, &mut HotkeyRecorderState, &mut Text, &mut UiColor),
        (Changed<Interaction>, With<SettingControl>),
    >,
) {
    for (interaction, mut state, mut text, mut color) in recorder_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            state.is_recording = true;
            state.current_modifiers.clear();
            state.current_key = None;
            *text = Text::new("Press keys...");
            *color = UiColor::from(Color::srgba(0.3, 0.5, 0.8, 1.0));  // Highlight
        }
    }
}

/// System: Capture hotkey while recording
pub fn capture_hotkey(
    mut recorder_query: Query<(
        Entity,
        &SettingControl,
        &mut HotkeyRecorderState,
        &mut ControlValue,
        &mut Text,
        &mut UiColor,
    )>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut write_events: EventWriter<SettingsWriteRequested>,
) {
    for (entity, control, mut state, mut value, mut text, mut color) in recorder_query.iter_mut() {
        if !state.is_recording {
            continue;
        }

        // Cancel on Escape
        if keyboard.just_pressed(KeyCode::Escape) {
            state.is_recording = false;
            *text = Text::new(value.value.as_str().unwrap_or(""));
            *color = UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0));
            continue;
        }

        // Capture modifiers
        if keyboard.pressed(KeyCode::ControlLeft) || keyboard.pressed(KeyCode::ControlRight) {
            if !state.current_modifiers.contains(&"Ctrl".to_string()) {
                state.current_modifiers.push("Ctrl".to_string());
            }
        }
        if keyboard.pressed(KeyCode::AltLeft) || keyboard.pressed(KeyCode::AltRight) {
            if !state.current_modifiers.contains(&"Alt".to_string()) {
                state.current_modifiers.push("Alt".to_string());
            }
        }
        if keyboard.pressed(KeyCode::ShiftLeft) || keyboard.pressed(KeyCode::ShiftRight) {
            if !state.current_modifiers.contains(&"Shift".to_string()) {
                state.current_modifiers.push("Shift".to_string());
            }
        }
        if keyboard.pressed(KeyCode::SuperLeft) || keyboard.pressed(KeyCode::SuperRight) {
            if !state.current_modifiers.contains(&"Cmd".to_string()) {
                state.current_modifiers.push("Cmd".to_string());
            }
        }

        // Capture regular key
        for key in keyboard.get_just_pressed() {
            if !matches!(key, 
                KeyCode::ControlLeft | KeyCode::ControlRight |
                KeyCode::AltLeft | KeyCode::AltRight |
                KeyCode::ShiftLeft | KeyCode::ShiftRight |
                KeyCode::SuperLeft | KeyCode::SuperRight |
                KeyCode::Escape
            ) {
                state.current_key = Some(format!("{:?}", key));
                
                // Build hotkey string
                let hotkey_string = format!(
                    "{} {}",
                    state.current_modifiers.join(" "),
                    state.current_key.as_ref().unwrap()
                );
                
                // Save and exit recording
                value.value = json!(hotkey_string);
                *text = Text::new(&hotkey_string);
                *color = UiColor::from(Color::srgba(0.2, 0.2, 0.25, 1.0));
                state.is_recording = false;

                // Emit database write
                write_events.send(SettingsWriteRequested {
                    operation_id: Uuid::new_v4(),
                    table: control.table.clone(),
                    field: control.field.clone(),
                    value: json!(hotkey_string),
                    requester: entity,
                });

                break;
            }
        }
    }
}
```

## Implementation Checklist

### Component Definitions
- [ ] Create `SettingControl` component
- [ ] Create `ControlType` enum
- [ ] Create `ControlValue` component
- [ ] Create control-specific state components
- [ ] Create validation structures

### Toggle Switch
- [ ] Implement spawn function
- [ ] Implement click handler
- [ ] Implement update from database
- [ ] Implement animations
- [ ] Test all states

### Dropdown
- [ ] Implement spawn function
- [ ] Implement open menu system
- [ ] Implement selection system
- [ ] Implement close menu system
- [ ] Implement keyboard navigation
- [ ] Test all interactions

### Slider
- [ ] Implement spawn function
- [ ] Implement drag system
- [ ] Implement snap-to-position
- [ ] Implement keyboard support
- [ ] Test dragging and updates

### Hotkey Recorder
- [ ] Implement spawn function
- [ ] Implement recording mode
- [ ] Implement key capture
- [ ] Implement conflict detection
- [ ] Test all key combinations

### Text Input
- [ ] Implement spawn function
- [ ] Implement text editing
- [ ] Implement validation
- [ ] Implement error states
- [ ] Test edge cases

### Radio Group
- [ ] Implement spawn function
- [ ] Implement selection logic
- [ ] Implement visual previews
- [ ] Test exclusivity

### Button Group
- [ ] Implement spawn function
- [ ] Implement selection highlight
- [ ] Test interactions

### Checkbox
- [ ] Implement spawn function
- [ ] Implement toggle logic
- [ ] Test states

### Action Button
- [ ] Implement spawn function
- [ ] Implement click handler
- [ ] Implement animations
- [ ] Test disabled state

### Info Icon
- [ ] Implement spawn function
- [ ] Implement tooltip system
- [ ] Implement positioning
- [ ] Test dismissal

## Definition of Done

- [ ] All 10 control types implemented
- [ ] All spawn functions work correctly
- [ ] All interaction systems work
- [ ] All database integrations work
- [ ] All animations are smooth (60fps)
- [ ] Keyboard navigation works for all controls
- [ ] Code compiles without warnings
- [ ] Tests pass for all controls
- [ ] Documentation complete
- [ ] **NO STUBS** - all controls fully functional
- [ ] Ready for TASK7.1-7.8 to use

## Performance Targets

- Control spawn: < 1ms per control
- Interaction response: < 16ms (60fps)
- Animation smoothness: 60fps maintained
- Memory: No leaks over 1000 interactions

## Notes

**Critical Implementation Notes:**

1. **Database Integration**: Every control must emit `SettingsWriteRequested` on change and update from `SettingsReadResponse`.

2. **Animation**: Use `UiClicked` and `UiHover` from ecs-ui for smooth, automatic animations.

3. **Keyboard Support**: All interactive controls should support keyboard navigation (Tab, Arrow keys, Space, Enter).

4. **State Management**: Use `ControlValue` component to track current value. Update on both user interaction and database response.

5. **Reusability**: Each spawn function should take minimal parameters and return an Entity. Make it easy for panels to use.

6. **Error Handling**: Validate inputs where appropriate. Show clear error states.

7. **Accessibility**: Consider screen readers and keyboard-only users.

**Common Pitfalls:**

- Don't forget to emit both `SettingsWriteRequested` and `SettingChanged` events
- Remember to update visual state when value changes from database
- Ensure dropdown menus despawn when closed
- Test edge cases (empty strings, out of range values, etc.)
- Hotkey conflicts need proper detection

**Testing Strategy:**

1. Unit test: Each control spawn function
2. Unit test: State updates from database
3. Integration test: User interaction flow
4. Integration test: Database round-trip
5. Performance test: Smooth animations
6. Accessibility test: Keyboard navigation
7. Visual test: Match screenshots
