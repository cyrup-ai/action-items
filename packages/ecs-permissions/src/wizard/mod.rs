//! Permission Setup Wizard Module
//!
//! Provides a comprehensive step-by-step wizard for setting up permissions
//! and configuring the application on first run. Integrates with ecs-ui for
//! elegant user interface and ecs-progress for robust progress tracking.

// Core wizard modules
pub mod states;
pub mod events;
pub mod components;
pub mod first_run;
pub mod plugin;

// System modules
pub mod systems;

// UI modules  
pub mod ui;

// Re-export commonly used types
pub use events::*;
pub use states::*;
pub use components::*;
pub use first_run::*;
pub use plugin::{PermissionWizardPlugin, WizardRequiredPermissions};

// System exports
pub use systems::*;