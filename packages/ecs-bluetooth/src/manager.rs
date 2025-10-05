//! Cross-platform Bluetooth manager following Bevy ECS patterns

use std::collections::HashMap;
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use tracing::{debug, error, info};

use crate::error::{BluetoothError, BluetoothResult};
use crate::types::{
    AdapterState, BluetoothDevice, BluetoothDeviceId, BluetoothEvent, ConnectionState, ScanOptions,
};

/// Event bridge resource for connecting platform events to Bevy events - following clipboard plugin
/// pattern
#[derive(Resource)]
pub struct BluetoothEventBridge {
    receiver: Arc<Mutex<Receiver<BluetoothEvent>>>,
}

/// Cross-platform Bluetooth manager - following ARCHITECTURE.md sync API pattern
#[derive(Resource)]
pub struct BluetoothManager {
    inner: Arc<Mutex<BluetoothManagerInner>>,
    /// Whether Bluetooth is available on this platform
    pub available: bool,
    /// Event sender for platform events
    _event_sender: Option<Sender<BluetoothEvent>>,
}

struct BluetoothManagerInner {
    devices: HashMap<BluetoothDeviceId, BluetoothDevice>,
    adapter_state: AdapterState,
    is_scanning: bool,
}

impl Default for BluetoothManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BluetoothManager {
    /// Create a new Bluetooth manager with event bridge - following clipboard plugin pattern
    pub fn new_with_bridge() -> (Self, BluetoothEventBridge) {
        // Create event channel for platform communication
        let (sender, receiver) = std::sync::mpsc::channel();

        // Test platform availability during initialization
        let available = Self::test_platform_availability();

        if available {
            // Initialize platform with event sender - following ARCHITECTURE.md pattern
            let init_result = {
                #[cfg(target_os = "macos")]
                {
                    crate::platforms::macos::initialize_bluetooth(sender.clone())
                }

                #[cfg(target_os = "windows")]
                {
                    crate::platforms::windows::initialize_bluetooth(sender.clone())
                }

                #[cfg(target_os = "linux")]
                {
                    crate::platforms::linux::initialize_bluetooth(sender.clone())
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
                {
                    Ok(()) // No-op for unsupported platforms
                }
            };

            match init_result {
                Ok(()) => info!("Bluetooth manager initialized successfully with event bridge"),
                Err(e) => error!("Failed to initialize platform Bluetooth: {:?}", e),
            }
        } else {
            error!("Bluetooth not available on this platform");
        }

        let manager = Self {
            inner: Arc::new(Mutex::new(BluetoothManagerInner {
                devices: HashMap::new(),
                adapter_state: AdapterState::Unknown,
                is_scanning: false,
            })),
            available,
            _event_sender: Some(sender),
        };

        let bridge = BluetoothEventBridge {
            receiver: Arc::new(Mutex::new(receiver)),
        };

        (manager, bridge)
    }

    /// Create a new Bluetooth manager - following ClipboardResource pattern
    pub fn new() -> Self {
        // Test platform availability during initialization
        let available = Self::test_platform_availability();

        if available {
            info!("Bluetooth manager initialized successfully");
        } else {
            error!("Bluetooth not available on this platform");
        }

        Self {
            inner: Arc::new(Mutex::new(BluetoothManagerInner {
                devices: HashMap::new(),
                adapter_state: AdapterState::Unknown,
                is_scanning: false,
            })),
            available,
            _event_sender: None,
        }
    }

    /// Test if Bluetooth is available on this platform
    fn test_platform_availability() -> bool {
        // Test platform-specific Bluetooth availability
        #[cfg(target_os = "macos")]
        {
            crate::platforms::macos::test_availability()
        }

        #[cfg(target_os = "windows")]
        {
            crate::platforms::windows::test_availability()
        }

        #[cfg(target_os = "linux")]
        {
            crate::platforms::linux::test_availability()
        }

        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            false // Unsupported platform
        }
    }

    /// Get all discovered devices - sync API following ARCHITECTURE.md
    pub fn devices(&self) -> BluetoothResult<Vec<BluetoothDevice>> {
        if !self.available {
            return Err(BluetoothError::NotSupported);
        }

        let inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;
        Ok(inner.devices.values().cloned().collect())
    }

    /// Get a specific device by ID - sync API following ARCHITECTURE.md
    pub fn device(&self, id: &BluetoothDeviceId) -> BluetoothResult<Option<BluetoothDevice>> {
        if !self.available {
            return Err(BluetoothError::NotSupported);
        }

        let inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;
        Ok(inner.devices.get(id).cloned())
    }

    /// Get current adapter state - sync API following ARCHITECTURE.md
    pub fn adapter_state(&self) -> BluetoothResult<AdapterState> {
        if !self.available {
            return Err(BluetoothError::NotSupported);
        }

        let inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;
        Ok(inner.adapter_state)
    }

    /// Check if currently scanning - sync API following ARCHITECTURE.md
    pub fn is_scanning(&self) -> BluetoothResult<bool> {
        if !self.available {
            return Ok(false);
        }

        let inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;
        Ok(inner.is_scanning)
    }

    /// Start scanning for devices - sync API following ARCHITECTURE.md
    pub fn start_scan(&self, options: ScanOptions) -> BluetoothResult<()> {
        if !self.available {
            return Err(BluetoothError::NotSupported);
        }

        debug!("Starting Bluetooth scan with options: {:?}", options);

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;

        if inner.is_scanning {
            debug!("Already scanning, ignoring start_scan request");
            return Ok(());
        }

        // Use tokio::task::block_in_place for platform operations - following ClipboardResource
        // pattern
        let result = tokio::task::block_in_place(|| {
            #[cfg(target_os = "macos")]
            {
                crate::platforms::macos::start_scan(options)
            }

            #[cfg(target_os = "windows")]
            {
                tokio::runtime::Handle::current()
                    .block_on(crate::platforms::windows::start_scan(options))
            }

            #[cfg(target_os = "linux")]
            {
                tokio::runtime::Handle::current()
                    .block_on(crate::platforms::linux::start_scan(options))
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            {
                error!("Bluetooth scanning not supported on this platform");
                Err(BluetoothError::NotSupported)
            }
        });

        match result {
            Ok(()) => {
                inner.is_scanning = true;
                info!("Bluetooth scan started successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to start Bluetooth scan: {:?}", e);
                Err(e)
            },
        }
    }

    /// Stop scanning for devices - sync API following ARCHITECTURE.md
    pub fn stop_scan(&self) -> BluetoothResult<()> {
        if !self.available {
            return Ok(()); // No-op if not available
        }

        debug!("Stopping Bluetooth scan");

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;

        if !inner.is_scanning {
            debug!("Not currently scanning, ignoring stop_scan request");
            return Ok(());
        }

        // Use tokio::task::block_in_place for platform operations - following ClipboardResource
        // pattern
        let result = tokio::task::block_in_place(|| {
            #[cfg(target_os = "macos")]
            {
                crate::platforms::macos::stop_scan()
            }

            #[cfg(target_os = "windows")]
            {
                tokio::runtime::Handle::current().block_on(crate::platforms::windows::stop_scan())
            }

            #[cfg(target_os = "linux")]
            {
                tokio::runtime::Handle::current().block_on(crate::platforms::linux::stop_scan())
            }

            #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
            {
                Ok(()) // No-op for unsupported platforms
            }
        });

        match result {
            Ok(()) => {
                inner.is_scanning = false;
                info!("Bluetooth scan stopped successfully");
                Ok(())
            },
            Err(e) => {
                error!("Failed to stop Bluetooth scan: {:?}", e);
                Err(e)
            },
        }
    }

    /// Connect to a device - sync API following ARCHITECTURE.md
    pub fn connect_device(&self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        if !self.available {
            return Err(BluetoothError::NotSupported);
        }

        debug!("Connecting to device: {}", device_id.as_str());

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;

        // Check if device exists in our discovered devices
        if let Some(device) = inner.devices.get_mut(device_id) {
            if device.is_connected() {
                debug!("Device {} already connected", device_id.as_str());
                return Ok(());
            }

            // Update state optimistically
            device.state = ConnectionState::Connecting;

            // Use tokio::task::block_in_place for platform operations - following ClipboardResource
            // pattern
            let result = tokio::task::block_in_place(|| {
                #[cfg(target_os = "macos")]
                {
                    crate::platforms::macos::connect_device(device_id)
                }

                #[cfg(target_os = "windows")]
                {
                    tokio::runtime::Handle::current()
                        .block_on(crate::platforms::windows::connect_device(device_id))
                }

                #[cfg(target_os = "linux")]
                {
                    tokio::runtime::Handle::current()
                        .block_on(crate::platforms::linux::connect_device(device_id))
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
                {
                    error!("Bluetooth connection not supported on this platform");
                    Err(BluetoothError::NotSupported)
                }
            });

            match result {
                Ok(()) => {
                    info!("Connection initiated for device: {}", device_id.as_str());
                    Ok(())
                },
                Err(e) => {
                    error!(
                        "Failed to connect to device {}: {:?}",
                        device_id.as_str(),
                        e
                    );
                    // Revert optimistic state change
                    device.state = ConnectionState::Disconnected;
                    Err(e)
                },
            }
        } else {
            error!(
                "Device {} not found in discovered devices",
                device_id.as_str()
            );
            Err(BluetoothError::DeviceNotFoundWithId {
                id: device_id.as_str().to_string(),
            })
        }
    }

    /// Disconnect from a device - sync API following ARCHITECTURE.md
    pub fn disconnect_device(&self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        if !self.available {
            return Ok(()); // No-op if not available
        }

        debug!("Disconnecting from device: {}", device_id.as_str());

        let mut inner = self
            .inner
            .lock()
            .map_err(|_| BluetoothError::internal("Lock poisoned"))?;

        // Check if device exists
        if let Some(device) = inner.devices.get_mut(device_id) {
            if !device.is_connected() && device.state != ConnectionState::Connecting {
                debug!("Device {} not connected", device_id.as_str());
                return Ok(());
            }

            // Update state optimistically
            device.state = ConnectionState::Disconnecting;

            // Use tokio::task::block_in_place for platform operations - following ClipboardResource
            // pattern
            let result = tokio::task::block_in_place(|| {
                #[cfg(target_os = "macos")]
                {
                    crate::platforms::macos::disconnect_device(device_id)
                }

                #[cfg(target_os = "windows")]
                {
                    tokio::runtime::Handle::current()
                        .block_on(crate::platforms::windows::disconnect_device(device_id))
                }

                #[cfg(target_os = "linux")]
                {
                    tokio::runtime::Handle::current()
                        .block_on(crate::platforms::linux::disconnect_device(device_id))
                }

                #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
                {
                    Ok(()) // No-op for unsupported platforms
                }
            });

            match result {
                Ok(()) => {
                    info!("Disconnection initiated for device: {}", device_id.as_str());
                    Ok(())
                },
                Err(e) => {
                    error!(
                        "Failed to disconnect from device {}: {:?}",
                        device_id.as_str(),
                        e
                    );
                    // Revert optimistic state change
                    device.state = ConnectionState::Connected;
                    Err(e)
                },
            }
        } else {
            error!(
                "Device {} not found in discovered devices",
                device_id.as_str()
            );
            Err(BluetoothError::DeviceNotFoundWithId {
                id: device_id.as_str().to_string(),
            })
        }
    }
}

/// Event bridge system that connects platform events to Bevy events - following clipboard plugin
/// pattern
pub fn bluetooth_event_bridge_system(
    bridge: Res<BluetoothEventBridge>,
    mut event_writer: EventWriter<BluetoothEvent>,
) {
    // Poll all available events from the platform channel
    if let Ok(receiver) = bridge.receiver.lock() {
        while let Ok(event) = receiver.try_recv() {
            debug!("Bridging platform event to Bevy: {:?}", event);
            event_writer.write(event);
        }
    }
}
