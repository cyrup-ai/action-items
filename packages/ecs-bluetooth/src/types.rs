//! Core types for Bluetooth operations

use std::collections::HashMap;
use std::time::Duration;

use bevy::prelude::Event;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Unique identifier for a Bluetooth device
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BluetoothDeviceId(pub String);

impl BluetoothDeviceId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<String> for BluetoothDeviceId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

impl From<&str> for BluetoothDeviceId {
    fn from(id: &str) -> Self {
        Self(id.to_string())
    }
}

/// Connection state of a Bluetooth device
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Disconnected,
    Connecting,
    Connected,
    Disconnecting,
}

/// Information about a discovered Bluetooth device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: BluetoothDeviceId,
    pub name: Option<String>,
    pub address: String,
    pub rssi: Option<i16>,
    pub services: Vec<Uuid>,
    pub manufacturer_data: HashMap<u16, Vec<u8>>,
    pub service_data: HashMap<Uuid, Vec<u8>>,
    pub tx_power: Option<i8>,
    pub connectable: bool,
}

/// Bluetooth device with connection state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BluetoothDevice {
    pub info: DeviceInfo,
    pub state: ConnectionState,
    pub last_seen: std::time::SystemTime,
}

impl BluetoothDevice {
    pub fn new(info: DeviceInfo) -> Self {
        Self {
            info,
            state: ConnectionState::Disconnected,
            last_seen: std::time::SystemTime::now(),
        }
    }

    pub fn id(&self) -> &BluetoothDeviceId {
        &self.info.id
    }

    pub fn name(&self) -> Option<&str> {
        self.info.name.as_deref()
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.state, ConnectionState::Connected)
    }

    pub fn update_rssi(&mut self, rssi: i16) {
        self.info.rssi = Some(rssi);
        self.last_seen = std::time::SystemTime::now();
    }
}

/// Options for scanning for Bluetooth devices
#[derive(Debug, Clone)]
pub struct ScanOptions {
    /// Duration to scan for devices
    pub duration: Option<Duration>,
    /// Filter by service UUIDs
    pub service_uuids: Vec<Uuid>,
    /// Allow duplicate discoveries
    pub allow_duplicates: bool,
    /// Minimum RSSI threshold
    pub min_rssi: Option<i16>,
}

impl Default for ScanOptions {
    fn default() -> Self {
        Self {
            duration: Some(Duration::from_secs(10)),
            service_uuids: Vec::new(),
            allow_duplicates: false,
            min_rssi: None,
        }
    }
}

/// Events emitted by the Bluetooth manager - following ARCHITECTURE.md event pattern
#[derive(Event, Debug, Clone)]
pub enum BluetoothEvent {
    /// A device was discovered during scanning
    DeviceDiscovered(BluetoothDevice),
    /// A device was updated (e.g., RSSI changed)
    DeviceUpdated(BluetoothDevice),
    /// A device connection state changed
    ConnectionStateChanged {
        device_id: BluetoothDeviceId,
        state: ConnectionState,
    },
    /// Scanning started
    ScanStarted,
    /// Scanning stopped
    ScanStopped,
    /// Adapter state changed
    AdapterStateChanged(AdapterState),
}

/// State of the Bluetooth adapter
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AdapterState {
    #[default]
    Unknown,
    PoweredOff,
    PoweredOn,
    Unauthorized,
    Unsupported,
}
