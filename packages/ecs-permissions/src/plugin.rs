//! Bevy plugin integration for permissions system

use std::collections::HashMap;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};

use bevy::prelude::*;

use crate::events::*;
use crate::manager::PermissionManager;
use crate::types::{PermissionError, PermissionStatus, PermissionType};

type PendingRequestReceiver = Arc<Mutex<Receiver<Result<PermissionStatus, PermissionError>>>>;

#[derive(Event)]
pub struct PermissionChanged {
    pub typ: PermissionType,
    pub status: PermissionStatus,
}

#[derive(Event)]
pub struct PermissionRequest {
    pub typ: PermissionType,
}

#[derive(Event)]
pub struct PermissionRequestError {
    pub typ: PermissionType,
    pub error: PermissionError,
}

#[derive(Resource)]
pub struct PermissionResource {
    pub manager: PermissionManager,
    pending_requests: HashMap<PermissionType, PendingRequestReceiver>,
}

impl Default for PermissionResource {
    fn default() -> Self {
        Self::new()
    }
}

impl PermissionResource {
    pub fn new() -> Self {
        Self {
            manager: PermissionManager::new(),
            pending_requests: HashMap::new(),
        }
    }
}

pub struct PermissionPlugin;

impl Plugin for PermissionPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PermissionResource::default())
            .add_event::<PermissionChanged>()
            .add_event::<PermissionRequest>()
            .add_event::<PermissionRequestError>()
            // Add new permission set request events
            .add_event::<PermissionSetRequest>()
            .add_event::<PermissionSetResponse>()
            .add_event::<PermissionWizardRequest>()
            .add_event::<PermissionWizardComplete>()
            .add_event::<PermissionBatchStatusUpdate>()
            .add_systems(Update, initiate_permission_requests)
            .add_systems(PostUpdate, poll_permission_results)
            .add_systems(PostUpdate, handle_permission_changes);
    }
}

fn initiate_permission_requests(
    mut events: EventReader<PermissionRequest>,
    mut res: ResMut<PermissionResource>,
) {
    for event in events.read() {
        // Only start request if not already pending
        if !res.pending_requests.contains_key(&event.typ) {
            let rx = res.manager.request_permission(event.typ);
            res.pending_requests
                .insert(event.typ, Arc::new(Mutex::new(rx)));
        }
    }
}

fn poll_permission_results(
    mut res: ResMut<PermissionResource>,
    mut changes: EventWriter<PermissionChanged>,
    mut errors: EventWriter<PermissionRequestError>,
) {
    let mut completed = Vec::new();

    for (&typ, rx_arc) in res.pending_requests.iter() {
        // Try to lock and check for results non-blocking
        if let Ok(rx) = rx_arc.try_lock()
            && let Ok(result) = rx.try_recv()
        {
            match result {
                Ok(status) => {
                    changes.write(PermissionChanged { typ, status });
                },
                Err(error) => {
                    errors.write(PermissionRequestError { typ, error });
                },
            }
            completed.push(typ);
        }
    }

    // Remove completed requests
    for typ in completed {
        res.pending_requests.remove(&typ);
    }
}

fn handle_permission_changes(
    mut events: EventReader<PermissionChanged>,
    res: Res<PermissionResource>,
) {
    for event in events.read() {
        res.manager.refresh_cache(event.typ);
        // Permission changed event handled - using debug logging in development builds only
        #[cfg(debug_assertions)]
        eprintln!("Permission changed for {:?}: {:?}", event.typ, event.status);
    }
}
