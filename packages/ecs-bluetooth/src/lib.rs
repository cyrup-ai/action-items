#![recursion_limit = "256"]
//! Cross-platform Bluetooth operations for ECS applications
//!
//! This crate provides a unified interface for Bluetooth Low Energy (BLE) operations
//! across macOS, Windows, and Linux platforms. It integrates with Bevy ECS for
//! reactive Bluetooth device management.

pub mod error;
pub mod manager;
pub mod types;

#[cfg(target_os = "macos")]
pub mod platforms {
    pub mod macos;
}

#[cfg(target_os = "windows")]
pub mod platforms {
    pub mod windows;
}

#[cfg(target_os = "linux")]
pub mod platforms {
    pub mod linux;
}

// Re-export main types
// Bevy plugin
use bevy::prelude::*;
pub use error::{BluetoothError, BluetoothResult};
pub use manager::{BluetoothEventBridge, BluetoothManager, bluetooth_event_bridge_system};
pub use types::{
    BluetoothDevice, BluetoothDeviceId, BluetoothEvent, ConnectionState, DeviceInfo, ScanOptions,
};

/// Bevy plugin for Bluetooth functionality - following ARCHITECTURE.md event-driven pattern
#[derive(Default)]
pub struct BluetoothPlugin;

impl Plugin for BluetoothPlugin {
    fn build(&self, app: &mut App) {
        // Create manager and bridge using the new event-driven pattern
        let (manager, bridge) = BluetoothManager::new_with_bridge();

        app
            // Add resources
            .insert_resource(manager)
            .insert_resource(bridge)
            // Register BluetoothEvent as a Bevy Event - following ARCHITECTURE.md
            .add_event::<BluetoothEvent>()
            // Add the event bridge system - following clipboard plugin pattern
            .add_systems(Update, bluetooth_event_bridge_system);
    }
}
