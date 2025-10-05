//! Basic permission usage example
//! 
//! Demonstrates how to check and request system permissions using the Action Items permission system.

use action_items_ecs_permissions::{PermissionType, PermissionStatus, add_permissions};
use bevy::prelude::*;
use tokio::sync::oneshot;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(action_items_ecs_permissions::PermissionPlugin)
        .add_systems(Startup, setup_permission_demo)
        .add_systems(Update, handle_permission_responses)
        .run();
}

#[derive(Resource)]
struct PermissionDemo {
    camera_requested: bool,
    microphone_requested: bool,
}

fn setup_permission_demo(mut commands: Commands) {
    commands.insert_resource(PermissionDemo {
        camera_requested: false,
        microphone_requested: false,
    });
    
    info!("Permission demo started. Press C for camera, M for microphone permissions.");
}

fn handle_permission_responses(
    input: Res<ButtonInput<KeyCode>>,
    mut demo: ResMut<PermissionDemo>,
    permission_res: Res<action_items_ecs_permissions::PermissionResource>,
    mut permission_events: EventWriter<action_items_ecs_permissions::PermissionRequest>,
) {
    // Check camera permission on 'C' key
    if input.just_pressed(KeyCode::KeyC) && !demo.camera_requested {
        demo.camera_requested = true;
        
        match permission_res.check_permission(PermissionType::Camera) {
            Ok(status) => {
                info!("Camera permission status: {}", status);
                
                if status == PermissionStatus::NotDetermined {
                    info!("Requesting camera permission...");
                    let (sender, _receiver) = oneshot::channel();
                    permission_events.send(action_items_ecs_permissions::PermissionRequest {
                        permission_type: PermissionType::Camera,
                        response_sender: sender,
                    });
                }
            }
            Err(e) => error!("Failed to check camera permission: {}", e),
        }
    }
    
    // Check microphone permission on 'M' key
    if input.just_pressed(KeyCode::KeyM) && !demo.microphone_requested {
        demo.microphone_requested = true;
        
        match permission_res.check_permission(PermissionType::Microphone) {
            Ok(status) => {
                info!("Microphone permission status: {}", status);
                
                if status == PermissionStatus::NotDetermined {
                    info!("Requesting microphone permission...");
                    let (sender, _receiver) = oneshot::channel();
                    permission_events.send(action_items_ecs_permissions::PermissionRequest {
                        permission_type: PermissionType::Microphone,
                        response_sender: sender,
                    });
                }
            }
            Err(e) => error!("Failed to check microphone permission: {}", e),
        }
    }
    
    // List supported permissions on 'L' key
    if input.just_pressed(KeyCode::KeyL) {
        let supported = permission_res.supported_permissions();
        info!("Supported permissions on this platform:");
        for permission in supported {
            info!("  - {}", permission);
        }
    }
}