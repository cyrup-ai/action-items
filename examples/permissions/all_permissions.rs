//! Comprehensive permissions example
//! 
//! Shows how to check all supported permissions and handle various scenarios.

use action_items_ecs_permissions::{PermissionType, PermissionStatus, PermissionPlugin, PermissionResource};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PermissionPlugin)
        .add_systems(Startup, check_all_permissions)
        .add_systems(Update, (
            handle_keyboard_input,
            monitor_permission_changes,
        ))
        .run();
}

fn check_all_permissions(permission_res: Res<PermissionResource>) {
    info!("=== Permission Status Report ===");
    
    let supported = permission_res.supported_permissions();
    info!("Platform supports {} permission types", supported.len());
    
    for permission_type in supported {
        match permission_res.check_permission(permission_type) {
            Ok(status) => {
                let status_icon = match status {
                    PermissionStatus::Authorized => "✅",
                    PermissionStatus::Denied => "❌", 
                    PermissionStatus::NotDetermined => "❓",
                    PermissionStatus::Restricted => "🔒",
                    PermissionStatus::Unknown => "❔",
                };
                info!("{} {}: {}", status_icon, permission_type, status);
            }
            Err(e) => {
                error!("❗ {}: Error - {}", permission_type, e);
            }
        }
    }
    
    info!("=== Controls ===");
    info!("Press R to refresh all permissions");
    info!("Press C to clear permission cache");
    info!("Press ESC to exit");
}

fn handle_keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    permission_res: Res<PermissionResource>,
    mut exit: EventWriter<AppExit>,
) {
    if input.just_pressed(KeyCode::KeyR) {
        info!("Refreshing all permissions...");
        check_all_permissions(permission_res);
    }
    
    if input.just_pressed(KeyCode::KeyC) {
        info!("Clearing permission cache...");
        permission_res.clear_cache();
        info!("Cache cleared. Press R to refresh.");
    }
    
    if input.just_pressed(KeyCode::Escape) {
        info!("Exiting...");
        exit.send(AppExit::Success);
    }
}

fn monitor_permission_changes(
    mut events: EventReader<action_items_ecs_permissions::PermissionChanged>,
) {
    for event in events.read() {
        let change_type = if event.old_status.is_some() {
            "changed"
        } else {
            "determined"
        };
        
        info!(
            "Permission {} for {}: {} -> {}",
            change_type,
            event.permission_type,
            event.old_status.map_or("Unknown".to_string(), |s| s.to_string()),
            event.new_status
        );
    }
}