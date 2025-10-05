use bevy::prelude::*;

// Import gradient primitives from ecs-ui colors submodule
// (not re-exported at theme level, must import from colors directly)
// These are automatically re-exported as layout::* for use by other modules
pub use action_items_ecs_ui::theme::colors::{
    BackgroundGradient,
    ColorStop,
    GradientFactory,
    LinearGradient,
};

#[derive(Component)]
pub struct LauncherContainer;

#[derive(Component)]
pub struct UiRoot;

// Re-export responsive components from ecs-ui
pub use action_items_ecs_ui::responsive::{
    ContentConstraints,
    TextTruncation,
    ViewportResponsiveContainer,
};
