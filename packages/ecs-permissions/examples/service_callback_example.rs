//! Service Callback Example
//! 
//! Demonstrates how services can request permissions through the wizard
//! and receive callbacks when the wizard completes.

use bevy::prelude::*;
use std::collections::HashSet;
use action_items_ecs_permissions::{
    PermissionSetRequest, PermissionSetResponse, PermissionType,
    PermissionPlugin,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(PermissionPlugin)
        .add_systems(Startup, setup_example_service)
        .add_systems(Update, (
            example_service_system,
            handle_permission_responses,
        ))
        .run();
}

/// Example service component
#[derive(Component)]
struct ExampleService {
    state: ServiceState,
    request_id: Option<String>,
}

/// Service workflow states
#[derive(Debug, Clone, PartialEq)]
enum ServiceState {
    WaitingToStart,
    RequestingPermissions,
    WaitingForPermissions,
    PermissionsGranted,
    PermissionsDenied,
    WorkflowComplete,
    Error(String),
}

/// Setup the example service
fn setup_example_service(mut commands: Commands) {
    commands.spawn(ExampleService {
        state: ServiceState::WaitingToStart,
        request_id: None,
    });
    
    info!("Example service started - will request permissions in 3 seconds");
}

/// Example service system demonstrating permission request workflow
fn example_service_system(
    mut query: Query<&mut ExampleService>,
    mut permission_requests: EventWriter<PermissionSetRequest>,
    time: Res<Time>,
) {
    for mut service in query.iter_mut() {
        match service.state {
            ServiceState::WaitingToStart => {
                // Wait 3 seconds before starting
                if time.elapsed_secs() > 3.0 {
                    info!("Service starting permission request workflow");
                    service.state = ServiceState::RequestingPermissions;
                }
            }
            ServiceState::RequestingPermissions => {
                // Request required permissions for our service
                let mut required_permissions = HashSet::new();
                required_permissions.insert(PermissionType::Accessibility);
                required_permissions.insert(PermissionType::ScreenCapture);
                
                let mut optional_permissions = HashSet::new();
                optional_permissions.insert(PermissionType::Camera);
                optional_permissions.insert(PermissionType::Microphone);
                
                let request = PermissionSetRequest::new("example_service")
                    .with_required_permissions(required_permissions)
                    .with_optional_permissions(optional_permissions)
                    .with_reason("Example service needs these permissions to demonstrate the workflow")
                    .with_wizard_fallback(true);
                
                let request_id = request.request_id.clone();
                permission_requests.write(request);
                service.request_id = Some(request_id.clone());
                service.state = ServiceState::WaitingForPermissions;
                
                info!("Sent permission request: {}", request_id);
            }
            ServiceState::WaitingForPermissions => {
                // Waiting for callback - handled in separate system
            }
            ServiceState::PermissionsGranted => {
                info!("Service received all required permissions - continuing workflow");
                service.state = ServiceState::WorkflowComplete;
            }
            ServiceState::PermissionsDenied => {
                error!("Service did not receive required permissions - cannot continue");
                service.state = ServiceState::Error("Required permissions denied".to_string());
            }
            ServiceState::WorkflowComplete => {
                info!("Service workflow completed successfully");
                // Service can now perform its intended functionality
            }
            ServiceState::Error(ref error) => {
                error!("Service error: {}", error);
                // Handle error state - could retry, notify user, etc.
            }
        }
    }
}

/// Handle permission responses from the wizard
fn handle_permission_responses(
    mut query: Query<&mut ExampleService>,
    mut permission_responses: EventReader<PermissionSetResponse>,
) {
    for response in permission_responses.read() {
        // Find the service that made this request
        for mut service in query.iter_mut() {
            if let Some(ref request_id) = service.request_id {
                if request_id == &response.request_id {
                    info!("Received permission response for request: {}", request_id);
                    
                    // Handle different response scenarios
                    if response.success {
                        info!("All required permissions granted!");
                        info!("Granted permissions: {:?}", response.granted_permissions);
                        if !response.denied_permissions.is_empty() {
                            info!("Some optional permissions denied: {:?}", response.denied_permissions);
                        }
                        service.state = ServiceState::PermissionsGranted;
                    } else {
                        // Check if this is a timeout or denial
                        if let Some(ref error_message) = response.error_message {
                            error!("Permission request failed: {}", error_message);
                            service.state = ServiceState::Error(error_message.clone());
                        } else {
                            error!("Required permissions not granted");
                            error!("Granted: {:?}", response.granted_permissions);
                            error!("Denied: {:?}", response.denied_permissions);
                            error!("Pending: {:?}", response.pending_permissions);
                            service.state = ServiceState::PermissionsDenied;
                        }
                    }
                    
                    // Log wizard interaction
                    if response.wizard_shown {
                        info!("Wizard was shown to the user");
                    } else {
                        info!("Permissions resolved without showing wizard");
                    }
                    
                    break;
                }
            }
        }
    }
}
