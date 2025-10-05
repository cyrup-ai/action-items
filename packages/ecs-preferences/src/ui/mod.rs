pub mod components;
pub mod plugin;
pub mod screens;
pub mod systems;

// Re-export UI plugin
pub use plugin::PreferencesUIPlugin;

// Re-export UI components
pub use components::*;
