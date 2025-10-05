//! Permission Set Request Examples
//!
//! Demonstrates how to use the enhanced ecs-permissions API to request
//! permission sets and automatically trigger the wizard UI when needed.

use bevy::prelude::*;
use action_items_ecs_permissions::{
    add_permissions_with_wizard, PermissionSetRequest, PermissionSetResponse,
    PermissionType, RequestPriority,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(add_permissions_with_wizard)
        .add_systems(Startup, setup_example)
        .add_systems(Update, (
            handle_user_input,
            handle_permission_responses,
        ))
        .run();
}

fn setup_example(mut commands: Commands) {
    commands.spawn(Camera2d);
    
    info!("Permission Set Request Example");
    info!("Press keys to request different permission sets:");
    info!("  C - Camera service permissions");
    info!("  A - Audio service permissions");
    info!("  S - System service permissions");
    info!("  F - Full application permissions");
    info!("  ESC - Exit");
}

fn handle_user_input(
    keys: Res<ButtonInput<KeyCode>>,
    mut permission_requests: EventWriter<PermissionSetRequest>,
) {
    if keys.just_pressed(KeyCode::KeyC) {
        // Request camera service permissions
        let request = PermissionSetRequest::new("camera_service")
            .with_required(PermissionType::Camera)
            .with_optional(PermissionType::Microphone)
            .with_reason("Camera service needs camera access for video calls and optional microphone for audio")
            .with_priority(RequestPriority::High);
        
        permission_requests.write(request);
        info!("Requested camera service permissions");
    }
    
    if keys.just_pressed(KeyCode::KeyA) {
        // Request audio service permissions
        let request = PermissionSetRequest::new("audio_service")
            .with_required_permissions([PermissionType::Microphone])
            .with_optional_permissions([PermissionType::Camera])
            .with_reason("Audio service needs microphone access for voice recording")
            .with_priority(RequestPriority::Normal);
        
        permission_requests.write(request);
        info!("Requested audio service permissions");
    }
    
    if keys.just_pressed(KeyCode::KeyS) {
        // Request system service permissions
        let request = PermissionSetRequest::new("system_service")
            .with_required_permissions([
                PermissionType::Accessibility,
                PermissionType::InputMonitoring,
            ])
            .with_optional_permissions([
                PermissionType::ScreenCapture,
                PermissionType::FullDiskAccess,
            ])
            .with_reason("System service needs accessibility and input monitoring for global hotkeys")
            .with_priority(RequestPriority::Critical);
        
        permission_requests.write(request);
        info!("Requested system service permissions");
    }
    
    if keys.just_pressed(KeyCode::KeyF) {
        // Request full application permissions
        let request = PermissionSetRequest::new("full_application")
            .with_required_permissions([
                PermissionType::Accessibility,
                PermissionType::InputMonitoring,
                PermissionType::ScreenCapture,
            ])
            .with_optional_permissions([
                PermissionType::Camera,
                PermissionType::Microphone,
                PermissionType::FullDiskAccess,
                PermissionType::WiFi,
            ])
            .with_reason("Full application setup requires core permissions for all features")
            .with_priority(RequestPriority::Critical)
            .with_force_recheck(true);
        
        permission_requests.write(request);
        info!("Requested full application permissions");
    }
    
    if keys.just_pressed(KeyCode::Escape) {
        std::process::exit(0);
    }
}

fn handle_permission_responses(
    mut responses: EventReader<PermissionSetResponse>,
) {
    for response in responses.read() {
        info!("Permission set response for request '{}': success={}", 
              response.request_id, response.success);
        
        if !response.granted_permissions.is_empty() {
            info!("  Granted permissions: {:?}", response.granted_permissions);
        }
        
        if !response.denied_permissions.is_empty() {
            warn!("  Denied permissions: {:?}", response.denied_permissions);
        }
        
        if !response.pending_permissions.is_empty() {
            info!("  Pending permissions: {:?}", response.pending_permissions);
        }
        
        if response.wizard_shown {
            info!("  Wizard was shown to guide user through permission setup");
        }
        
        if let Some(error) = &response.error_message {
            error!("  Error: {}", error);
        }
    }
}

/// Advanced example showing batch permission requests
#[allow(dead_code)]
fn advanced_batch_example(
    mut permission_requests: EventWriter<PermissionSetRequest>,
) {
    // Example: Media editing application
    let media_request = PermissionSetRequest::new("media_editor")
        .with_required_permissions([
            PermissionType::Camera,
            PermissionType::Microphone,
            PermissionType::ScreenCapture,
        ])
        .with_optional_permissions([
            PermissionType::FullDiskAccess,
            PermissionType::Photos,
        ])
        .with_reason("Media editor needs camera, microphone, and screen capture for content creation")
        .with_priority(RequestPriority::High)
        .with_wizard_fallback(true);
    
    permission_requests.write(media_request);
    
    // Example: System utility application
    let system_request = PermissionSetRequest::new("system_utility")
        .with_required_permissions([
            PermissionType::Accessibility,
            PermissionType::InputMonitoring,
        ])
        .with_optional_permissions([
            PermissionType::FullDiskAccess,
            PermissionType::AdminFiles,
        ])
        .with_reason("System utility needs accessibility and input monitoring for automation")
        .with_priority(RequestPriority::Critical)
        .with_wizard_fallback(true)
        .with_force_recheck(false);
    
    permission_requests.write(system_request);
    
    // Example: Background service (no wizard)
    let background_request = PermissionSetRequest::new("background_service")
        .with_required_permissions([PermissionType::WiFi])
        .with_reason("Background service needs network access for sync")
        .with_priority(RequestPriority::Low)
        .with_wizard_fallback(false); // Don't show wizard for background services
    
    permission_requests.write(background_request);
}