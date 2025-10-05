//! Windows Bluetooth implementation using Windows Runtime APIs

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use tracing::{debug, error, info, warn};
use uuid::Uuid;
use windows::Devices::Bluetooth::Advertisement::*;
use windows::Devices::Bluetooth::{BluetoothLEDevice, BluetoothUuidHelper};
use windows::Devices::Enumeration::*;
use windows::Devices::Radios::{Radio, RadioKind, RadioState};
use windows::Foundation::{EventRegistrationToken, IAsyncOperation, TypedEventHandler};
use windows::Storage::Streams::{DataReader, IBuffer};
use windows::core::*;
use windows_core::IntoFuture; // For .into_future().await pattern

use crate::error::{BluetoothError, BluetoothResult};
use crate::types::{
    AdapterState, BluetoothDevice, BluetoothDeviceId, BluetoothEvent, ConnectionState, DeviceInfo,
    ScanOptions,
};

/// Windows Bluetooth manager using Windows Runtime APIs
pub struct WindowsBluetoothManager {
    event_sender: Sender<BluetoothEvent>,
    discovered_devices: Arc<Mutex<HashMap<String, BluetoothDevice>>>,
    is_scanning: Arc<Mutex<bool>>,
    adapter_state: Arc<Mutex<AdapterState>>,
    advertisement_watcher: Option<BluetoothLEAdvertisementWatcher>,
    _received_token: Option<EventRegistrationToken>,
    _stopped_token: Option<EventRegistrationToken>,
    /// Storage for connected device instances to enable proper disconnection
    connected_devices: Arc<Mutex<HashMap<String, BluetoothLEDevice>>>,
}

impl WindowsBluetoothManager {
    /// Convert Windows Runtime buffer to Vec<u8> - following btleplug pattern
    fn buffer_to_vec(buffer: &windows::Storage::Streams::IBuffer) -> Vec<u8> {
        if let Ok(reader) = DataReader::FromBuffer(buffer) {
            if let Ok(len) = reader.UnconsumedBufferLength() {
                let mut data = vec![0u8; len as usize];
                if reader.ReadBytes(&mut data).is_ok() {
                    return data;
                }
            }
        }
        Vec::new()
    }

    /// Convert 16-bit UUID to full UUID - following Bluetooth SIG standard
    fn uuid_from_u16(uuid: u16) -> Uuid {
        // Bluetooth base UUID: 00000000-0000-1000-8000-00805F9B34FB
        // 16-bit UUIDs are inserted at positions 12-13 (0-indexed)
        let mut bytes = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b,
            0x34, 0xfb,
        ];
        bytes[2] = (uuid >> 8) as u8;
        bytes[3] = uuid as u8;
        Uuid::from_bytes(bytes)
    }

    /// Convert 32-bit UUID to full UUID - following Bluetooth SIG standard  
    fn uuid_from_u32(uuid: u32) -> Uuid {
        // Bluetooth base UUID: 00000000-0000-1000-8000-00805F9B34FB
        // 32-bit UUIDs replace the first 4 bytes
        let mut bytes = [
            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x10, 0x00, 0x80, 0x00, 0x00, 0x80, 0x5f, 0x9b,
            0x34, 0xfb,
        ];
        let uuid_bytes = uuid.to_be_bytes();
        bytes[0] = uuid_bytes[0];
        bytes[1] = uuid_bytes[1];
        bytes[2] = uuid_bytes[2];
        bytes[3] = uuid_bytes[3];
        Uuid::from_bytes(bytes)
    }

    pub fn new(event_sender: Sender<BluetoothEvent>) -> Self {
        Self {
            event_sender,
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
            is_scanning: Arc::new(Mutex::new(false)),
            adapter_state: Arc::new(Mutex::new(AdapterState::Unknown)),
            advertisement_watcher: None,
            _received_token: None,
            _stopped_token: None,
            connected_devices: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Initialize the Windows Bluetooth system
    pub fn initialize(&mut self) -> BluetoothResult<()> {
        debug!("Initializing Windows Bluetooth system");

        // Check Bluetooth radio state
        match self.check_bluetooth_availability() {
            Ok(state) => {
                self.update_adapter_state(state);

                // Create advertisement watcher
                let watcher = BluetoothLEAdvertisementWatcher::new().map_err(|e| {
                    error!("Failed to create advertisement watcher: {}", e);
                    BluetoothError::InternalError
                })?;

                // Configure watcher settings
                watcher
                    .SetScanningMode(BluetoothLEScanningMode::Active)
                    .map_err(|_| BluetoothError::InternalError)?;

                self.advertisement_watcher = Some(watcher);
                Ok(())
            },
            Err(e) => {
                error!("Failed to check Bluetooth availability: {:?}", e);
                self.update_adapter_state(AdapterState::Unknown);
                Err(e)
            },
        }
    }

    /// Check Bluetooth availability through Radio Management API
    fn check_bluetooth_availability(&self) -> BluetoothResult<AdapterState> {
        // Use tokio::task::block_in_place to properly await the Windows Runtime future
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Get all system radios
                let radios_future = Radio::GetRadiosAsync().map_err(|e| {
                    error!("Failed to get system radios: {}", e);
                    BluetoothError::InternalError
                })?;

                // Properly await the Windows Runtime async operation
                let radios = radios_future.into_future().await.map_err(|e| {
                    error!("Failed to await radios future: {}", e);
                    BluetoothError::InternalError
                })?;

                // Check if we have any Bluetooth radios
                let radio_count = radios.Size().map_err(|e| {
                    error!("Failed to get radio count: {}", e);
                    BluetoothError::InternalError
                })?;

                if radio_count == 0 {
                    debug!("No Bluetooth radios found");
                    return Ok(AdapterState::Unsupported);
                }

                // Check the state of Bluetooth radios
                for i in 0..radio_count {
                    if let Ok(radio) = radios.GetAt(i) {
                        if let Ok(radio_kind) = radio.Kind() {
                            if radio_kind == RadioKind::Bluetooth {
                                // Found a Bluetooth radio, check its state
                                match radio.State() {
                                    Ok(RadioState::On) => {
                                        debug!("Bluetooth radio is powered on");
                                        return Ok(AdapterState::PoweredOn);
                                    },
                                    Ok(RadioState::Off) => {
                                        debug!("Bluetooth radio is powered off");
                                        return Ok(AdapterState::PoweredOff);
                                    },
                                    Ok(RadioState::Disabled) => {
                                        debug!("Bluetooth radio is disabled");
                                        return Ok(AdapterState::Unauthorized);
                                    },
                                    _ => {
                                        debug!("Bluetooth radio state unknown");
                                        return Ok(AdapterState::Unknown);
                                    },
                                }
                            }
                        }
                    }
                }

                debug!("No Bluetooth radios found in system");
                Ok(AdapterState::Unsupported)
            })
        })
    }

    /// Start scanning for Bluetooth devices
    pub fn start_scan(&mut self, options: ScanOptions) -> BluetoothResult<()> {
        debug!(
            "Starting Windows Bluetooth LE scan with options: {:?}",
            options
        );

        // Check if already scanning
        if *self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)?
        {
            return Ok(());
        }

        let watcher = self
            .advertisement_watcher
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Configure service UUID filters if specified
        if !options.service_uuids.is_empty() {
            let advertisement_filter = watcher
                .AdvertisementFilter()
                .map_err(|_| BluetoothError::InternalError)?;

            for uuid in &options.service_uuids {
                let guid = self.uuid_to_guid(uuid)?;
                advertisement_filter
                    .Advertisement()
                    .map_err(|_| BluetoothError::InternalError)?
                    .ServiceUuids()
                    .map_err(|_| BluetoothError::InternalError)?
                    .Append(&guid)
                    .map_err(|_| BluetoothError::InternalError)?;
            }
        }

        // Set up event handlers
        let discovered_devices = Arc::clone(&self.discovered_devices);
        let event_sender = self.event_sender.clone();

        let received_token = watcher
            .Received(&TypedEventHandler::<
                BluetoothLEAdvertisementWatcher,
                BluetoothLEAdvertisementReceivedEventArgs,
            >::new({
                move |_watcher, args| {
                    if let Some(args) = args {
                        let device_id = BluetoothDeviceId::new(format!(
                            "{:016X}",
                            args.BluetoothAddress().unwrap_or(0)
                        ));

                        let name = args.LocalName().ok().and_then(|hstring| {
                            let name_str = hstring.to_string();
                            if name_str.is_empty() {
                                None
                            } else {
                                Some(name_str)
                            }
                        });

                        let rssi = args.RawSignalStrengthInDBm().ok();

                        // Extract advertisement data from Windows Runtime collections - following
                        // btleplug pattern
                        let advertisement = args.Advertisement().ok();

                        // Extract manufacturer data
                        let mut manufacturer_data = HashMap::new();
                        if let Some(ref adv) = advertisement {
                            if let Ok(mfg_data) = adv.ManufacturerData() {
                                for data_entry in &mfg_data {
                                    if let (Ok(company_id), Ok(buffer)) =
                                        (data_entry.CompanyId(), data_entry.Data())
                                    {
                                        let data = Self::buffer_to_vec(&buffer);
                                        manufacturer_data.insert(company_id, data);
                                    }
                                }
                            }
                        }

                        // Extract service UUIDs
                        let mut services = Vec::new();
                        if let Some(ref adv) = advertisement {
                            if let Ok(service_uuids) = adv.ServiceUuids() {
                                for uuid in &service_uuids {
                                    if let Ok(uuid_str) = uuid.ToString() {
                                        if let Ok(parsed_uuid) =
                                            Uuid::parse_str(&uuid_str.to_string())
                                        {
                                            services.push(parsed_uuid);
                                        }
                                    }
                                }
                            }
                        }

                        // Extract service data from raw data sections (Windows doesn't provide
                        // direct service data API)
                        let mut service_data = HashMap::new();
                        if let Some(ref adv) = advertisement {
                            if let Ok(data_sections) = adv.DataSections() {
                                for section in &data_sections {
                                    if let (Ok(data_type), Ok(buffer)) =
                                        (section.DataType(), section.Data())
                                    {
                                        // Parse service data based on advertisement data type
                                        // Types 0x16 (16-bit), 0x20 (32-bit), 0x21 (128-bit service
                                        // data)
                                        let data = Self::buffer_to_vec(&buffer);

                                        match data_type {
                                            0x16 => {
                                                // 16-bit service data
                                                if data.len() >= 2 {
                                                    let uuid_bytes = &data[0..2];
                                                    let service_uuid =
                                                        Self::uuid_from_u16(u16::from_le_bytes([
                                                            uuid_bytes[0],
                                                            uuid_bytes[1],
                                                        ]));
                                                    service_data
                                                        .insert(service_uuid, data[2..].to_vec());
                                                }
                                            },
                                            0x20 => {
                                                // 32-bit service data
                                                if data.len() >= 4 {
                                                    let uuid_bytes = &data[0..4];
                                                    let service_uuid =
                                                        Self::uuid_from_u32(u32::from_le_bytes([
                                                            uuid_bytes[0],
                                                            uuid_bytes[1],
                                                            uuid_bytes[2],
                                                            uuid_bytes[3],
                                                        ]));
                                                    service_data
                                                        .insert(service_uuid, data[4..].to_vec());
                                                }
                                            },
                                            0x21 => {
                                                // 128-bit service data
                                                if data.len() >= 16 {
                                                    if let Ok(service_uuid) =
                                                        Uuid::from_slice(&data[0..16])
                                                    {
                                                        service_data.insert(
                                                            service_uuid,
                                                            data[16..].to_vec(),
                                                        );
                                                    }
                                                }
                                            },
                                            _ => {}, // Other data types not handled
                                        }
                                    }
                                }
                            }
                        }

                        let device_info = DeviceInfo {
                            id: device_id.clone(),
                            name,
                            address: format!("{:016X}", args.BluetoothAddress().unwrap_or(0)),
                            rssi,
                            services,
                            manufacturer_data,
                            service_data,
                            tx_power: args.TransmitPowerLevelInDBm().ok(),
                            connectable: true, // Most LE devices are connectable by default
                        };

                        let device = BluetoothDevice::new(device_info);

                        // Store device and determine if it's new
                        let is_new_device = {
                            if let Ok(mut devices) = discovered_devices.lock() {
                                let is_new = !devices.contains_key(device_id.as_str());
                                devices.insert(device_id.as_str().to_string(), device.clone());
                                is_new
                            } else {
                                false // Default to false if lock fails
                            }
                        };

                        // Send appropriate event
                        let event = if is_new_device {
                            BluetoothEvent::DeviceDiscovered(device)
                        } else {
                            BluetoothEvent::DeviceUpdated(device)
                        };

                        let _ = event_sender.send(event);
                    }
                    Ok(())
                }
            }))
            .map_err(|_| BluetoothError::InternalError)?;

        let is_scanning = Arc::clone(&self.is_scanning);
        let event_sender_stopped = self.event_sender.clone();

        let stopped_token = watcher
            .Stopped(&TypedEventHandler::<
                BluetoothLEAdvertisementWatcher,
                BluetoothLEAdvertisementWatcherStoppedEventArgs,
            >::new({
                move |_watcher, _args| {
                    if let Ok(mut scanning) = is_scanning.lock() {
                        *scanning = false;
                    }
                    let _ = event_sender_stopped.send(BluetoothEvent::ScanStopped);
                    Ok(())
                }
            }))
            .map_err(|_| BluetoothError::InternalError)?;

        // Store event tokens for cleanup
        self._received_token = Some(received_token);
        self._stopped_token = Some(stopped_token);

        // Start the watcher
        watcher.Start().map_err(|e| {
            error!("Failed to start advertisement watcher: {}", e);
            BluetoothError::InternalError
        })?;

        *self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)? = true;
        self.send_event(BluetoothEvent::ScanStarted);

        // Set up scan timeout if specified
        if let Some(duration) = options.duration {
            let watcher_weak = watcher.clone(); // Clone for timeout thread
            std::thread::spawn(move || {
                std::thread::sleep(duration);
                if let Err(e) = watcher_weak.Stop() {
                    error!("Failed to stop watcher on timeout: {}", e);
                }
            });
        }

        Ok(())
    }

    /// Stop scanning for devices
    pub fn stop_scan(&mut self) -> BluetoothResult<()> {
        debug!("Stopping Windows Bluetooth LE scan");

        if !*self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)?
        {
            return Ok(());
        }

        if let Some(watcher) = &self.advertisement_watcher {
            watcher.Stop().map_err(|e| {
                error!("Failed to stop advertisement watcher: {}", e);
                BluetoothError::InternalError
            })?;
        }

        Ok(())
    }

    /// Connect to a Bluetooth device - using real async operations
    pub async fn connect_device(&mut self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        debug!(
            "Connecting to Windows Bluetooth device: {}",
            device_id.as_str()
        );

        // Find the device in our discovered devices
        let device_address = {
            let devices = self
                .discovered_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;

            let device = devices
                .get(device_id.as_str())
                .ok_or(BluetoothError::DeviceNotFound)?;

            // Parse the device address from the stored address string
            u64::from_str_radix(&device.info.address, 16)
                .map_err(|_| BluetoothError::InternalError)?
        };

        // Update connection state
        self.update_device_connection_state(device_id, ConnectionState::Connecting);

        // Use real Windows Runtime async operation - following btleplug pattern
        let async_op =
            BluetoothLEDevice::FromBluetoothAddressAsync(device_address).map_err(|e| {
                error!(
                    "Failed to create async operation for device connection: {}",
                    e
                );
                BluetoothError::InternalError
            })?;

        // Properly await the Windows Runtime future
        match async_op.into_future().await {
            Ok(device) => {
                if let Some(_device) = device {
                    // Store the device instance for proper disconnection
                    if let Ok(mut connected_devices) = self.connected_devices.lock() {
                        connected_devices.insert(device_id.as_str().to_string(), _device.clone());
                    }

                    debug!("Successfully connected to device: {}", device_id.as_str());
                    self.update_device_connection_state(device_id, ConnectionState::Connected);
                } else {
                    warn!(
                        "Device connection returned null for: {}",
                        device_id.as_str()
                    );
                    self.update_device_connection_state(device_id, ConnectionState::Disconnected);
                    return Err(BluetoothError::DeviceNotFound);
                }
            },
            Err(e) => {
                error!("Failed to connect to device {}: {}", device_id.as_str(), e);
                self.update_device_connection_state(device_id, ConnectionState::Disconnected);
                return Err(BluetoothError::InternalError);
            },
        }

        Ok(())
    }

    /// Disconnect from a Bluetooth device - using real async operations
    pub async fn disconnect_device(
        &mut self,
        device_id: &BluetoothDeviceId,
    ) -> BluetoothResult<()> {
        debug!(
            "Disconnecting from Windows Bluetooth device: {}",
            device_id.as_str()
        );

        // Update connection state
        self.update_device_connection_state(device_id, ConnectionState::Disconnecting);

        // Get the stored device instance
        let device_instance = {
            let mut connected_devices = self
                .connected_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;

            connected_devices.remove(device_id.as_str())
        };

        match device_instance {
            Some(device) => {
                // Call the proper Windows Runtime Close() method
                if let Err(e) = device.Close() {
                    error!("Failed to close device connection: {}", e);
                    self.update_device_connection_state(device_id, ConnectionState::Connected);
                    return Err(BluetoothError::InternalError);
                }

                self.update_device_connection_state(device_id, ConnectionState::Disconnected);
                debug!("Device {} disconnected successfully", device_id.as_str());
            },
            None => {
                warn!(
                    "Device {} not found in connected devices",
                    device_id.as_str()
                );
                // Still update state to disconnected since device wasn't tracked as connected
                self.update_device_connection_state(device_id, ConnectionState::Disconnected);
            },
        }

        Ok(())
    }

    /// Convert Uuid to Windows GUID
    fn uuid_to_guid(&self, uuid: &Uuid) -> BluetoothResult<windows::core::GUID> {
        let uuid_bytes = uuid.as_bytes();

        // Convert UUID bytes to Windows GUID format
        let data1 =
            u32::from_be_bytes([uuid_bytes[0], uuid_bytes[1], uuid_bytes[2], uuid_bytes[3]]);
        let data2 = u16::from_be_bytes([uuid_bytes[4], uuid_bytes[5]]);
        let data3 = u16::from_be_bytes([uuid_bytes[6], uuid_bytes[7]]);
        let data4 = [
            uuid_bytes[8],
            uuid_bytes[9],
            uuid_bytes[10],
            uuid_bytes[11],
            uuid_bytes[12],
            uuid_bytes[13],
            uuid_bytes[14],
            uuid_bytes[15],
        ];

        Ok(windows::core::GUID {
            data1,
            data2,
            data3,
            data4,
        })
    }

    /// Update adapter state and notify
    fn update_adapter_state(&mut self, new_state: AdapterState) {
        if let Ok(mut state) = self.adapter_state.lock() {
            *state = new_state;
        }
        self.send_event(BluetoothEvent::AdapterStateChanged(new_state));
    }

    /// Update device connection state
    fn update_device_connection_state(
        &mut self,
        device_id: &BluetoothDeviceId,
        state: ConnectionState,
    ) {
        if let Ok(mut devices) = self.discovered_devices.lock() {
            if let Some(device) = devices.get_mut(device_id.as_str()) {
                device.state = state;
                device.last_seen = SystemTime::now();
            }
        }

        self.send_event(BluetoothEvent::ConnectionStateChanged {
            device_id: device_id.clone(),
            state,
        });
    }

    /// Send event through the event sender
    fn send_event(&self, event: BluetoothEvent) {
        let _ = self.event_sender.send(event);
    }
}

// Thread-local storage for the global manager instance
thread_local! {
    static MANAGER: Arc<Mutex<Option<WindowsBluetoothManager>>> = Arc::new(Mutex::new(None));
}

/// Initialize Windows Bluetooth on main thread
pub fn initialize_bluetooth(event_sender: Sender<BluetoothEvent>) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.lock().map_err(|_| BluetoothError::InternalError)?;
        let mut bluetooth_manager = WindowsBluetoothManager::new(event_sender);
        bluetooth_manager.initialize()?;
        *mgr = Some(bluetooth_manager);
        Ok(())
    })
}

/// Start scanning for devices
pub fn start_scan(options: ScanOptions) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.lock().map_err(|_| BluetoothError::InternalError)?;
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .start_scan(options)
    })
}

/// Stop scanning for devices
pub fn stop_scan() -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.lock().map_err(|_| BluetoothError::InternalError)?;
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .stop_scan()
    })
}

/// Connect to a device - async operation
pub async fn connect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    let manager_ref = MANAGER.with(|manager| manager.clone());
    let mut mgr = manager_ref
        .lock()
        .map_err(|_| BluetoothError::InternalError)?;
    mgr.as_mut()
        .ok_or(BluetoothError::NotInitialized)?
        .connect_device(device_id)
        .await
}

/// Test if Bluetooth is available on Windows
pub fn test_availability() -> bool {
    // Test if we can access Windows Bluetooth APIs - indicates Bluetooth is available
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Try to get the default Bluetooth radio - this indicates Bluetooth hardware is
            // available
            match Radio::GetRadiosAsync() {
                Ok(async_op) => {
                    if let Ok(radios) = async_op.into_future().await {
                        radios.Size().unwrap_or(0) > 0
                    } else {
                        false
                    }
                },
                Err(_) => false,
            }
        })
    })
}

/// Disconnect from a device - async operation  
pub async fn disconnect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    let manager_ref = MANAGER.with(|manager| manager.clone());
    let mut mgr = manager_ref
        .lock()
        .map_err(|_| BluetoothError::InternalError)?;
    mgr.as_mut()
        .ok_or(BluetoothError::NotInitialized)?
        .disconnect_device(device_id)
        .await
}
