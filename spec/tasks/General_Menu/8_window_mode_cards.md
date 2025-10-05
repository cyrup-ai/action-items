# General Menu - Window Mode Selection Cards

## Task: Implement Visual Window Mode Selection Interface

### File: `ui/src/settings/general/window_mode_cards.rs` (new file)

Create visual selection cards for window mode configuration with rounded corner wireframes, interactive selection, and mode preview functionality.

### Implementation Requirements

#### Window Mode Card Component
```rust
#[derive(Component)]
pub struct WindowModeCard {
    pub mode_type: WindowModeType,
    pub is_selected: bool,
    pub preview_image: Handle<Image>,
    pub card_style: CardStyle,
    pub wireframe_corners: CornerRadius,
}
```

#### Visual Selection Card System
- File: `ui/src/settings/general/window_mode_cards.rs` (line 1-156)
- Implement Default Mode card with purple gradient styling
- Implement Compact Mode card with gray minimalist design
- **Rounded corner wireframes** within each card showing UI mockups
- Interactive card selection with highlighted border states

#### Wireframe Rendering System
- File: `ui/src/settings/general/wireframe_renderer.rs` (new file, line 1-89)
- **Rounded corner UI mockup rendering** for both modes
- Default mode wireframe: complete interface layout with rounded corners
- Compact mode wireframe: streamlined interface layout with rounded corners
- Dynamic wireframe generation based on current application state

#### Card Interaction System
- File: `ui/src/settings/general/card_interaction.rs` (new file, line 1-67)
- Mouse hover effects for card selection
- Click handling for mode switching
- Visual feedback for selected state
- Animation transitions between selection states

#### Mode Preview Integration
- File: `ui/src/settings/general/mode_preview.rs` (new file, line 1-123)
- Real-time preview generation for window modes
- Integration with existing window management system
- Preview accuracy validation against actual modes
- Performance optimization for preview rendering

### Architecture Notes
- Use Bevy's `Handle<Image>` for card preview images
- Implement `BorderRadius` component for rounded corner styling
- Integration with window management system for mode switching
- Card selection state synchronization with settings

### Visual Design Requirements
- **Default Mode Card**: Purple gradient background with rounded corner wireframe
- **Compact Mode Card**: Gray background with simplified rounded corner wireframe
- Selection highlight: distinct border styling for active card
- Consistent card sizing and spacing
- Accessibility compliance for card selection

### Integration Points
- Window management system for actual mode switching
- Settings persistence for selected mode
- Animation system for smooth mode transitions
- Existing UI theme system for consistent styling

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Window Mode Card Components

```rust
// Core window mode card component with Reflect support
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct WindowModeCard {
    pub mode_type: WindowModeType,
    pub is_selected: bool,
    pub preview_image: Handle<Image>,
    pub card_style: CardStyle,
    pub wireframe_corners: CornerRadius,
    pub hover_state: HoverState,
    pub selection_animation_progress: f32,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum WindowModeType {
    Default,
    Compact,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct CardStyle {
    pub background_gradient: LinearGradient,
    pub border_color: Color,
    pub selected_border_color: Color,
    pub hover_border_color: Color,
    pub corner_radius: f32,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum HoverState {
    None,
    Hovering,
    Selected,
}
```

### Wireframe Rendering Components

```rust
// Components for wireframe rendering within cards
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct WireframeRenderer {
    pub mode_type: WindowModeType,
    pub wireframe_elements: Vec<WireframeElement>,
    pub render_scale: f32,
    pub corner_style: WireframeCornerStyle,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WireframeElement {
    pub element_type: WireframeElementType,
    pub position: Vec2,
    pub size: Vec2,
    pub corner_radius: f32,
    pub color: Color,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum WireframeElementType {
    SearchBar,
    ResultItem,
    MenuBar,
    FavoritesPanel,
    StatusBar,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub struct WireframeCornerStyle {
    pub radius: f32,
    pub stroke_width: f32,
    pub stroke_color: Color,
    pub fill_color: Color,
}
```

### Window Mode Card System Sets

```rust
// System sets for window mode card management
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum WindowModeCardSystemSet {
    SpawnCards,           // Spawn mode selection cards
    UpdateWireframes,     // Update wireframe previews
    HandleInteraction,    // Handle mouse interaction
    AnimateSelection,     // Animate selection states
    UpdatePreviews,       // Update mode previews
    SyncSettings,         // Sync with settings
}

// Window Mode Cards Plugin
pub struct WindowModeCardsPlugin;

impl Plugin for WindowModeCardsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<WindowModeCardSettings>()
            .init_resource::<WireframeAssets>()
            
            // Events
            .add_event::<WindowModeSelectedEvent>()
            .add_event::<CardHoverEvent>()
            .add_event::<ModePreviewGeneratedEvent>()
            
            // Component registration
            .register_type::<WindowModeCard>()
            .register_type::<WireframeRenderer>()
            .register_type::<WindowModeType>()
            .register_type::<CardStyle>()
            .register_type::<CornerRadius>()
            
            // System sets configuration
            .configure_sets(
                Update,
                (
                    WindowModeCardSystemSet::SpawnCards,
                    WindowModeCardSystemSet::UpdateWireframes,
                    WindowModeCardSystemSet::HandleInteraction,
                    WindowModeCardSystemSet::AnimateSelection,
                    WindowModeCardSystemSet::UpdatePreviews,
                    WindowModeCardSystemSet::SyncSettings,
                ).chain()
            )
            
            // Window mode card systems
            .add_systems(Update, (
                spawn_window_mode_cards.in_set(WindowModeCardSystemSet::SpawnCards),
                update_wireframe_previews.in_set(WindowModeCardSystemSet::UpdateWireframes),
                handle_card_interaction.in_set(WindowModeCardSystemSet::HandleInteraction),
                animate_card_selection.in_set(WindowModeCardSystemSet::AnimateSelection),
                update_mode_previews.in_set(WindowModeCardSystemSet::UpdatePreviews),
                sync_window_mode_settings.in_set(WindowModeCardSystemSet::SyncSettings),
            ))
            
            // Startup systems
            .add_systems(Startup, (
                setup_window_mode_card_assets,
                initialize_wireframe_templates,
            ));
    }
}
```

### Card Spawning System with Proper Flex Layout

```rust
// System to spawn window mode selection cards
fn spawn_window_mode_cards(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
    theme: Res<CurrentTheme>,
    card_settings: Res<WindowModeCardSettings>,
    existing_cards: Query<Entity, With<WindowModeCard>>,
) {
    // Only spawn cards once
    if !existing_cards.is_empty() {
        return;
    }
    
    // Container for both cards with proper flex constraints
    let cards_container = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_height: Val::Px(200.0), // Constrain height
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceEvenly,
            column_gap: Val::Px(20.0),
            padding: UiRect::all(Val::Px(16.0)),
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            ..default()
        },
        WindowModeCardContainer,
    )).id();
    
    // Default Mode Card with purple gradient
    let default_card = spawn_mode_card(
        &mut commands,
        &asset_server,
        &mut images,
        &theme,
        WindowModeType::Default,
        CardStyle {
            background_gradient: LinearGradient::new(
                Color::srgb(0.6, 0.4, 0.8), // Purple start
                Color::srgb(0.4, 0.2, 0.6), // Purple end
            ),
            border_color: Color::srgb(0.5, 0.3, 0.7),
            selected_border_color: Color::srgb(0.8, 0.6, 1.0),
            hover_border_color: Color::srgb(0.7, 0.5, 0.9),
            corner_radius: 12.0,
        },
        true, // Initially selected
    );
    
    // Compact Mode Card with gray minimalist design
    let compact_card = spawn_mode_card(
        &mut commands,
        &asset_server,
        &mut images,
        &theme,
        WindowModeType::Compact,
        CardStyle {
            background_gradient: LinearGradient::new(
                Color::srgb(0.9, 0.9, 0.9), // Light gray start
                Color::srgb(0.7, 0.7, 0.7), // Gray end
            ),
            border_color: Color::srgb(0.6, 0.6, 0.6),
            selected_border_color: Color::srgb(0.4, 0.4, 0.4),
            hover_border_color: Color::srgb(0.5, 0.5, 0.5),
            corner_radius: 12.0,
        },
        false, // Not initially selected
    );
    
    commands.entity(cards_container)
        .add_child(default_card)
        .add_child(compact_card);
}

// Function to spawn individual mode cards
fn spawn_mode_card(
    commands: &mut Commands,
    asset_server: &AssetServer,
    images: &mut Assets<Image>,
    theme: &CurrentTheme,
    mode_type: WindowModeType,
    card_style: CardStyle,
    is_selected: bool,
) -> Entity {
    // Generate wireframe preview for this mode
    let wireframe_image = generate_wireframe_preview(images, &mode_type, &card_style);
    
    // Card container with proper flex constraints
    let card_entity = commands.spawn((
        Node {
            width: Val::Px(280.0),
            height: Val::Px(180.0),
            max_width: Val::Px(300.0), // Prevent expansion
            max_height: Val::Px(200.0), // Prevent expansion
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::SpaceBetween,
            padding: UiRect::all(Val::Px(16.0)),
            border: UiRect::all(Val::Px(2.0)),
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            flex_shrink: 0.0, // Maintain size
            ..default()
        },
        BackgroundColor(card_style.background_gradient.start_color()),
        BorderColor(if is_selected { 
            card_style.selected_border_color 
        } else { 
            card_style.border_color 
        }),
        BorderRadius::all(Val::Px(card_style.corner_radius)),
        WindowModeCard {
            mode_type: mode_type.clone(),
            is_selected,
            preview_image: wireframe_image,
            card_style: card_style.clone(),
            wireframe_corners: CornerRadius {
                top_left: card_style.corner_radius,
                top_right: card_style.corner_radius,
                bottom_left: card_style.corner_radius,
                bottom_right: card_style.corner_radius,
            },
            hover_state: if is_selected { HoverState::Selected } else { HoverState::None },
            selection_animation_progress: if is_selected { 1.0 } else { 0.0 },
        },
        Interaction::default(),
    )).id();
    
    // Card title
    let title_text = match mode_type {
        WindowModeType::Default => "Default",
        WindowModeType::Compact => "Compact",
    };
    
    let title_entity = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Auto,
            max_width: Val::Px(250.0), // Constrain text width
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            flex_grow: 0.0,
            ..default()
        },
        Text::new(title_text),
        TextFont {
            font: asset_server.load("fonts/Inter-Bold.ttf"),
            font_size: 16.0,
        },
        TextColor(Color::WHITE),
    )).id();
    
    // Wireframe preview container
    let wireframe_container = commands.spawn((
        Node {
            width: Val::Px(200.0),
            height: Val::Px(120.0),
            max_width: Val::Px(220.0), // Constrain wireframe size
            max_height: Val::Px(140.0),
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            border: UiRect::all(Val::Px(1.0)),
            flex_grow: 0.0,
            ..default()
        },
        BackgroundColor(Color::srgba(1.0, 1.0, 1.0, 0.1)),
        BorderColor(Color::srgba(1.0, 1.0, 1.0, 0.3)),
        BorderRadius::all(Val::Px(8.0)),
        WireframeRenderer {
            mode_type: mode_type.clone(),
            wireframe_elements: generate_wireframe_elements(&mode_type),
            render_scale: 0.6, // Scale down for card preview
            corner_style: WireframeCornerStyle {
                radius: 4.0,
                stroke_width: 1.0,
                stroke_color: Color::srgba(1.0, 1.0, 1.0, 0.8),
                fill_color: Color::srgba(1.0, 1.0, 1.0, 0.2),
            },
        },
    )).id();
    
    commands.entity(card_entity)
        .add_child(title_entity)
        .add_child(wireframe_container);
    
    card_entity
}
```

### Wireframe Generation System

```rust
// Function to generate wireframe elements for different modes
fn generate_wireframe_elements(mode_type: &WindowModeType) -> Vec<WireframeElement> {
    match mode_type {
        WindowModeType::Default => vec![
            // Search bar with rounded corners
            WireframeElement {
                element_type: WireframeElementType::SearchBar,
                position: Vec2::new(20.0, 20.0),
                size: Vec2::new(160.0, 24.0),
                corner_radius: 12.0,
                color: Color::srgba(1.0, 1.0, 1.0, 0.4),
            },
            // Result items with rounded corners
            WireframeElement {
                element_type: WireframeElementType::ResultItem,
                position: Vec2::new(20.0, 55.0),
                size: Vec2::new(160.0, 16.0),
                corner_radius: 4.0,
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
            },
            WireframeElement {
                element_type: WireframeElementType::ResultItem,
                position: Vec2::new(20.0, 75.0),
                size: Vec2::new(160.0, 16.0),
                corner_radius: 4.0,
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
            },
            WireframeElement {
                element_type: WireframeElementType::ResultItem,
                position: Vec2::new(20.0, 95.0),
                size: Vec2::new(160.0, 16.0),
                corner_radius: 4.0,
                color: Color::srgba(1.0, 1.0, 1.0, 0.3),
            },
        ],
        WindowModeType::Compact => vec![
            // Compact search bar
            WireframeElement {
                element_type: WireframeElementType::SearchBar,
                position: Vec2::new(30.0, 30.0),
                size: Vec2::new(140.0, 20.0),
                corner_radius: 10.0,
                color: Color::srgba(0.3, 0.3, 0.3, 0.6),
            },
            // Fewer, smaller result items
            WireframeElement {
                element_type: WireframeElementType::ResultItem,
                position: Vec2::new(30.0, 60.0),
                size: Vec2::new(140.0, 14.0),
                corner_radius: 3.0,
                color: Color::srgba(0.3, 0.3, 0.3, 0.4),
            },
            WireframeElement {
                element_type: WireframeElementType::ResultItem,
                position: Vec2::new(30.0, 80.0),
                size: Vec2::new(140.0, 14.0),
                corner_radius: 3.0,
                color: Color::srgba(0.3, 0.3, 0.3, 0.4),
            },
        ],
    }
}

// System to update wireframe previews
fn update_wireframe_previews(
    mut wireframe_query: Query<&mut WireframeRenderer, Changed<WireframeRenderer>>,
    mut gizmos: Gizmos,
) {
    for wireframe_renderer in wireframe_query.iter_mut() {
        // Render wireframe elements with rounded corners
        for element in &wireframe_renderer.wireframe_elements {
            // Draw rounded rectangles for wireframe elements
            draw_rounded_wireframe_rect(
                &mut gizmos,
                element.position,
                element.size,
                element.corner_radius,
                element.color,
                wireframe_renderer.render_scale,
            );
        }
    }
}

// Helper function to draw rounded wireframe rectangles
fn draw_rounded_wireframe_rect(
    gizmos: &mut Gizmos,
    position: Vec2,
    size: Vec2,
    corner_radius: f32,
    color: Color,
    scale: f32,
) {
    let scaled_pos = position * scale;
    let scaled_size = size * scale;
    let scaled_radius = corner_radius * scale;
    
    // Draw rounded rectangle outline
    // This would use Bevy's gizmo system to draw the wireframe
    // For now, draw a simple rectangle
    let rect = Rect::from_corners(
        scaled_pos,
        scaled_pos + scaled_size,
    );
    
    gizmos.rect_2d(rect.center(), rect.size(), color);
}
```

### Card Interaction System

```rust
// System to handle card interaction
fn handle_card_interaction(
    mut card_query: Query<(&mut WindowModeCard, &mut BorderColor, &Interaction), Changed<Interaction>>,
    mut events: EventWriter<WindowModeSelectedEvent>,
    mut hover_events: EventWriter<CardHoverEvent>,
) {
    for (mut card, mut border_color, interaction) in card_query.iter_mut() {
        match interaction {
            Interaction::Pressed => {
                // Select this card and deselect others
                if !card.is_selected {
                    card.is_selected = true;
                    card.hover_state = HoverState::Selected;
                    card.selection_animation_progress = 0.0; // Start animation
                    *border_color = BorderColor(card.card_style.selected_border_color);
                    
                    events.send(WindowModeSelectedEvent {
                        mode_type: card.mode_type.clone(),
                        previous_selection: None, // Would track previous selection
                    });
                }
            },
            Interaction::Hovered => {
                if !card.is_selected {
                    card.hover_state = HoverState::Hovering;
                    *border_color = BorderColor(card.card_style.hover_border_color);
                    
                    hover_events.send(CardHoverEvent {
                        mode_type: card.mode_type.clone(),
                        hover_started: true,
                    });
                }
            },
            Interaction::None => {
                if !card.is_selected {
                    card.hover_state = HoverState::None;
                    *border_color = BorderColor(card.card_style.border_color);
                    
                    hover_events.send(CardHoverEvent {
                        mode_type: card.mode_type.clone(),
                        hover_started: false,
                    });
                }
            },
        }
    }
}

// System to animate card selection
fn animate_card_selection(
    mut card_query: Query<&mut WindowModeCard>,
    time: Res<Time>,
) {
    for mut card in card_query.iter_mut() {
        if card.is_selected && card.selection_animation_progress < 1.0 {
            card.selection_animation_progress += time.delta_seconds() * 3.0; // Animation speed
            card.selection_animation_progress = card.selection_animation_progress.min(1.0);
        } else if !card.is_selected && card.selection_animation_progress > 0.0 {
            card.selection_animation_progress -= time.delta_seconds() * 3.0;
            card.selection_animation_progress = card.selection_animation_progress.max(0.0);
        }
    }
}
```

### Event System for Window Mode Cards

```rust
// Events for window mode card communication
#[derive(Event, Debug, Clone)]
pub struct WindowModeSelectedEvent {
    pub mode_type: WindowModeType,
    pub previous_selection: Option<WindowModeType>,
}

#[derive(Event, Debug, Clone)]
pub struct CardHoverEvent {
    pub mode_type: WindowModeType,
    pub hover_started: bool,
}

#[derive(Event, Debug, Clone)]
pub struct ModePreviewGeneratedEvent {
    pub mode_type: WindowModeType,
    pub preview_image: Handle<Image>,
}

// Resource for managing window mode card settings
#[derive(Resource, Debug)]
pub struct WindowModeCardSettings {
    pub current_selection: WindowModeType,
    pub animation_duration: Duration,
    pub card_spacing: f32,
    pub preview_update_interval: Duration,
}

impl Default for WindowModeCardSettings {
    fn default() -> Self {
        Self {
            current_selection: WindowModeType::Default,
            animation_duration: Duration::from_millis(300),
            card_spacing: 20.0,
            preview_update_interval: Duration::from_secs(1),
        }
    }
}
```

This comprehensive window mode card system provides visual selection cards with rounded corner wireframes, interactive selection, smooth animations, and proper integration with the window management system using Bevy's ECS architecture and flex-based UI layout with proper constraints.