//! Camera permission example
//! 
//! Demonstrates requesting camera access with proper error handling.

use action_items_ecs_permissions::{PermissionType, PermissionStatus, PermissionPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PermissionPlugin)
        .add_systems(Startup, request_camera_permission)
        .add_systems(Update, handle_permission_changes)
        .run();
}

fn request_camera_permission(
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
) {
    match permission_res.check_permission(PermissionType::Camera) {
        Ok(PermissionStatus::Authorized) => {
            info!("Camera access already granted!");
        }
        Ok(PermissionStatus::Denied) => {
            warn!("Camera access was denied. Please enable in System Preferences.");
        }
        Ok(PermissionStatus::NotDetermined) => {
            info!("Camera permission not determined, requesting access...");
            // In a real app, you would trigger a permission request here
        }
        Ok(status) => {
            info!("Camera permission status: {}", status);
        }
        Err(e) => {
            error!("Failed to check camera permission: {}", e);
        }
    }
}

fn handle_permission_changes(
    mut events: EventReader<action_items_ecs_permissions::PermissionChanged>,
) {
    for event in events.read() {
        if event.permission_type == PermissionType::Camera {
            match event.new_status {
                PermissionStatus::Authorized => {
                    info!("Camera access granted! You can now use the camera.");
                }
                PermissionStatus::Denied => {
                    warn!("Camera access denied. Some features may not work.");
                }
                _ => {
                    info!("Camera permission changed to: {}", event.new_status);
                }
            }
        }
    }
}