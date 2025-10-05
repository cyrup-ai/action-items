//! UI systems module
//!
//! Zero-allocation UI systems with blazing-fast component management and interaction handling.

// Re-export all UI systems (now all are used)
pub use animations::{animate_hover_effects_system, animate_result_items_system};
pub use gradients::gradient_selection_system;
pub use hover_effects::{
    handle_keyboard_selection_highlighting_system, handle_result_item_hover_system,
};
pub use icons::{
    initialize_icon_system_system as initialize_icon_system, update_result_icons_system,
};
pub use interactions::{
    handle_keyboard_input_system as handle_keyboard_input,
    handle_result_item_click_system as handle_result_item_click,
};
pub use monitor_constraints::MonitorConstraintsPlugin;
pub use professional_results::{
    handle_result_hover_effects, handle_result_selection, render_professional_results,
};
pub use responsive::adaptive_container_system;
pub use search_input::{SearchQueryChanged, results_visibility_system, search_input_system};
pub use setup::{load_fallback_icon_system, setup_ui_system as setup_ui};
pub use visibility::handle_visibility_animation_complete;

// Module declarations
pub mod animations;
pub mod gradients;
pub mod hover_effects;
pub mod icons;
pub mod interactions;
pub mod monitor_constraints;
pub mod professional_results;
pub mod responsive;
pub mod search_input;
pub mod setup;
pub mod visibility;

// Type aliases for backward compatibility
