//! Core Location permissions - Complete reference implementation

use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex, OnceLock};

use objc2::rc::Retained;
use objc2::runtime::{NSObject, NSObjectProtocol, ProtocolObject};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send};
use objc2_core_location::{CLAuthorizationStatus, CLLocationManager, CLLocationManagerDelegate};

use crate::types::{PermissionError, PermissionStatus};

type LocationTxType = Arc<Mutex<Sender<Result<PermissionStatus, PermissionError>>>>;

static LOCATION_TX: OnceLock<LocationTxType> = OnceLock::new();

define_class!(
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct LocationDelegate;

    unsafe impl NSObjectProtocol for LocationDelegate {}

    unsafe impl CLLocationManagerDelegate for LocationDelegate {
        #[unsafe(method(locationManager:didChangeAuthorizationStatus:))]
        #[allow(non_snake_case)]
        fn locationManager_didChangeAuthorizationStatus(
            &self,
            _manager: &CLLocationManager,
            status: CLAuthorizationStatus,
        ) {
            if let Some(tx_arc) = LOCATION_TX.get()
                && let Ok(tx) = tx_arc.lock()
            {
                let _ = tx.send(Ok(status.into()));
            }
        }
    }
);

impl From<CLAuthorizationStatus> for PermissionStatus {
    fn from(status: CLAuthorizationStatus) -> Self {
        match status {
            CLAuthorizationStatus::AuthorizedAlways
            | CLAuthorizationStatus::AuthorizedWhenInUse => Self::Authorized,
            CLAuthorizationStatus::Denied => Self::Denied,
            CLAuthorizationStatus::Restricted => Self::Restricted,
            _ => Self::NotDetermined,
        }
    }
}

pub fn check_permission() -> Result<PermissionStatus, PermissionError> {
    if let Some(_mtm) = MainThreadMarker::new() {
        let manager = unsafe { CLLocationManager::new() };
        let status = unsafe { manager.authorizationStatus() };
        Ok(status.into())
    } else {
        Err(PermissionError::SystemError(
            "Not on main thread".to_string(),
        ))
    }
}

pub fn request_permission(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    if let Some(mtm) = MainThreadMarker::new() {
        let _ = LOCATION_TX.set(Arc::new(Mutex::new(tx.clone())));
        let delegate = LocationDelegate::alloc(mtm);
        let delegate: Retained<LocationDelegate> = unsafe { msg_send![delegate, init] };
        let manager = unsafe { CLLocationManager::new() };
        unsafe { manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate))) };

        let status = unsafe { manager.authorizationStatus() };
        if status == CLAuthorizationStatus::NotDetermined {
            unsafe { manager.requestAlwaysAuthorization() };
        } else {
            let _ = tx.send(Ok(status.into()));
        }
    }
}
