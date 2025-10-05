# Contextual Action Menu Specification

## Overview
The Contextual Action Menu is a modal overlay that provides power user functionality for managing individual action items. It appears when users right-click on an action item or use keyboard shortcuts to access advanced management options.

## Visual Design

### Layout Structure
- **Modal Overlay**: Semi-transparent dark overlay covering the main interface
- **Menu Container**: Rounded rectangular popup positioned near the selected action item
- **Menu Items**: Vertically stacked interactive items with icons, text, and keyboard shortcuts
- **Search Field**: Bottom text input for filtering menu actions ("Search for actions...")
- **Background Dimming**: Main interface partially visible but dimmed

### Positioning & Sizing
- **Width**: Approximately 25-30% of viewport width
- **Height**: Auto-sizing based on available actions
- **Position**: Context-aware positioning near the selected item, with smart edge detection
- **Spacing**: 0.8% VMin padding around menu container, 0.4% VMin between menu items
- **Margins**: 1.2% VMin margin from screen edges

### Visual Hierarchy
- **Primary Actions**: Standard white/light gray text
- **Destructive Actions**: Red text (#ff6b6b) for dangerous operations like "Disable Command"
- **Icons**: Responsive icons (~2% VMin) on the left of each menu item
- **Shortcuts**: Right-aligned keyboard shortcut indicators
- **Separators**: Subtle divider lines between logical groups (when applicable)

## Functional Requirements

### Menu Actions
1. **Copy Deeplink** (⌘C)
   - Copy a shareable link to the action item
   - Icon: Clipboard/link icon
   - Creates a universal reference to the action

2. **Configure Command** (⇧⌘,)
   - Open command-specific configuration interface
   - Icon: Gear/settings icon
   - Allows customization of command behavior and parameters

3. **Configure Extension** (⌥⌘,)
   - Open extension-wide configuration interface
   - Icon: Plugin/extension icon
   - Manages settings for the entire plugin/extension

4. **Disable Command** (D)
   - Temporarily disable the selected command
   - Icon: Prohibition/disable icon
   - Red text color to indicate destructive/cautionary action
   - Should show confirmation dialog for safety

### Interaction Behaviors
1. **Modal Behavior**: Click outside menu to close
2. **Keyboard Navigation**: Arrow keys navigate menu items
3. **Quick Actions**: Direct keyboard shortcuts execute actions immediately
4. **Escape Key**: Close menu without action
5. **Enter Key**: Execute selected menu item
6. **Search Integration**: Bottom search field filters available actions

### Context Sensitivity
- Menu contents adapt based on selected action type
- Some actions may be disabled/hidden for certain action types
- Plugin-specific actions appear contextually
- System commands have different options than user commands

## Technical Implementation

### Core Systems Required

#### Modal System
```rust
// Key Bevy examples to reference:
// - ui/overflow.rs - overlay positioning and clipping
// - games/game_menu.rs - modal menu behavior
// - ui/z_index.rs - layering and depth management
```

#### Context Menu Component
```rust
#[derive(Component)]
pub struct ContextMenu {
    pub target_action_id: String,
    pub position: Vec2,
    pub visible: bool,
    pub selected_index: usize,
    pub available_actions: Vec<ContextAction>,
}

#[derive(Clone)]
pub struct ContextAction {
    pub id: String,
    pub title: String,
    pub icon: IconType,
    pub shortcut: String,
    pub destructive: bool,
    pub enabled: bool,
}
```

#### Input Handling System
```rust
// Key Bevy examples to reference:
// - input/keyboard_input.rs - keyboard navigation
// - input/keyboard_modifiers.rs - modifier combinations (⌘, ⇧, ⌥)
// - input/mouse_input.rs - right-click detection
// - input/mouse_input_events.rs - click outside detection
```

### Event System
```rust
#[derive(Event)]
pub enum ContextMenuEvent {
    Show { action_id: String, position: Vec2 },
    Hide,
    ExecuteAction(String),
    CopyDeeplink(String),
    ConfigureCommand(String),
    ConfigureExtension(String),
    DisableCommand(String),
}
```

### Positioning System
```rust
// Key Bevy examples to reference:
// - ui/relative_cursor_position.rs - position calculation
// - camera/2d_top_down_camera.rs - coordinate system handling
```

## Advanced Features

### Smart Positioning
1. **Edge Detection**: Automatically reposition when near screen edges
2. **Multi-Monitor Support**: Proper positioning across multiple displays
3. **Adaptive Sizing**: Menu size adjusts based on available actions
4. **Animation**: Smooth show/hide transitions with spring physics

### Accessibility Features
1. **Keyboard Navigation**: Full keyboard accessibility
2. **Screen Reader Support**: Proper ARIA labels and role attributes
3. **High Contrast**: Respect system high contrast preferences
4. **Focus Management**: Proper focus trapping within modal
5. **Escape Sequences**: Multiple ways to dismiss menu

### Performance Optimizations
1. **Lazy Loading**: Only generate menu when needed
2. **Action Filtering**: Efficiently filter available actions based on context
3. **Animation Optimization**: GPU-accelerated transitions
4. **Memory Management**: Proper cleanup of modal resources

## Integration Points

### Plugin System Integration
- **Plugin Actions**: Dynamic actions from registered plugins
- **Context Awareness**: Actions filtered based on plugin capabilities
- **Icon Loading**: Efficient loading of plugin-specific icons
- **Configuration Bridge**: Connect to plugin configuration interfaces

### Command Management Integration
- **Command Registry**: Integration with central command registry
- **State Management**: Track enabled/disabled state of commands
- **Configuration Persistence**: Save configuration changes
- **Validation**: Ensure commands remain functional after configuration

### Deep Linking System
- **URL Generation**: Create shareable URLs for actions
- **Action Resolution**: Resolve URLs back to actions
- **Cross-Platform Support**: URLs work across different installations
- **Security**: Validate and sanitize deep link requests

## Related Bevy Examples

### Primary References
- [`games/game_menu.rs`](../../docs/bevy/examples/games/game_menu.rs) - Modal menu patterns and state management
- [`ui/overflow.rs`](../../docs/bevy/examples/ui/overflow.rs) - Overlay positioning and clipping
- [`ui/z_index.rs`](../../docs/bevy/examples/ui/z_index.rs) - Layering system for modals
- [`input/mouse_input_events.rs`](../../docs/bevy/examples/input/mouse_input_events.rs) - Right-click and outside-click detection
- [`input/keyboard_modifiers.rs`](../../docs/bevy/examples/input/keyboard_modifiers.rs) - Modifier key combinations

### Supporting References
- [`ui/button.rs`](../../docs/bevy/examples/ui/button.rs) - Interactive menu items
- [`ui/flex_layout.rs`](../../docs/bevy/examples/ui/flex_layout.rs) - Menu item layout
- [`input/keyboard_input.rs`](../../docs/bevy/examples/input/keyboard_input.rs) - Menu navigation
- [`ui/relative_cursor_position.rs`](../../docs/bevy/examples/ui/relative_cursor_position.rs) - Position calculation
- [`animation/animated_ui.rs`](../../docs/bevy/examples/animation/animated_ui.rs) - Show/hide animations
- [`ui/transparency_ui.rs`](../../docs/bevy/examples/ui/transparency_ui.rs) - Modal overlay effects

### Animation & Effects References
- [`animation/easing_functions.rs`](../../docs/bevy/examples/animation/easing_functions.rs) - Smooth transitions
- [`2d/bloom_2d.rs`](../../docs/bevy/examples/2d/bloom_2d.rs) - Visual effects for focus
- [`ui/gradients.rs`](../../docs/bevy/examples/ui/gradients.rs) - Background styling

## Error Handling & Edge Cases

### Error States
1. **Action Not Found**: Handle cases where selected action no longer exists
2. **Plugin Unavailable**: Graceful degradation when plugin is disabled
3. **Configuration Errors**: Handle invalid configuration states
4. **Permission Errors**: Handle cases where user lacks permissions

### Edge Cases
1. **Screen Edge Positioning**: Menu positioning near screen boundaries
2. **Rapid Menu Operations**: Handle quick open/close sequences
3. **Multi-Selection**: Handle multiple selected actions (future feature)
4. **Concurrent Modifications**: Handle actions being modified while menu is open

## Testing Requirements

### Interaction Testing
1. **Keyboard Navigation**: Verify all keyboard shortcuts work correctly
2. **Mouse Interaction**: Test right-click activation and click-outside dismissal
3. **Multi-Platform**: Verify behavior across macOS, Windows, and Linux
4. **Accessibility**: Screen reader and keyboard-only navigation testing

### Visual Testing
1. **Positioning**: Menu appears in correct location relative to trigger
2. **Responsive Design**: Menu adapts to different screen sizes
3. **Theme Integration**: Menu respects system dark/light theme
4. **Animation**: Smooth transitions without visual artifacts

## Implementation Priority

### Phase 1: Basic Modal
1. Modal overlay system with backdrop
2. Basic menu item rendering
3. Click outside to close
4. Escape key to dismiss

### Phase 2: Core Actions
1. Copy Deeplink functionality
2. Configure Command integration
3. Configure Extension integration
4. Disable Command with confirmation

### Phase 3: Enhanced UX
1. Keyboard navigation within menu
2. Smart positioning system
3. Show/hide animations
4. Context-sensitive action filtering

### Phase 4: Polish & Optimization
1. Performance optimizations
2. Accessibility enhancements
3. Advanced animations
4. Comprehensive error handling

## Bevy Implementation Details

### Contextual Action Menu Components

```rust
use bevy::prelude::*;

#[derive(Component, Reflect)]
pub struct ContextualActionMenu {
    pub visible: bool,
    pub target_action: Option<String>,
    pub position: Vec2,
    pub selected_index: usize,
    pub actions: Vec<ContextAction>,
}

#[derive(Component, Reflect)]
pub struct ModalOverlay {
    pub alpha: f32,
    pub target_alpha: f32,
    pub animation_speed: f32,
}

// Modal overlay system with backdrop
fn spawn_contextual_menu(mut commands: Commands, position: Vec2, actions: Vec<ContextAction>) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)), // Modal backdrop
            ZIndex(999),
            ModalOverlay {
                alpha: 0.0,
                target_alpha: 0.5,
                animation_speed: 8.0,
            },
        ))
        .with_children(|parent| {
            // Context menu popup
            parent.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    left: Val::Px(position.x),
                    top: Val::Px(position.y),
                    width: Val::Px(280.0),
                    flex_direction: FlexDirection::Column,
                    padding: UiRect::all(Val::Px(8.0)),
                    border_radius: BorderRadius::all(Val::Px(8.0)),
                    overflow: Overflow::clip(), // Prevent expansion
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                BorderColor(Color::srgb(0.4, 0.4, 0.4)),
                ContextualActionMenu {
                    visible: true,
                    target_action: None,
                    position,
                    selected_index: 0,
                    actions,
                },
            ));
        });
}

// Keyboard navigation in context menu
fn contextual_menu_navigation_system(
    mut menus: Query<&mut ContextualActionMenu>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
) {
    for mut menu in menus.iter_mut() {
        if !menu.visible { continue; }
        
        if keyboard_input.just_pressed(KeyCode::ArrowUp) {
            if menu.selected_index > 0 {
                menu.selected_index -= 1;
            }
        }
        
        if keyboard_input.just_pressed(KeyCode::ArrowDown) {
            if menu.selected_index < menu.actions.len().saturating_sub(1) {
                menu.selected_index += 1;
            }
        }
        
        if keyboard_input.just_pressed(KeyCode::Escape) {
            menu.visible = false;
            // Despawn menu entity
        }
    }
}
```

### SystemSet Organization for Modal Menus

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ContextMenuSystems {
    Input,
    Animation,
    Positioning,
    UI,
}

impl Plugin for ContextMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<ContextualActionMenu>()
            .register_type::<ModalOverlay>()
            
            .add_systems(Update, (
                contextual_menu_navigation_system.in_set(ContextMenuSystems::Input),
                modal_animation_system.in_set(ContextMenuSystems::Animation),
            ));
    }
}
```