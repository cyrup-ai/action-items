use bevy::prelude::*;
use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::theme::Theme;
use super::{screens::*, systems::*, components::*};

/// Optional UI plugin for ecs-search
/// 
/// Provides a visual search interface using ecs-ui patterns.
/// Requires Theme resource to be present in the app.
pub struct SearchUIPlugin {
    /// Show search UI on startup
    pub show_on_startup: bool,
    /// Maximum number of results to display
    pub max_results_displayed: usize,
}

impl Default for SearchUIPlugin {
    fn default() -> Self {
        Self {
            show_on_startup: false,
            max_results_displayed: 50,
        }
    }
}

impl Plugin for SearchUIPlugin {
    fn build(&self, app: &mut App) {
        // Initialize resources
        app.init_resource::<SearchSelection>();
        
        // Setup systems
        app.add_systems(Startup, setup_search_ui);
        
        // Update systems
        app.add_systems(Update, (
            // Input handling
            handle_search_input_changes,
            handle_keyboard_navigation,
            
            // Results display
            update_results_on_completion,
            handle_result_click_selection,
            
            // Visual effects
            animate_results_appear,
            update_result_hover_colors,
            animate_search_loading,
            update_focus_indicators,
            update_search_bar_loading_state,
        ));
    }
}

/// Setup system that creates the search UI
/// 
/// Spawns search bar and results container using screen generators.
/// Theme must be present as a Resource.
fn setup_search_ui(
    mut commands: Commands,
    theme: Res<Theme>,
) {
    // Create root entity for search UI
    let search_root = commands.spawn((
        UiLayout::window()
            .size((Vw(100.0), Vh(100.0)))
            .pos((Vw(0.0), Vh(0.0)))
            .pack(),
        Visibility::Visible,
        Name::new("SearchUIRoot"),
    )).id();
    
    // Create search bar (visible by default)
    let search_bar = create_search_bar_screen(&mut commands, &theme);
    commands.entity(search_bar).insert(ChildOf(search_root));
    
    // Create results container
    let results_container = create_results_container(&mut commands, &theme);
    commands.entity(results_container).insert(ChildOf(search_root));
}
