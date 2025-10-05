//! Windows system-level permissions (Screen capture, Input monitoring, Admin)

use std::sync::mpsc::Sender;

#[cfg(target_os = "windows")]
use {
    windows::Graphics::Capture::GraphicsCaptureSession,
    windows::Win32::Foundation::BOOL,
    windows::Win32::Security::{
        CheckTokenMembership, CreateWellKnownSid, WinBuiltinAdministratorsSid,
    },
    windows::Win32::UI::Input::{RAWINPUTDEVICE, RIDEV_INPUTSINK, RegisterRawInputDevices},
    windows::Win32::UI::WindowsAndMessaging::GetDesktopWindow,
    windows::core::Result as WinResult,
};

use crate::types::{PermissionError, PermissionStatus, PermissionType};

pub fn check_screen_capture() -> Result<PermissionStatus, PermissionError> {
    #[cfg(target_os = "windows")]
    {
        // Check screen capture capability using Windows APIs
        match GraphicsCaptureSession::IsSupported() {
            Ok(is_supported) => {
                let status = if is_supported {
                    PermissionStatus::Authorized
                } else {
                    PermissionStatus::Denied
                };
                Ok(status)
            },
            Err(_) => Ok(PermissionStatus::NotDetermined),
        }
    }
    #[cfg(not(target_os = "windows"))]
    Ok(PermissionStatus::Authorized)
}

pub fn check_input_monitoring() -> Result<PermissionStatus, PermissionError> {
    #[cfg(target_os = "windows")]
    {
        // Windows desktop applications have broad input access by default
        // Check if we can register for raw input to verify capability
        unsafe {
            let mut rid = RAWINPUTDEVICE {
                usUsagePage: 0x01, // Generic Desktop
                usUsage: 0x06,     // Keyboard
                dwFlags: RIDEV_INPUTSINK,
                hwndTarget: GetDesktopWindow(),
            };

            // Test if we can register for input monitoring
            let status = match RegisterRawInputDevices(
                &mut [rid],
                std::mem::size_of::<RAWINPUTDEVICE>() as u32,
            ) {
                Ok(_) => PermissionStatus::Authorized,
                Err(_) => PermissionStatus::Denied,
            };
            Ok(status)
        }
    }
    #[cfg(not(target_os = "windows"))]
    Ok(PermissionStatus::Authorized)
}

pub fn check_admin_access() -> Result<PermissionStatus, PermissionError> {
    #[cfg(target_os = "windows")]
    {
        // Check if running as administrator using Windows APIs
        unsafe {
            let mut admin_group: [u8; 256] = [0; 256];
            let mut admin_group_size = admin_group.len() as u32;
            if CreateWellKnownSid(
                WinBuiltinAdministratorsSid,
                None,
                Some(&mut admin_group),
                &mut admin_group_size,
            )
            .is_ok()
            {
                let mut is_member = BOOL(0);
                if CheckTokenMembership(None, admin_group.as_ptr() as *mut _, &mut is_member)
                    .is_ok()
                {
                    let status = if is_member.as_bool() {
                        PermissionStatus::Authorized
                    } else {
                        PermissionStatus::Denied
                    };
                    Ok(status)
                } else {
                    Ok(PermissionStatus::Denied)
                }
            } else {
                Ok(PermissionStatus::Denied)
            }
        }
    }
    #[cfg(not(target_os = "windows"))]
    Ok(PermissionStatus::Authorized)
}

pub fn request_screen_capture(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            let result = match GraphicsCaptureSession::IsSupported() {
                Ok(is_supported) => {
                    if is_supported {
                        Ok(PermissionStatus::Authorized)
                    } else {
                        Ok(PermissionStatus::Denied)
                    }
                },
                Err(e) => Err(PermissionError::SystemError(format!(
                    "Screen capture check failed: {}",
                    e
                ))),
            };
            let _ = tx.send(result);
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = tx.send(Ok(PermissionStatus::Authorized));
    }
}

pub fn request_input_monitoring(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    #[cfg(target_os = "windows")]
    {
        std::thread::spawn(move || {
            let result = unsafe {
                let mut rid = RAWINPUTDEVICE {
                    usUsagePage: 0x01, // Generic Desktop
                    usUsage: 0x06,     // Keyboard
                    dwFlags: RIDEV_INPUTSINK,
                    hwndTarget: GetDesktopWindow(),
                };

                // Test if we can register for input monitoring
                match RegisterRawInputDevices(
                    &mut [rid],
                    std::mem::size_of::<RAWINPUTDEVICE>() as u32,
                ) {
                    Ok(_) => Ok(PermissionStatus::Authorized),
                    Err(e) => Err(PermissionError::SystemError(format!(
                        "Input monitoring registration failed: {}",
                        e
                    ))),
                }
            };
            let _ = tx.send(result);
        });
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = tx.send(Ok(PermissionStatus::Authorized));
    }
}

pub fn request_admin_access(tx: Sender<Result<PermissionStatus, PermissionError>>) {
    #[cfg(target_os = "windows")]
    {
        // Use same administrator checking logic as check_permission
        let result = unsafe {
            let mut admin_group: [u8; 256] = [0; 256];
            let mut admin_group_size = admin_group.len() as u32;
            if CreateWellKnownSid(
                WinBuiltinAdministratorsSid,
                None,
                Some(&mut admin_group),
                &mut admin_group_size,
            )
            .is_ok()
            {
                let mut is_member = BOOL(0);
                if CheckTokenMembership(None, admin_group.as_ptr() as *mut _, &mut is_member)
                    .is_ok()
                {
                    if is_member.as_bool() {
                        Ok(PermissionStatus::Authorized)
                    } else {
                        // Could potentially elevate or request UAC here, but for now just
                        // return current status
                        Ok(PermissionStatus::Denied)
                    }
                } else {
                    Ok(PermissionStatus::Denied)
                }
            } else {
                Ok(PermissionStatus::Denied)
            }
        };
        let _ = tx.send(result);
    }
    #[cfg(not(target_os = "windows"))]
    {
        let _ = tx.send(Ok(PermissionStatus::Authorized));
    }
}
