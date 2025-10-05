//! Permission manager with caching and async support

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, channel};
use std::sync::{Arc, RwLock};

use bevy::tasks::AsyncComputeTaskPool;

use crate::types::{PermissionError, PermissionStatus, PermissionType};

#[cfg(target_os = "macos")]
use crate::platforms::macos::handler::MacOSHandler;

pub struct PermissionManager {
    cache: Arc<RwLock<HashMap<PermissionType, PermissionStatus>>>,
}

impl PermissionManager {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn check_permission(
        &self,
        typ: PermissionType,
    ) -> Result<PermissionStatus, PermissionError> {
        // Try to read from cache, gracefully handle poisoned lock
        if let Ok(cache) = self.cache.read()
            && let Some(status) = cache.get(&typ)
        {
            return Ok(*status);
        }
        // If lock is poisoned, skip cache and proceed with fresh check

        let status = match typ {
            #[cfg(target_os = "macos")]
            PermissionType::Camera | PermissionType::Microphone => {
                crate::platforms::macos::av_permissions::check_permission(typ)
            },
            #[cfg(target_os = "macos")]
            PermissionType::Location => {
                crate::platforms::macos::location_permissions::check_permission()
            },
            #[cfg(target_os = "macos")]
            PermissionType::Calendar | PermissionType::Reminders => {
                crate::platforms::macos::event_kit_permissions::check_permission(typ)
            },
            #[cfg(target_os = "macos")]
            PermissionType::Contacts => {
                crate::platforms::macos::contacts_permissions::check_permission()
            },
            #[cfg(target_os = "macos")]
            PermissionType::Bluetooth => {
                crate::platforms::macos::bluetooth_permissions::check_permission()
            },
            #[cfg(target_os = "macos")]
            PermissionType::Accessibility | PermissionType::AccessibilityMouse => {
                MacOSHandler::new().check_accessibility()
            },
            #[cfg(target_os = "macos")]
            PermissionType::WiFi => {
                MacOSHandler::new().check_wifi()
            },
            #[cfg(target_os = "macos")]
            PermissionType::ScreenCapture => {
                MacOSHandler::new().check_screen_recording()
            },
            #[cfg(target_os = "macos")]
            PermissionType::InputMonitoring => {
                MacOSHandler::new().check_input_monitoring()
            },
            #[cfg(target_os = "macos")]
            _ => crate::platforms::macos::tcc_permissions::check_permission(typ),

            #[cfg(target_os = "windows")]
            _ => crate::platforms::windows::check_permission(typ),

            #[cfg(target_os = "linux")]
            _ => crate::platforms::linux::check_permission(typ),

            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            _ => Err(PermissionError::Unknown),
        };

        if let Ok(s) = &status {
            // Try to update cache, silently ignore if lock is poisoned
            if let Ok(mut cache) = self.cache.write() {
                cache.insert(typ, *s);
            }
            // If cache update fails, the permission operation still succeeds
        }
        status
    }

    pub fn request_permission(
        &self,
        typ: PermissionType,
    ) -> Receiver<Result<PermissionStatus, PermissionError>> {
        let (tx, rx) = channel();
        let cache = self.cache.clone();
        let task_pool = AsyncComputeTaskPool::get();

        task_pool
            .spawn(async move {
                let result = match typ {
                    #[cfg(target_os = "macos")]
                    PermissionType::Camera | PermissionType::Microphone => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::macos::av_permissions::request_permission(typ, inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::Location => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::macos::location_permissions::request_permission(inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::Calendar | PermissionType::Reminders => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::macos::event_kit_permissions::request_permission(
                            typ, inner_tx,
                        );
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::Contacts => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::macos::contacts_permissions::request_permission(inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::Bluetooth => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::macos::bluetooth_permissions::request_permission(
                            inner_tx,
                        );
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::Accessibility | PermissionType::AccessibilityMouse => {
                        let (inner_tx, inner_rx) = channel();
                        MacOSHandler::new().request_accessibility(inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::WiFi => {
                        // WiFi doesn't have a specific request method, just return current status
                        MacOSHandler::new().check_wifi()
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::ScreenCapture => {
                        let (inner_tx, inner_rx) = channel();
                        MacOSHandler::new().request_screen_recording(inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    PermissionType::InputMonitoring => {
                        let (inner_tx, inner_rx) = channel();
                        MacOSHandler::new().request_input_monitoring(inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },
                    #[cfg(target_os = "macos")]
                    _ => crate::platforms::macos::tcc_permissions::request_permission(typ),

                    #[cfg(target_os = "windows")]
                    _ => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::windows::request_permission(typ, inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },

                    #[cfg(target_os = "linux")]
                    _ => {
                        let (inner_tx, inner_rx) = channel();
                        crate::platforms::linux::request_permission(typ, inner_tx);
                        inner_rx.recv().unwrap_or(Err(PermissionError::Unknown))
                    },

                    #[cfg(not(any(
                        target_os = "macos",
                        target_os = "windows",
                        target_os = "linux"
                    )))]
                    _ => Err(PermissionError::Unknown),
                };

                if let Ok(s) = &result {
                    // Try to update cache, silently ignore if lock is poisoned
                    if let Ok(mut cache_guard) = cache.write() {
                        cache_guard.insert(typ, *s);
                    }
                    // If cache update fails, permission result is still sent successfully
                }
                let _ = tx.send(result);
            })
            .detach();

        rx
    }

    pub fn refresh_cache(&self, typ: PermissionType) {
        let _ = self.check_permission(typ);
    }
}

impl Default for PermissionManager {
    fn default() -> Self {
        Self::new()
    }
}
