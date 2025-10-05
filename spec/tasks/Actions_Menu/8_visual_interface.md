# Actions Menu - Visual Interface and Animation System

## Implementation Task: Complete Visual Interface with Animations and Styling

### Architecture Overview
Implement the comprehensive visual interface system for the Actions Menu including dark theme styling, micro-animations, icon management, and responsive layout components.

### Core Components

#### Visual Interface System
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct ActionsMenuInterface {
    pub search_bar: SearchBarComponent,
    pub favorites_list: FavoritesListComponent,
    pub action_menu: ContextActionMenu,
    pub bottom_bar: ActionBarComponent,
    pub theme: InterfaceTheme,
}

#[derive(Reflect, Default)]
pub struct InterfaceTheme {
    pub background_primary: Color,      // #1a1a1a
    pub background_secondary: Color,    // #2a2a2a  
    pub text_primary: Color,           // #ffffff
    pub text_secondary: Color,         // #888888
    pub accent_blue: Color,           // Selection/focus
    pub border_subtle: Color,         // rgba(255,255,255,0.1)
}

#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct SearchBarComponent {
    pub input_field: Entity,
    pub ai_buttons: Vec<Entity>,
    pub placeholder_text: String,
    pub has_focus: bool,
}
```

#### Animation System
```rust
#[derive(Resource, Reflect, Default)]
#[reflect(Resource)]
pub struct UIAnimationSystem {
    pub active_animations: HashMap<Entity, Vec<ActiveAnimation>>,
    pub animation_presets: HashMap<AnimationType, AnimationSettings>,
    pub performance_mode: AnimationPerformanceMode,
}

#[derive(Reflect)]
pub struct ActiveAnimation {
    pub animation_type: AnimationType,
    pub duration: Duration,
    pub easing: EasingFunction,
    pub start_time: SystemTime,
    pub target_values: AnimationTarget,
}

#[derive(Reflect)]
pub enum AnimationType {
    SelectionHighlight,
    HoverTransition,
    MenuExpansion,
    IconLoad,
    SearchFilter,
    ContextMenuSlide,
}
```

### Bevy Implementation References

#### UI Layout and Styling
- **Flex Layout**: `docs/bevy/examples/ui/flex_layout.rs`
  - Two-panel layout with search bar, favorites list, and action menu
  - Responsive flex containers for different screen sizes
  - Dynamic sizing and spacing for UI components

#### Text Input and Search
- **Text Input**: `docs/bevy/examples/input/text_input.rs`
  - Search input field with placeholder text and focus states
  - Real-time text input handling and visual feedback
  - Cursor positioning and text selection

#### Button and Interactive Elements
- **Button System**: `docs/bevy/examples/ui/button.rs`
  - AI buttons in search bar with hover and click states
  - Action buttons in bottom bar with keyboard shortcuts
  - Context menu button interactions

#### Icon and Asset Management
- **Texture Atlas**: `docs/bevy/examples/ui/ui_texture_atlas.rs`
  - Command and application icons in favorites list
  - Efficient icon atlas loading and rendering
  - Dynamic icon updates and caching

### Search Bar Interface Implementation

#### Search Input Field
- **Styling**: Full-width dark input field (#2a2a2a) with rounded corners
- **Typography**: White text with medium gray placeholder
- **Placeholder**: "Search for apps and commands..."
- **Focus State**: Subtle border highlight and cursor visibility
- **Real-time Updates**: Dynamic filtering as user types

#### AI Integration Buttons
- **Ask AI Button**: Compact tab-style button with "Ask AI" text
- **Tab Button**: Secondary button indicating keyboard shortcut
- **Styling**: Subtle dark background matching interface theme
- **Positioning**: Right-aligned within search bar container
- **Interaction**: Hover states and click feedback

### Favorites List Visual System

#### List Container Structure
- **Section Header**: "Favorites" with consistent typography
- **List Background**: Dark theme background with item separation
- **Scrolling**: Smooth scrolling for large lists of commands
- **Selection**: Clear visual indication of selected items

#### Command Item Visual Components
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct CommandItemVisual {
    pub icon: Handle<Image>,
    pub name_text: Entity,
    pub source_text: Entity,
    pub alias_pill: Entity,
    pub type_indicator: Entity,
    pub hover_state: HoverState,
}

#[derive(Reflect)]
pub struct CommandIcon {
    pub texture: Handle<Image>,
    pub size: Vec2,
    pub color: Color,
    pub loading_state: IconLoadingState,
}
```

#### Command Item Layout
- **Icon Section**: 16x16px colorful icons with app/command branding
- **Text Section**: 
  - **Primary Name**: White text, medium weight (e.g., "Search Snippets")
  - **Source**: Gray text, smaller font (e.g., "Snippets", "Raycast")
- **Metadata Section**:
  - **Alias Pill**: Monospace text in subtle background (e.g., "snip", "/quicklink")
  - **Type Label**: Right-aligned gray "Command" text

#### Visual Examples from Specification
1. **Search Snippets**: Red snippets icon, "snip" alias
2. **Kill Process**: Yellow warning icon, "kill" alias  
3. **Create Quicklink**: Red link icon, "/quicklink" alias
4. **Search Crates**: Yellow package icon, "/cargo-search" alias
5. **Webpage to Markdown**: Green conversion icon, no alias shown

### Context Action Menu System

#### Menu Structure and Animation
- **Background**: Dark overlay with rounded corners and subtle drop shadow
- **Expansion**: Smooth slide-up animation from selected item
- **Content**: Dynamic action list based on selected command
- **Dismissal**: Click outside or Escape key with fade-out animation

#### Menu Content Components
```rust
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct ContextMenuItem {
    pub action_type: ContextAction,
    pub display_text: String,
    pub keyboard_shortcut: Option<String>,
    pub icon: Option<Handle<Image>>,
    pub enabled: bool,
}

#[derive(Reflect)]
pub enum ContextAction {
    OpenCommand,        // ⏎ shortcut
    ResetRanking,      // No shortcut shown
    MoveDown,          // ⌃⌘↓ shortcut  
    RemoveFromFavorites, // ⌃⌘F shortcut
    MoveUp,            // ⌃⌘↑ shortcut
    SearchActions,     // Secondary search
}
```

#### Standard Context Actions Implementation
- **Title Section**: Selected command name with icon
- **Primary Action**: "Open Command" with Enter key symbol (⏎)
- **Secondary Actions**: Reset Ranking, Move Up/Down, Remove
- **Keyboard Shortcuts**: Right-aligned shortcut indicators
- **Search Integration**: "Search for actions" field at bottom

### Bottom Action Bar Interface

#### Action Bar Layout
- **Background**: Dark theme with top border separation
- **Left Section**: Primary action button with Enter key indicator
- **Right Section**: Actions button with ⌘K shortcut indicator
- **Search Field**: "Search for actions..." with matching search bar styling

#### Action Button Components
- **Open Command Button**: Left-aligned with ⏎ indicator
- **Actions ⌘K Button**: Right-aligned with keyboard shortcut
- **Visual Hierarchy**: Primary action prominence over secondary
- **Interactive States**: Hover and focus states for accessibility

### Animation and Micro-Interactions

#### Selection Animations
```rust
#[derive(Reflect)]
pub struct SelectionAnimation {
    pub highlight_color: Color,
    pub animation_duration: Duration,
    pub easing_function: EasingFunction,
    pub scale_factor: f32,
}
```

- **Selection Highlight**: Smooth background color transition
- **Hover Effects**: Subtle background lightening on interactive elements
- **Focus Indicators**: Clear accessibility-compliant focus rings
- **Transition Smoothness**: All animations maintain 60fps performance

#### Menu Animations
- **Context Menu Slide**: Smooth expansion from selected item position
- **Menu Dismissal**: Fade-out animation with scale reduction
- **Search Results**: Smooth filtering animations as user types
- **Icon Loading**: Progressive enhancement as icons load

### Icon Management System

#### Dynamic Icon Loading
- **Async Loading**: Non-blocking icon loading with fallbacks
- **Cache Management**: Efficient icon caching and memory management
- **High DPI Support**: Scalable icons for retina displays
- **Loading States**: Progressive enhancement with placeholders

#### Icon Categories and Sources
- **Application Icons**: System application bundle icons
- **Command Icons**: Extension-specific command branding
- **Status Icons**: UI state indicators (loading, error, success)
- **Shortcut Icons**: Keyboard shortcut visual representations

### Color Palette and Typography

#### Dark Theme Color System
```rust
#[derive(Reflect)]
pub struct ColorPalette {
    pub background_primary: Color,    // #1a1a1a
    pub background_secondary: Color,  // #2a2a2a
    pub text_primary: Color,         // #ffffff  
    pub text_secondary: Color,       // #888888
    pub accent_blue: Color,         // Focus/selection
    pub border_color: Color,        // rgba(255,255,255,0.1)
    pub success_green: Color,       // Success indicators
    pub warning_yellow: Color,      // Warning indicators
    pub error_red: Color,          // Error indicators
}
```

#### Typography System
- **Primary Text**: White color for command names and primary content
- **Secondary Text**: Medium gray for descriptions and metadata
- **Monospace Text**: For aliases, shortcuts, and technical content
- **Font Weights**: Medium weight for names, regular for descriptions
- **Font Sizing**: Hierarchical sizing for clear information hierarchy

### Performance Optimization

#### Efficient Rendering
- **Virtual Scrolling**: Render only visible list items for large lists
- **Texture Atlasing**: Efficient icon rendering with sprite atlases
- **Batched Updates**: Minimize individual UI component updates
- **Animation Culling**: Skip animations for non-visible elements

#### Memory Management
- **Icon Cache**: LRU cache for frequently accessed icons
- **Component Pooling**: Reuse UI components for list items
- **Animation Cleanup**: Proper cleanup of completed animations
- **Resource Loading**: Efficient loading and unloading of UI assets

### Integration Points

#### Search System Integration
- **Real-time Filtering**: Visual updates as search results change
- **Result Highlighting**: Highlight matching text in search results
- **Empty States**: Appropriate UI for no results or empty favorites
- **Loading States**: Visual indicators for search operations

#### Command System Integration
- **Execution Feedback**: Visual confirmation of command execution
- **Status Indicators**: Show command execution status and progress
- **Error Display**: User-friendly error message display
- **Success Confirmation**: Visual confirmation of successful operations

### Testing Requirements

#### Visual Testing
- **Pixel Perfect**: Verify UI matches specification exactly
- **Cross-Resolution**: Test on different screen resolutions and DPI settings
- **Animation Smoothness**: Verify all animations maintain 60fps
- **Color Accuracy**: Validate color palette matches specification

#### Interaction Testing
- **Hover States**: Test all interactive element hover states
- **Focus States**: Verify keyboard focus indicators work correctly
- **Click Feedback**: Test visual feedback for all clickable elements
- **Animation Performance**: Verify animations don't impact performance

### Implementation Files
- `actions_menu/visual_interface.rs` - Main visual interface system
- `actions_menu/animations.rs` - Animation system and micro-interactions
- `actions_menu/theming.rs` - Color palette and typography definitions
- `ui/icon_management.rs` - Icon loading and caching system
- `ui/list_rendering.rs` - Efficient list rendering and virtual scrolling

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

### Constraints
- **Never use `unwrap()`** in source code
- **Never use `expect()`** in source code (tests only)
- **Zero-allocation patterns** for all animation and rendering loops
- **Blazing-fast performance** - maintain 60fps during all animations
- **Production quality** - pixel-perfect visual implementation## Bevy Implementation Details

### Visual Interface Component Architecture

```rust
#[derive(Component, Reflect)]
pub struct ActionsMenuUI {
    pub search_bar_entity: Entity,
    pub favorites_list_entity: Entity,
    pub context_menu_entity: Option<Entity>,
    pub bottom_bar_entity: Entity,
    pub theme: InterfaceTheme,
    pub animation_state: UIAnimationState,
}

#[derive(Component, Reflect)]
pub struct AnimatedComponent {
    pub target_transform: Transform,
    pub animation_duration: Duration,
    pub easing_function: EasingType,
    pub start_time: Instant,
    pub animation_type: AnimationType,
}

#[derive(Component, Reflect)]
pub struct SearchBarUI {
    pub input_entity: Entity,
    pub ai_button_entities: Vec<Entity>,
    pub placeholder_visible: bool,
    pub focus_state: FocusState,
}
```

### Flex-Based Layout System

```rust
fn setup_actions_menu_layout(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    theme: Res<InterfaceTheme>,
) {
    // Main container with proper flex constraints
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            max_width: Val::Px(800.0),
            height: Val::Percent(100.0),
            max_height: Val::Px(600.0),
            flex_direction: FlexDirection::Column,
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 0.0, // Prevent expansion
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(theme.background_primary),
        ActionsMenuContainer,
    )).with_children(|parent| {
        // Search bar section
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(56.0),
                padding: UiRect::all(Val::Px(12.0)),
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                ..default()
            },
            BackgroundColor(theme.background_secondary),
            SearchBarContainer,
        ));
        
        // Favorites list with flexible height
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                flex_grow: 1.0,
                flex_direction: FlexDirection::Column,
                overflow: Overflow::scroll_y(),
                ..default()
            },
            FavoritesListContainer,
        ));
        
        // Bottom action bar
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(48.0),
                flex_direction: FlexDirection::Row,
                justify_content: JustifyContent::SpaceBetween,
                align_items: AlignItems::Center,
                flex_shrink: 0.0,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            BackgroundColor(theme.background_secondary),
            BottomActionBar,
        ));
    });
}
```

### Animation System Implementation

```rust
fn animate_ui_components(
    time: Res<Time>,
    mut animated_components: Query<(&mut Transform, &mut AnimatedComponent)>,
    mut commands: Commands,
) {
    for (mut transform, mut animation) in &mut animated_components {
        let elapsed = animation.start_time.elapsed();
        
        if elapsed >= animation.animation_duration {
            // Animation complete
            transform.translation = animation.target_transform.translation;
            transform.scale = animation.target_transform.scale;
            commands.entity(entity).remove::<AnimatedComponent>();
        } else {
            // Calculate interpolation progress
            let progress = elapsed.as_secs_f32() / animation.animation_duration.as_secs_f32();
            let eased_progress = apply_easing(progress, animation.easing_function);
            
            // Apply smooth interpolation
            transform.translation = transform.translation.lerp(
                animation.target_transform.translation,
                eased_progress
            );
            transform.scale = transform.scale.lerp(
                animation.target_transform.scale,
                eased_progress
            );
        }
    }
}

fn handle_selection_animations(
    mut selection_events: EventReader<SelectionEvent>,
    mut commands: Commands,
    command_items: Query<Entity, With<CommandItem>>,
) {
    for event in selection_events.read() {
        match event {
            SelectionEvent::ItemSelected(entity) => {
                // Add selection highlight animation
                commands.entity(*entity).insert(AnimatedComponent {
                    target_transform: Transform::from_scale(Vec3::splat(1.02)),
                    animation_duration: Duration::from_millis(200),
                    easing_function: EasingType::EaseOutQuad,
                    start_time: Instant::now(),
                    animation_type: AnimationType::SelectionHighlight,
                });
            },
            _ => {}
        }
    }
}
```

### Icon Management and Performance

```rust
#[derive(Component, Reflect)]
pub struct IconManager {
    pub icon_cache: HashMap<String, Handle<Image>>,
    pub loading_states: HashMap<String, IconLoadingState>,
    pub fallback_icons: HashMap<IconType, Handle<Image>>,
    pub cache_stats: IconCacheStats,
}

fn manage_icon_loading(
    mut icon_manager: ResMut<IconManager>,
    mut loading_tasks: Query<(Entity, &mut IconLoadingTask)>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    // Process completed icon loading tasks
    for (entity, mut task) in &mut loading_tasks {
        if let Some(icon_handle) = block_on(future::poll_once(&mut task.0)) {
            match icon_handle {
                Ok(handle) => {
                    icon_manager.icon_cache.insert(task.icon_id.clone(), handle.clone());
                    commands.entity(entity)
                        .remove::<IconLoadingTask>()
                        .insert(LoadedIcon(handle));
                    icon_manager.cache_stats.successful_loads += 1;
                },
                Err(_) => {
                    // Use fallback icon
                    let fallback = icon_manager.fallback_icons
                        .get(&IconType::Default)
                        .cloned()
                        .unwrap_or_default();
                    commands.entity(entity)
                        .remove::<IconLoadingTask>()
                        .insert(LoadedIcon(fallback));
                    icon_manager.cache_stats.failed_loads += 1;
                }
            }
        }
    }
}
```