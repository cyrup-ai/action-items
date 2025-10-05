# Global Hotkey Configuration Specification

## Overview
The Global Hotkey Configuration interface provides comprehensive controls for system-wide launcher activation, theme preferences, window modes, and user interface customization. This is a critical component for seamless launcher integration with the user's workflow.

## Visual Design

### Layout Structure
- **Settings Window**: Standard macOS window with General tab selected
- **Configuration Sections**: Vertically stacked sections with consistent spacing
- **Recording Modal**: Floating modal for real-time hotkey capture
- **Visual Previews**: Window mode previews with actual interface mockups

### Recording Modal Interface
- **Modal Position**: Centered overlay with subtle backdrop blur
- **Recording Indicator**: "Recording..." text with activity indicator
- **Key Display**: Large, clear display of captured key combination (⌘ Space)
- **Real-time Feedback**: Immediate display of pressed keys
- **Modal Dismiss**: Automatic closure on successful capture

### Section Organization
1. **Startup Configuration**: Application launch preferences
2. **Raycast Hotkey**: Global activation key configuration
3. **Menu Bar Icon**: System menu bar integration toggle
4. **Text Size**: User interface text scaling options  
5. **Theme Configuration**: Appearance and color scheme settings
6. **Window Mode**: Interface density and layout options
7. **Favorites**: Content display preferences

## Functional Requirements

### Global Hotkey Management
- **Real-time Capture**: Live recording of key combinations
- **Conflict Detection**: Automatic detection of system shortcut conflicts
- **Modifier Support**: Full support for ⌘, ⌥, ⌃, ⇧ modifier keys
- **Visual Feedback**: Clear display of recorded combinations
- **Error Handling**: Graceful handling of invalid or conflicted shortcuts

### Recording System
- **Activation**: Click-to-record interaction model
- **Key Capture**: System-level keyboard hook for global capture
- **Validation**: Real-time validation of key combination validity
- **Cancellation**: Escape key or click-outside to cancel recording
- **Confirmation**: Automatic confirmation of successful capture

### Theme System
- **Theme Selection**: Multiple theme options (Raycast Dark, Raycast Light)
- **System Integration**: "Follow system appearance" automatic switching
- **Theme Studio**: Advanced theme customization interface
- **Live Preview**: Real-time theme switching without restart
- **Accessibility**: High contrast and color blind friendly options

### Window Mode Configuration
- **Mode Selection**: Visual selection between Default and Compact modes
- **Live Previews**: Actual interface previews showing layout differences
- **Mode Switching**: Instant application of selected mode
- **Responsive Design**: Automatic adaptation to selected mode
- **State Persistence**: Remember mode selection across sessions

## Technical Implementation

### Core Systems Required

#### Hotkey Capture System
```rust
// Key Bevy examples to reference:
// - input/keyboard_input.rs - Keyboard capture and processing
// - input/keyboard_input_events.rs - Key event handling
// - input/keyboard_modifiers.rs - Modifier key combinations
```

#### Global Hotkey Component
```rust
#[derive(Resource)]
pub struct GlobalHotkeyConfig {
    pub current_hotkey: Option<KeyCombination>,
    pub recording_active: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub registration_status: RegistrationStatus,
}

#[derive(Clone, Debug)]
pub struct KeyCombination {
    pub modifiers: ModifierKeys,
    pub key: KeyCode,
    pub display_string: String,
}

#[derive(Component)]
pub struct HotkeyRecordingModal {
    pub visible: bool,
    pub captured_keys: Vec<KeyCode>,
    pub recording_start_time: Instant,
}
```

#### Theme Management System
```rust
// Key Bevy examples to reference:
// - ui/gradients.rs - Theme color systems
// - ui/transparency_ui.rs - Theme-aware transparency
// - games/game_menu.rs - Theme switching implementation
```

#### Theme Configuration
```rust
#[derive(Resource, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub active_theme: ThemeType,
    pub follow_system: bool,
    pub custom_theme_path: Option<PathBuf>,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ThemeType {
    RaycastDark,
    RaycastLight,
    Custom(ThemeId),
}
```

### Event System
```rust
#[derive(Event)]
pub enum GlobalConfigEvent {
    StartHotkeyRecording,
    CancelHotkeyRecording,
    HotkeyRecorded(KeyCombination),
    ThemeChanged(ThemeType),
    WindowModeChanged(WindowMode),
    TextSizeChanged(TextSize),
}
```

### Input Handling System
```rust
// Key Bevy examples to reference:
// - input/text_input.rs - Text input and key capture
// - ui/button.rs - Interactive UI elements
// - games/game_menu.rs - Menu interaction patterns
```

## Advanced Features

### Hotkey Validation
1. **System Integration**: Check against macOS system shortcuts
2. **Application Conflicts**: Detect conflicts with running applications
3. **Accessibility**: Ensure shortcuts work with accessibility tools
4. **Multi-platform**: Handle platform-specific key mappings
5. **Fallback System**: Automatic fallback when preferred hotkey unavailable

### Recording Interface
1. **Visual Feedback**: Clear indication of recording state
2. **Key Visualization**: Real-time display of pressed keys
3. **Error States**: Clear indication of invalid combinations
4. **Success Confirmation**: Visual confirmation of successful recording
5. **Timeout Handling**: Automatic timeout for abandoned recordings

### Theme Integration
1. **Live Switching**: Instant theme changes without restart
2. **System Sync**: Automatic switching with macOS appearance changes
3. **Custom Themes**: Support for user-created theme files
4. **Theme Studio**: Advanced theme editor integration
5. **Export/Import**: Share themes between installations

### Window Mode System
1. **Mode Previews**: Accurate previews of interface changes
2. **Responsive Layout**: Automatic UI adaptation to selected mode
3. **Performance Optimization**: Efficient rendering in compact mode
4. **Accessibility**: Ensure all modes meet accessibility standards
5. **Animation**: Smooth transitions between modes

## Integration Points

### System Integration
- **Global Hotkey Registration**: macOS/Windows/Linux hotkey APIs
- **System Theme Detection**: OS theme change notifications
- **Menu Bar Integration**: System menu bar icon management
- **Accessibility Services**: Integration with screen readers and assistive tech

### Application Integration
- **Settings Persistence**: Save configuration across sessions
- **Live Updates**: Apply changes without restart when possible
- **Validation System**: Ensure configuration integrity
- **Error Recovery**: Handle corrupted or invalid configurations

## Related Bevy Examples

### Primary References
- [`input/keyboard_input.rs`](../../docs/bevy/examples/input/keyboard_input.rs) - Global keyboard capture and processing
- [`input/keyboard_input_events.rs`](../../docs/bevy/examples/input/keyboard_input_events.rs) - Key event handling for recording
- [`input/keyboard_modifiers.rs`](../../docs/bevy/examples/input/keyboard_modifiers.rs) - Modifier key combinations (⌘⌥⌃⇧)
- [`games/game_menu.rs`](../../docs/bevy/examples/games/game_menu.rs) - Settings interface and state management
- [`ui/button.rs`](../../docs/bevy/examples/ui/button.rs) - Interactive UI elements and dropdowns

### Supporting References
- [`ui/flex_layout.rs`](../../docs/bevy/examples/ui/flex_layout.rs) - Settings panel layout
- [`ui/text.rs`](../../docs/bevy/examples/ui/text.rs) - Text rendering and size options
- [`ui/transparency_ui.rs`](../../docs/bevy/examples/ui/transparency_ui.rs) - Modal overlay effects
- [`ui/gradients.rs`](../../docs/bevy/examples/ui/gradients.rs) - Theme color systems
- [`animation/animated_ui.rs`](../../docs/bevy/examples/animation/animated_ui.rs) - Recording feedback animations

### System Integration References
- [`window/window_settings.rs`](../../docs/bevy/examples/window/window_settings.rs) - Window mode configuration
- [`ui/ui_scaling.rs`](../../docs/bevy/examples/ui/ui_scaling.rs) - Text size and UI scaling
- [`asset/asset_loading.rs`](../../docs/bevy/examples/asset/asset_loading.rs) - Theme resource loading

## Performance Requirements

### Recording Performance
- **Key Capture Latency**: < 10ms for key registration
- **Visual Feedback**: < 50ms for recording display updates
- **Validation Speed**: < 100ms for conflict detection
- **Modal Response**: < 100ms for modal show/hide

### Theme Switching
- **Theme Application**: < 200ms for complete theme switch
- **Preview Generation**: < 500ms for window mode previews
- **Resource Loading**: Lazy loading of theme assets
- **Memory Usage**: Efficient theme resource management

## Accessibility Features

### Keyboard Navigation
1. **Full Keyboard Access**: All settings accessible via keyboard
2. **Recording Accessibility**: Screen reader announcements during recording
3. **Focus Management**: Clear focus indicators and logical tab order
4. **Shortcut Keys**: Quick access keys for common settings

### Visual Accessibility
1. **High Contrast**: Support for system high contrast modes
2. **Text Scaling**: Respect system text size preferences
3. **Color Independence**: Don't rely solely on color for information
4. **Animation Control**: Respect reduced motion preferences

## Error Handling

### Recording Errors
1. **Invalid Combinations**: Clear feedback for unusable shortcuts
2. **System Conflicts**: Warning about conflicting system shortcuts
3. **Platform Limitations**: Handle platform-specific restrictions
4. **Timeout Recovery**: Graceful handling of recording timeouts

### Configuration Errors
1. **Corrupted Settings**: Recovery from invalid configuration files
2. **Theme Failures**: Fallback to default theme on load errors
3. **Hotkey Registration**: Fallback options when primary hotkey unavailable
4. **System Changes**: Adapt to system configuration changes

## Implementation Priority

### Phase 1: Core Hotkey System
1. Basic hotkey recording interface
2. Key combination capture and validation
3. Global hotkey registration
4. Settings persistence

### Phase 2: Enhanced Interface
1. Recording modal with visual feedback
2. Theme selection and switching
3. Window mode configuration
4. Text size options

### Phase 3: Advanced Features
1. Conflict detection and resolution
2. Custom theme support
3. System integration features
4. Advanced accessibility options

### Phase 4: Polish & Optimization
1. Smooth animations and transitions
2. Performance optimizations
3. Comprehensive error handling
4. Advanced theme studio integration

## Bevy Implementation Details

### Component Architecture & Modal System

```rust
use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct HotkeyRecordingModal {
    pub visible: bool,
    pub captured_keys: Vec<KeyCode>,
    pub modifiers: Vec<Modifier>,
    pub display_text: String,
    pub recording_start: std::time::Instant,
}

#[derive(Component, Reflect)]
pub struct GlobalHotkeyConfig {
    pub current_hotkey: KeyCombination,
    pub conflicts: Vec<String>,
    pub valid: bool,
}

#[derive(Event)]
pub enum HotkeyConfigEvent {
    StartRecording,
    KeyCaptured(KeyCombination),
    RecordingCancelled,
    HotkeyApplied(KeyCombination),
    ConflictDetected(Vec<String>),
}

// Modal overlay system with backdrop blur
fn spawn_recording_modal(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)), // Modal backdrop
            ZIndex(1000),
            HotkeyRecordingModal {
                visible: true,
                captured_keys: vec![],
                modifiers: vec![],
                display_text: "Recording...".to_string(),
                recording_start: std::time::Instant::now(),
            },
        ))
        .with_children(|parent| {
            // Modal content card
            parent.spawn((
                Node {
                    width: Val::Px(300.0),
                    height: Val::Px(150.0),
                    padding: UiRect::all(Val::Px(24.0)),
                    border_radius: BorderRadius::all(Val::Px(12.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    overflow: Overflow::clip(), // Prevent expansion
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                BorderColor(Color::srgb(0.4, 0.4, 0.4)),
            )).with_children(|parent| {
                // Recording status
                parent.spawn((
                    Text::new("Recording..."),
                    TextFont { font_size: 18.0, ..default() },
                    TextColor(Color::WHITE),
                    Node { margin: UiRect::bottom(Val::Px(16.0)), ..default() },
                ));
                
                // Key display
                parent.spawn((
                    Text::new("⌘ Space"),
                    TextFont { font_size: 24.0, ..default() },
                    TextColor(Color::srgb(0.8, 0.8, 1.0)),
                ));
            });
        });
}

// Keyboard capture system with real-time feedback
fn hotkey_capture_system(
    mut modals: Query<&mut HotkeyRecordingModal>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut events: EventWriter<HotkeyConfigEvent>,
    mut text_query: Query<&mut Text>,
) {
    for mut modal in modals.iter_mut() {
        if !modal.visible { continue; }
        
        // Capture modifiers in real-time
        modal.modifiers.clear();
        if keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight) {
            modal.modifiers.push(Modifier::Command);
        }
        if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
            modal.modifiers.push(Modifier::Control);
        }
        if keyboard_input.pressed(KeyCode::AltLeft) || keyboard_input.pressed(KeyCode::AltRight) {
            modal.modifiers.push(Modifier::Alt);
        }
        if keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight) {
            modal.modifiers.push(Modifier::Shift);
        }
        
        // Update display in real-time
        let mut display_parts = vec![];
        for modifier in &modal.modifiers {
            display_parts.push(modifier.symbol());
        }
        
        // Capture final key
        for key in keyboard_input.get_just_pressed() {
            if !matches!(key, 
                KeyCode::SuperLeft | KeyCode::SuperRight |
                KeyCode::ControlLeft | KeyCode::ControlRight |
                KeyCode::AltLeft | KeyCode::AltRight |
                KeyCode::ShiftLeft | KeyCode::ShiftRight |
                KeyCode::Escape
            ) {
                display_parts.push(key_to_symbol(*key));
                
                let combination = KeyCombination {
                    modifiers: modal.modifiers.clone(),
                    key: *key,
                };
                
                events.send(HotkeyConfigEvent::KeyCaptured(combination));
                modal.visible = false;
                break;
            }
        }
        
        modal.display_text = display_parts.join(" ");
        
        // Update text components
        for mut text in text_query.iter_mut() {
            **text = modal.display_text.clone();
        }
        
        // Handle escape to cancel
        if keyboard_input.just_pressed(KeyCode::Escape) {
            events.send(HotkeyConfigEvent::RecordingCancelled);
            modal.visible = false;
        }
    }
}
```

This focused Bevy implementation provides essential hotkey configuration patterns with modal overlays and real-time keyboard capture using proper flex constraints to prevent UI expansion.