//! Contacts framework permissions - Complete reference implementation

use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_contacts::{CNAuthorizationStatus, CNContactStore, CNEntityType};
use objc2_foundation::NSError;

use crate::types::{PermissionError, PermissionStatus};

pub fn check_permission() -> Result<PermissionStatus, PermissionError> {
    let status =
        unsafe { CNContactStore::authorizationStatusForEntityType(CNEntityType::Contacts) };
    let mapped = match status {
        CNAuthorizationStatus::Authorized => PermissionStatus::Authorized,
        CNAuthorizationStatus::Denied => PermissionStatus::Denied,
        CNAuthorizationStatus::Restricted => PermissionStatus::Restricted,
        _ => PermissionStatus::NotDetermined,
    };
    Ok(mapped)
}

pub fn request_permission(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    let store = unsafe { CNContactStore::new() };
    let handler = RcBlock::new(move |granted: Bool, error: *mut NSError| {
        if !error.is_null() {
            let _ = tx.send(Err(PermissionError::SystemError(
                "Error in request".to_string(),
            )));
            return;
        }
        let status = if granted.as_bool() {
            PermissionStatus::Authorized
        } else {
            PermissionStatus::Denied
        };
        let _ = tx.send(Ok(status));
    });

    unsafe {
        store.requestAccessForEntityType_completionHandler(CNEntityType::Contacts, &handler);
    }
}
