#![recursion_limit = "1024"]
pub mod ui;

// Re-export systems modules for external access
// Re-export the public interface
use bevy::prelude::*;
use action_items_ecs_ui::accessibility::AccessibilityPlugin;
use action_items_ecs_ui::performance::PerformancePlugin;
use ui::accessibility::{setup_accessibility, update_accessibility_announcements};
use ui::icons::extraction::poll_icon_extraction_tasks;
use ui::icons::process_icon_extraction_requests;
pub use ui::systems::MonitorConstraintsPlugin;
// PHASE 3: Re-export SearchQueryChanged for main app access
pub use ui::systems::SearchQueryChanged;
use ui::systems::{
    // Responsive container system (Phase 1 according to specification)
    adaptive_container_system, // Updated responsive container system
    animate_hover_effects_system,
    animate_result_items_system,
    handle_keyboard_input,
    handle_result_item_click,
    handle_visibility_animation_complete,  // App-specific visibility completion handler
    initialize_icon_system,
    gradient_selection_system, // App-specific gradient system (generic ones in GradientPlugin)
    load_fallback_icon_system, // PostStartup icon loading system
    // Professional results display systems
    render_professional_results,
    results_visibility_system,
    // PHASE 3: Real KeyboardInput search systems (SearchQueryChanged exported above)
    search_input_system,
    setup_ui,
    // Animation and icon systems (functions that can be used as Bevy systems)
    update_result_icons_system,
};
// Re-export theme, typography, gradients, responsive layout, and visibility system for main app
pub use action_items_ecs_ui::gradients::GradientPlugin;
pub use action_items_ecs_ui::responsive::ResponsivePlugin;
pub use action_items_ecs_ui::theme::{ShadowElevation, SpacingScale, Theme};
pub use action_items_ecs_ui::visibility::{
    UiAnimationCompleteEvent, UiComponentTarget, UiVisibilityAnimation,
    UiVisibilityAnimationType, UiVisibilityEvent, VisibilityPlugin,
};
pub use ui::typography::{TextBundleBuilder, TypographyScale};
pub use ui::{
    LauncherIconCache,  // App-specific wrapper (keep)
    PrivacyConfiguration, PrivacyIndicatorPlugin, PrivacyIndicators,
    UiState, set_ui_visibility, systems,
};
// Re-export ecs-ui icon types (expanded set)
pub use action_items_ecs_ui::icons::{
    IconSize, IconType, IconTheme, ThemeColors, FontAwesome,
    IconExtractionRequest, IconExtractionResult,
};

pub struct LauncherUiPlugin;

impl Plugin for LauncherUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiState>()
            .init_resource::<LauncherIconCache>()
            .init_resource::<IconTheme>()
            .init_resource::<FontAwesome>()
            // Add performance monitoring plugin from ecs-ui
            .add_plugins(PerformancePlugin)
            // Add gradient system plugin from ecs-ui
            .add_plugins(GradientPlugin)
            // Add responsive layout plugin from ecs-ui
            .add_plugins(ResponsivePlugin)
            // Add visibility animation plugin from ecs-ui
            .add_plugins(VisibilityPlugin)
            // Add accessibility plugin from ecs-ui
            .add_plugins(AccessibilityPlugin)
            // Add privacy indicators plugin
            .add_plugins(ui::ai_menu::PrivacyIndicatorPlugin)
            .add_event::<IconExtractionRequest>()
            .add_event::<IconExtractionResult>()
            .add_event::<SearchQueryChanged>()
            .add_systems(Startup, setup_ui.before(initialize_icon_system))
            .add_systems(PostStartup, load_fallback_icon_system)
            // Core UI input systems
            .add_systems(
                Update,
                (
                    handle_keyboard_input,
                    handle_result_item_click,
                    search_input_system,
                    results_visibility_system,
                ),
            )
            // Professional results display systems
            .add_systems(
                Update,
                render_professional_results,
            )
            // UI visibility completion handler (generic animation is in VisibilityPlugin)
            .add_systems(Update, handle_visibility_animation_complete)
            // Application-specific responsive container adaptation
            .add_systems(Update, adaptive_container_system)
            // Icon extraction systems
            .add_systems(
                Update,
                (process_icon_extraction_requests, poll_icon_extraction_tasks),
            )
            // Icon and animation systems
            .add_systems(Update, update_result_icons_system)
            .add_systems(Update, animate_result_items_system)
            .add_systems(Update, animate_hover_effects_system)
            // Note: animate_window_visibility_system is now provided by VisibilityPlugin
            // App-specific gradient system (generic gradient systems are in GradientPlugin)
            .add_systems(Update, gradient_selection_system)
            // UI-specific accessibility systems
            .add_systems(
                Update,
                (
                    setup_accessibility,
                    update_accessibility_announcements,
                ),
            );
    }
}

pub mod prelude {
    pub use crate::ui::components::{set_ui_visibility, *};
    pub use crate::ui::icons::*;
    pub use crate::ui::systems::{
        adaptive_container_system,
    };
}
