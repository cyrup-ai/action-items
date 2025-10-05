//! UI resource types and event definitions for the Lunex UI system

use std::collections::{HashMap, HashSet};

use crate::{Rectangle2D, *};

/// Resource for tracking dirty layout roots for optimized computation
#[derive(Resource, Default)]
pub struct DirtyLayout {
    pub dirty_roots: HashSet<Entity>,
}

/// Resource for caching computed layout rectangles
#[derive(Resource, Default)]
pub struct LayoutCache {
    pub cache: HashMap<Entity, Rectangle2D>,
}

/// Global UI theme defining default styles for elements like buttons, panels, and text.
/// Insert into the app to customize app-wide appearance:
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_lunex::prelude::*;
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(UiTheme {
///         default_color: UiColor::from(bevy_color::Color::srgb(0.2, 0.2, 0.8)),
///         default_font_size: 24.0,
///         button_padding: UiValue::from(Vec2::new(20.0, 10.0)),
///         panel_margin: UiValue::from(Vec2::new(10.0, 10.0)),
///     });
/// }
/// ```
#[derive(Resource, Reflect, Clone)]
#[reflect(Resource)]
pub struct UiTheme {
    /// Default color for UI elements when no color is specified
    pub default_color: crate::UiColor,
    /// Default font size for text elements
    pub default_font_size: f32,
    /// Default padding for button elements
    pub button_padding: Vec2,
    /// Default margin for panel elements  
    pub panel_margin: Vec2,
}

impl Default for UiTheme {
    fn default() -> Self {
        Self {
            default_color: crate::UiColor::from(Color::srgb(0.8, 0.8, 0.8)),
            default_font_size: 16.0,
            button_padding: Vec2::new(16.0, 8.0),
            panel_margin: Vec2::new(8.0, 8.0),
        }
    }
}

/// Event emitted for UI interactions, allowing decoupled handling.
///
/// ## 🛠️ Example
/// ```
/// # use bevy_ecs::prelude::*;
/// # use bevy_lunex::prelude::*;
/// fn handle_ui_events(mut events: EventReader<UiEvent>) {
///     for event in events.read() {
///         match event {
///             UiEvent::Hover(entity) => println!("Hovered entity: {:?}", entity),
///             UiEvent::Click(entity) => println!("Clicked entity: {:?}", entity),
///             UiEvent::Select(entity) => println!("Selected entity: {:?}", entity),
///             UiEvent::Focus(entity) => println!("Focused entity: {:?}", entity),
///         }
///     }
/// }
/// ```
#[derive(Event, Clone)]
pub enum UiEvent {
    /// Fired when an element is hovered
    Hover(Entity),
    /// Fired when an element is clicked
    Click(Entity),
    /// Fired when an element is selected
    Select(Entity),
    /// Fired when an element gains focus
    Focus(Entity),
}

/// Trigger this event to recompute all [`UiLayoutRoot`] entities.
#[derive(Event)]
pub struct RecomputeUiLayout;

/// Event for theme changes
#[derive(Event)]
pub struct UiThemeChanged;

/// Observer that triggers layout recomputation when theme changes
pub fn observer_theme_changed(_trigger: Trigger<UiThemeChanged>, mut commands: Commands) {
    commands.trigger(RecomputeUiLayout);
}

/// System that triggers layout recomputation when UiTheme resource changes
pub fn system_recompute_on_theme_change(theme: Res<UiTheme>, mut commands: Commands) {
    if theme.is_changed() && !theme.is_added() {
        commands.trigger(RecomputeUiLayout);
    }
}
