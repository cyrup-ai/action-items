#![recursion_limit = "256"]

use tracing::info;
use bevy::prelude::Startup;

// Hotkey registration now handled by ECS service

// Import all other modules
mod app_main;
mod events;
mod hotkeys;
mod input;
mod overlay_window;
// Permissions now handled by ECS service
// Preferences now handled by ecs-preferences service
mod search;
// mod tasks; // Removed - using ECS task management service instead
// UI now handled by ECS service
mod window;
// Wizard now handled by ecs-permissions service

use app_main::{add_post_startup_systems, add_startup_systems, add_update_systems, configure_app};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Note: Logging now handled by Bevy's LogPlugin with file output
    // Configured in configure_app() with comprehensive panic capture

    // Configure the main Bevy app - ECS HotkeyPlugin handles hotkey registration
    let mut app = configure_app();
    
    // Add startup logging system to verify logging is working
    app.add_systems(Startup, || {
        info!("Action Items v{} starting with Bevy LogPlugin + file output", env!("CARGO_PKG_VERSION"));
        info!("Log files are written to ~/.config/action-items/logs/action-items.log");
        info!("Panic stack traces are automatically captured with trace feature enabled");
    });

    // Add all system schedules
    add_startup_systems(&mut app);
    add_post_startup_systems(&mut app);
    add_update_systems(&mut app);

    // Run the application
    app.run();

    Ok(())
}
