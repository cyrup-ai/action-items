//! EventKit permissions for Calendar and Reminders - Complete reference implementation

use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_event_kit::{EKAuthorizationStatus, EKEntityType, EKEventStore};
use objc2_foundation::NSError;

use crate::types::{PermissionError, PermissionStatus, PermissionType};

pub fn check_permission(typ: PermissionType) -> Result<PermissionStatus, PermissionError> {
    let entity_type = match typ {
        PermissionType::Calendar => EKEntityType::Event,
        PermissionType::Reminders => EKEntityType::Reminder,
        _ => return Err(PermissionError::Unknown),
    };
    let status = unsafe { EKEventStore::authorizationStatusForEntityType(entity_type) };
    let mapped = match status {
        EKAuthorizationStatus::FullAccess => PermissionStatus::Authorized,
        EKAuthorizationStatus::Denied => PermissionStatus::Denied,
        EKAuthorizationStatus::Restricted => PermissionStatus::Restricted,
        _ => PermissionStatus::NotDetermined,
    };
    Ok(mapped)
}

pub fn request_permission(
    typ: PermissionType,
    tx: Sender<Result<PermissionStatus, PermissionError>>,
) {
    let entity_type = match typ {
        PermissionType::Calendar => EKEntityType::Event,
        PermissionType::Reminders => EKEntityType::Reminder,
        _ => {
            let _ = tx.send(Err(PermissionError::Unknown));
            return;
        },
    };

    let store = unsafe { EKEventStore::new() };
    let tx_clone = tx.clone();
    let handler = RcBlock::new(move |granted: Bool, error: *mut NSError| {
        if !error.is_null() {
            let _ = tx_clone.send(Err(PermissionError::SystemError(
                "Error in request".to_string(),
            )));
            return;
        }
        let status = if granted.as_bool() {
            PermissionStatus::Authorized
        } else {
            PermissionStatus::Denied
        };
        let _ = tx_clone.send(Ok(status));
    });

    unsafe {
        match entity_type {
            EKEntityType::Event => {
                store.requestFullAccessToEventsWithCompletion(RcBlock::as_ptr(&handler));
            },
            EKEntityType::Reminder => {
                store.requestFullAccessToRemindersWithCompletion(RcBlock::as_ptr(&handler));
            },
            _ => {
                let _ = tx.send(Err(PermissionError::Unknown));
            },
        }
    }
}
