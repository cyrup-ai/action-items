//! Execution context management

use crate::raycast::wrapper::RaycastPluginComponent;

/// Build raycast command arguments
pub fn build_raycast_command_args(plugin: &RaycastPluginComponent, action_id: &str) -> Vec<String> {
    // Build optimized command arguments for Raycast execution
    let mut args = Vec::with_capacity(4);

    // Add plugin identifier
    args.push(format!("--plugin={}", plugin.id));

    // Add command/action identifier
    args.push(format!("--command={}", action_id));

    // Add plugin path for execution context
    args.push(format!("--path={}", plugin.path.display()));

    // Add execution mode
    args.push("--mode=execute".to_string());

    args
}
