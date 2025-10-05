//! AV Foundation permissions for Camera and Microphone - Complete reference implementation

use std::sync::mpsc::Sender;

use block2::RcBlock;
use objc2::runtime::Bool;
use objc2_av_foundation::{
    AVAuthorizationStatus, AVCaptureDevice, AVMediaTypeAudio, AVMediaTypeVideo,
};

use crate::types::{PermissionError, PermissionStatus, PermissionType};

impl From<AVAuthorizationStatus> for PermissionStatus {
    fn from(status: AVAuthorizationStatus) -> Self {
        match status {
            AVAuthorizationStatus::Authorized => Self::Authorized,
            AVAuthorizationStatus::Denied => Self::Denied,
            AVAuthorizationStatus::Restricted => Self::Restricted,
            _ => Self::NotDetermined,
        }
    }
}

pub fn check_permission(typ: PermissionType) -> Result<PermissionStatus, PermissionError> {
    let media_type = match typ {
        PermissionType::Camera => unsafe { AVMediaTypeVideo.ok_or(PermissionError::Unknown)? },
        PermissionType::Microphone => unsafe { AVMediaTypeAudio.ok_or(PermissionError::Unknown)? },
        _ => return Err(PermissionError::Unknown),
    };
    let status = unsafe { AVCaptureDevice::authorizationStatusForMediaType(media_type) };
    Ok(status.into())
}

pub fn request_permission(
    typ: PermissionType,
    tx: Sender<Result<PermissionStatus, PermissionError>>,
) {
    let media_type = match typ {
        PermissionType::Camera => unsafe {
            match AVMediaTypeVideo {
                Some(media_type) => media_type,
                None => {
                    let _ = tx.send(Err(PermissionError::Unknown));
                    return;
                },
            }
        },
        PermissionType::Microphone => unsafe {
            match AVMediaTypeAudio {
                Some(media_type) => media_type,
                None => {
                    let _ = tx.send(Err(PermissionError::Unknown));
                    return;
                },
            }
        },
        _ => {
            let _ = tx.send(Err(PermissionError::Unknown));
            return;
        },
    };
    let handler = RcBlock::new(move |granted: Bool| {
        let status = if granted.as_bool() {
            PermissionStatus::Authorized
        } else {
            PermissionStatus::Denied
        };
        let _ = tx.send(Ok(status));
    });

    unsafe {
        AVCaptureDevice::requestAccessForMediaType_completionHandler(media_type, &handler);
    }
}
