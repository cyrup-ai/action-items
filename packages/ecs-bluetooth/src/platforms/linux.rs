//! Linux Bluetooth implementation using BlueZ via zbus D-Bus

use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use futures_util::stream::StreamExt;
use tracing::{debug, error, info, warn};
use uuid::Uuid;
use zbus::{Connection, proxy};
use zvariant::{ObjectPath, OwnedValue, Value};

use crate::error::{BluetoothError, BluetoothResult};
use crate::types::{
    AdapterState, BluetoothDevice, BluetoothDeviceId, BluetoothEvent, ConnectionState, DeviceInfo,
    ScanOptions,
};

// BlueZ D-Bus interface definitions using zbus proxy macro

#[proxy(
    default_service = "org.bluez",
    interface = "org.bluez.Adapter1",
    default_path = "/org/bluez/hci0"
)]
trait BlueZAdapter1 {
    /// Start device discovery
    fn start_discovery(&self) -> zbus::Result<()>;

    /// Stop device discovery  
    fn stop_discovery(&self) -> zbus::Result<()>;

    /// Remove device from known devices
    fn remove_device(&self, device: &ObjectPath<'_>) -> zbus::Result<()>;

    /// Set discovery filter for specific device types/services
    fn set_discovery_filter(&self, properties: HashMap<&str, Value<'_>>) -> zbus::Result<()>;

    // Properties
    #[zbus(property)]
    fn powered(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn set_powered(&self, powered: bool) -> zbus::Result<()>;

    #[zbus(property)]
    fn discoverable(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn discovering(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn uuids(&self) -> zbus::Result<Vec<String>>;
}

#[proxy(default_service = "org.bluez", interface = "org.bluez.Device1")]
trait BlueZDevice1 {
    /// Connect to this device
    fn connect(&self) -> zbus::Result<()>;

    /// Disconnect from this device
    fn disconnect(&self) -> zbus::Result<()>;

    /// Pair with this device
    fn pair(&self) -> zbus::Result<()>;

    // Properties
    #[zbus(property)]
    fn address(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn name(&self) -> zbus::Result<Option<String>>;

    #[zbus(property)]
    fn alias(&self) -> zbus::Result<String>;

    #[zbus(property)]
    fn paired(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn connected(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn trusted(&self) -> zbus::Result<bool>;

    #[zbus(property)]
    fn rssi(&self) -> zbus::Result<Option<i16>>;

    #[zbus(property)]
    fn uuids(&self) -> zbus::Result<Vec<String>>;

    #[zbus(property)]
    fn manufacturer_data(&self) -> zbus::Result<HashMap<u16, Vec<u8>>>;

    #[zbus(property)]
    fn service_data(&self) -> zbus::Result<HashMap<String, Vec<u8>>>;

    #[zbus(property)]
    fn tx_power(&self) -> zbus::Result<Option<i16>>;
}

#[proxy(
    default_service = "org.bluez",
    default_path = "/",
    interface = "org.freedesktop.DBus.ObjectManager"
)]
trait ObjectManager {
    #[zbus(signal)]
    fn interfaces_added(
        &self,
        object_path: ObjectPath<'_>,
        interfaces_and_properties: HashMap<String, HashMap<String, OwnedValue>>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    fn interfaces_removed(
        &self,
        object_path: ObjectPath<'_>,
        interfaces: Vec<String>,
    ) -> zbus::Result<()>;

    #[zbus(signal)]
    fn properties_changed(
        &self,
        interface_name: String,
        changed_properties: HashMap<String, OwnedValue>,
        invalidated_properties: Vec<String>,
    ) -> zbus::Result<()>;
}

/// Linux Bluetooth manager using BlueZ via zbus D-Bus
pub struct LinuxBluetoothManager {
    event_sender: Sender<BluetoothEvent>,
    discovered_devices: Arc<Mutex<HashMap<String, BluetoothDevice>>>,
    is_scanning: Arc<Mutex<bool>>,
    adapter_state: Arc<Mutex<AdapterState>>,
    connection: Option<Connection>,
    adapter_proxy: Option<BlueZAdapter1Proxy<'static>>,
    object_manager_proxy: Option<ObjectManagerProxy<'static>>,
}

impl LinuxBluetoothManager {
    pub fn new(event_sender: Sender<BluetoothEvent>) -> Self {
        Self {
            event_sender,
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
            is_scanning: Arc::new(Mutex::new(false)),
            adapter_state: Arc::new(Mutex::new(AdapterState::Unknown)),
            connection: None,
            adapter_proxy: None,
            object_manager_proxy: None,
        }
    }

    /// Initialize the Linux Bluetooth system - using real zbus async connection
    pub async fn initialize(&mut self) -> BluetoothResult<()> {
        debug!("Initializing Linux BlueZ connection via D-Bus");

        // Create real D-Bus connection - following BlueBus pattern
        let connection = Connection::system().await.map_err(|e| {
            error!("Failed to connect to system D-Bus: {}", e);
            BluetoothError::InternalError
        })?;

        // Create BlueZ adapter proxy
        let adapter_proxy = BlueZAdapter1Proxy::new(&connection).await.map_err(|e| {
            error!("Failed to create adapter proxy: {}", e);
            BluetoothError::InternalError
        })?;

        // Create object manager proxy for device discovery signals
        let object_manager = ObjectManagerProxy::new(&connection).await.map_err(|e| {
            error!("Failed to create object manager proxy: {}", e);
            BluetoothError::InternalError
        })?;

        // Check adapter power state using real D-Bus property access
        let adapter_state = match adapter_proxy.powered().await {
            Ok(true) => AdapterState::PoweredOn,
            Ok(false) => AdapterState::PoweredOff,
            Err(e) => {
                warn!("Failed to read adapter power state: {}", e);
                AdapterState::Unknown
            },
        };

        // Store connections and proxies
        self.connection = Some(connection);
        self.adapter_proxy = Some(adapter_proxy);
        self.object_manager_proxy = Some(object_manager);

        self.update_adapter_state(adapter_state);
        info!("BlueZ D-Bus initialization completed successfully");

        Ok(())
    }

    /// Start scanning for Bluetooth devices using real BlueZ D-Bus signals
    pub async fn start_scan(&mut self, options: ScanOptions) -> BluetoothResult<()> {
        debug!(
            "Starting Linux BlueZ discovery via D-Bus with options: {:?}",
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

        let adapter_proxy = self
            .adapter_proxy
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        let object_manager = self
            .object_manager_proxy
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Set discovery filter if service UUIDs are specified
        if !options.service_uuids.is_empty() {
            let mut filter_props = HashMap::new();
            let uuids: Vec<Value> = options
                .service_uuids
                .iter()
                .map(|uuid| Value::Str(uuid.to_string().into()))
                .collect();
            filter_props.insert("UUIDs", Value::Array(uuids.into()));

            if let Err(e) = adapter_proxy.set_discovery_filter(filter_props).await {
                warn!("Failed to set discovery filter: {}", e);
            }
        }

        // Start BlueZ discovery
        adapter_proxy.start_discovery().await.map_err(|e| {
            error!("Failed to start BlueZ discovery: {}", e);
            BluetoothError::InternalError
        })?;

        *self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)? = true;
        self.send_event(BluetoothEvent::ScanStarted);

        // Set up real BlueZ signal stream for device discovery - following BlueBus pattern
        let event_sender = self.event_sender.clone();
        let discovered_devices = Arc::clone(&self.discovered_devices);
        let is_scanning = Arc::clone(&self.is_scanning);
        let mut interfaces_added =
            object_manager
                .receive_interfaces_added()
                .await
                .map_err(|e| {
                    error!("Failed to create interfaces_added signal stream: {}", e);
                    BluetoothError::InternalError
                })?;

        // Spawn task to handle real BlueZ InterfacesAdded signals
        tokio::spawn(async move {
            while let Some(signal) = interfaces_added.next().await {
                // Check if scanning was stopped
                let should_continue = is_scanning
                    .lock()
                    .map(|scanning| *scanning)
                    .unwrap_or(false);

                if !should_continue {
                    break; // Scan was stopped
                }

                if let Ok(args) = signal.args() {
                    // Check if this is a Device1 interface (Bluetooth device)
                    if let Some(device_props) = args.interfaces().get("org.bluez.Device1") {
                        // Extract device properties from BlueZ D-Bus signal - following BlueBus
                        // pattern
                        let address = device_props
                            .get("Address")
                            .and_then(|v| v.downcast_ref::<zvariant::Str>().ok())
                            .map(|s| s.as_str().to_string())
                            .unwrap_or_default();

                        let name = device_props
                            .get("Alias")
                            .and_then(|v| v.downcast_ref::<zvariant::Str>().ok())
                            .map(|s| s.as_str().to_string());

                        let rssi = device_props
                            .get("RSSI")
                            .and_then(|v| v.downcast_ref::<i16>().ok())
                            .copied();

                        // Extract UUIDs (service advertisements)
                        let services = device_props
                            .get("UUIDs")
                            .and_then(|v| v.downcast_ref::<zvariant::Array>().ok())
                            .map(|arr| {
                                arr.iter()
                                    .filter_map(|v| {
                                        v.downcast_ref::<zvariant::Str>()
                                            .ok()
                                            .and_then(|s| Uuid::parse_str(s.as_str()).ok())
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        // Extract manufacturer data
                        let manufacturer_data = device_props
                            .get("ManufacturerData")
                            .and_then(|v| v.downcast_ref::<HashMap<u16, Vec<u8>>>().ok())
                            .cloned()
                            .unwrap_or_default();

                        // Extract service data
                        let service_data = device_props
                            .get("ServiceData")
                            .and_then(|v| v.downcast_ref::<HashMap<String, Vec<u8>>>().ok())
                            .map(|data| {
                                data.iter()
                                    .filter_map(|(k, v)| {
                                        Uuid::parse_str(k).ok().map(|uuid| (uuid, v.clone()))
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();

                        let tx_power = device_props
                            .get("TxPower")
                            .and_then(|v| v.downcast_ref::<i16>().ok())
                            .map(|&p| p as i8);

                        if !address.is_empty() {
                            let device_id = BluetoothDeviceId::new(address.clone());

                            let device_info = DeviceInfo {
                                id: device_id.clone(),
                                name,
                                address,
                                rssi,
                                services,
                                manufacturer_data,
                                service_data,
                                tx_power,
                                connectable: true, // Assume connectable if discovered
                            };

                            let device = BluetoothDevice::new(device_info);

                            // Store device and check if it's new
                            let is_new_device = {
                                if let Ok(mut devices) = discovered_devices.lock() {
                                    let is_new = !devices.contains_key(device_id.as_str());
                                    devices.insert(device_id.as_str().to_string(), device.clone());
                                    is_new
                                } else {
                                    false
                                }
                            };

                            // Send appropriate discovery event
                            let event = if is_new_device {
                                BluetoothEvent::DeviceDiscovered(device)
                            } else {
                                BluetoothEvent::DeviceUpdated(device)
                            };

                            let _ = event_sender.send(event);
                        }
                    }
                }
            }
        });

        // Handle scan timeout if specified
        if let Some(duration) = options.duration {
            let event_sender_timeout = self.event_sender.clone();
            let is_scanning_timeout = Arc::clone(&self.is_scanning);
            let adapter_proxy_timeout = adapter_proxy.clone();

            tokio::spawn(async move {
                tokio::time::sleep(duration).await;

                // Stop discovery
                if let Err(e) = adapter_proxy_timeout.stop_discovery().await {
                    warn!("Failed to stop discovery on timeout: {}", e);
                }

                // Update scanning state
                if let Ok(mut scanning) = is_scanning_timeout.lock() {
                    *scanning = false;
                }

                let _ = event_sender_timeout.send(BluetoothEvent::ScanStopped);
            });
        }

        Ok(())
    }

    /// Stop scanning for devices
    pub fn stop_scan(&mut self) -> BluetoothResult<()> {
        debug!("Stopping Linux BlueZ discovery via D-Bus");

        if !*self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)?
        {
            return Ok(());
        }

        // Use real BlueZ adapter proxy to stop discovery
        let adapter_proxy = self
            .adapter_proxy
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Call real BlueZ stop_discovery D-Bus method
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                adapter_proxy.stop_discovery().await.map_err(|e| {
                    error!("Failed to stop BlueZ discovery: {}", e);
                    BluetoothError::InternalError
                })
            })
        })?;

        *self
            .is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)? = false;
        self.send_event(BluetoothEvent::ScanStopped);

        Ok(())
    }

    /// Connect to a Bluetooth device using BlueZ
    pub fn connect_device(&mut self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        debug!("Connecting to Linux BlueZ device: {}", device_id.as_str());

        // Find the device in our discovered devices
        let device_address = {
            let devices = self
                .discovered_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;

            let device = devices
                .get(device_id.as_str())
                .ok_or(BluetoothError::DeviceNotFound)?;

            device.info.address.clone()
        };

        // Update connection state
        self.update_device_connection_state(device_id, ConnectionState::Connecting);

        // Create BlueZ device path from address - following D-Bus object path convention
        let device_path = format!("/org/bluez/hci0/dev_{}", device_address.replace(':', "_"));

        // Get D-Bus connection
        let connection = self
            .connection
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Create real BlueZ device proxy and connect
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create device proxy for the specific device
                match BlueZDevice1Proxy::builder(connection)
                    .path(device_path.clone())
                    .map_err(|e| {
                        error!("Failed to create device proxy for {}: {}", device_path, e);
                        BluetoothError::InternalError
                    })?
                    .build()
                    .await
                {
                    Ok(device_proxy) => {
                        // Call real BlueZ connect D-Bus method
                        if let Err(e) = device_proxy.connect().await {
                            error!(
                                "Failed to connect to BlueZ device {}: {}",
                                device_address, e
                            );
                            return Err(BluetoothError::ConnectionFailed);
                        }

                        debug!(
                            "Successfully initiated BlueZ connection to {}",
                            device_address
                        );
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to build device proxy for {}: {}", device_path, e);
                        Err(BluetoothError::InternalError)
                    },
                }
            })
        })?;

        Ok(())
    }

    /// Disconnect from a Bluetooth device using BlueZ
    pub fn disconnect_device(&mut self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        debug!(
            "Disconnecting from Linux BlueZ device: {}",
            device_id.as_str()
        );

        // Find the device address for creating the correct D-Bus path
        let device_address = {
            let devices = self
                .discovered_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;

            let device = devices
                .get(device_id.as_str())
                .ok_or(BluetoothError::DeviceNotFound)?;

            device.info.address.clone()
        };

        // Update connection state
        self.update_device_connection_state(device_id, ConnectionState::Disconnecting);

        // Create BlueZ device path from address - following D-Bus object path convention
        let device_path = format!("/org/bluez/hci0/dev_{}", device_address.replace(':', "_"));

        // Get D-Bus connection
        let connection = self
            .connection
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Create real BlueZ device proxy and disconnect
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create device proxy for the specific device
                match BlueZDevice1Proxy::builder(connection)
                    .path(device_path.clone())
                    .map_err(|e| {
                        error!("Failed to create device proxy for {}: {}", device_path, e);
                        BluetoothError::InternalError
                    })?
                    .build()
                    .await
                {
                    Ok(device_proxy) => {
                        // Call real BlueZ disconnect D-Bus method
                        if let Err(e) = device_proxy.disconnect().await {
                            error!(
                                "Failed to disconnect from BlueZ device {}: {}",
                                device_address, e
                            );
                            return Err(BluetoothError::InternalError);
                        }

                        debug!(
                            "Successfully initiated BlueZ disconnection from {}",
                            device_address
                        );
                        Ok(())
                    },
                    Err(e) => {
                        error!("Failed to build device proxy for {}: {}", device_path, e);
                        Err(BluetoothError::InternalError)
                    },
                }
            })
        })?;

        Ok(())
    }

    /// Convert service UUID string to proper GATT service format
    fn format_service_uuid(&self, uuid_str: &str) -> String {
        // BlueZ uses different UUID formats (16-bit, 32-bit, 128-bit)
        // Parse and normalize UUID to standard 128-bit format
        if let Ok(parsed_uuid) = Uuid::parse_str(uuid_str) {
            // Return properly formatted UUID string in standard hyphenated format
            parsed_uuid.to_string().to_uppercase()
        } else {
            // If parsing fails, try to handle short UUIDs (16-bit, 32-bit)
            let clean_uuid = uuid_str.replace(['-', ' '], "");

            match clean_uuid.len() {
                4 => {
                    // 16-bit UUID - convert to full 128-bit using Bluetooth base UUID
                    // 0000XXXX-0000-1000-8000-00805F9B34FB
                    format!(
                        "0000{}-0000-1000-8000-00805F9B34FB",
                        clean_uuid.to_uppercase()
                    )
                },
                8 => {
                    // 32-bit UUID - convert to full 128-bit using Bluetooth base UUID
                    // XXXXXXXX-0000-1000-8000-00805F9B34FB
                    format!("{}-0000-1000-8000-00805F9B34FB", clean_uuid.to_uppercase())
                },
                32 => {
                    // 128-bit UUID without hyphens - add proper formatting
                    let uuid = clean_uuid.to_uppercase();
                    format!(
                        "{}-{}-{}-{}-{}",
                        &uuid[0..8],
                        &uuid[8..12],
                        &uuid[12..16],
                        &uuid[16..20],
                        &uuid[20..32]
                    )
                },
                _ => {
                    // Invalid format - return original string uppercased as fallback
                    warn!("Invalid UUID format: {}", uuid_str);
                    uuid_str.to_uppercase()
                },
            }
        }
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
    static MANAGER: Arc<Mutex<Option<LinuxBluetoothManager>>> = Arc::new(Mutex::new(None));
}

/// Initialize Linux Bluetooth with BlueZ via zbus - async operation
pub async fn initialize_bluetooth(event_sender: Sender<BluetoothEvent>) -> BluetoothResult<()> {
    let manager_ref = MANAGER.with(|manager| manager.clone());
    let mut mgr = manager_ref
        .lock()
        .map_err(|_| BluetoothError::InternalError)?;
    let mut bluetooth_manager = LinuxBluetoothManager::new(event_sender);
    bluetooth_manager.initialize().await?;
    *mgr = Some(bluetooth_manager);
    Ok(())
}

/// Start scanning for devices
pub async fn start_scan(options: ScanOptions) -> BluetoothResult<()> {
    let manager_ref = MANAGER.with(|manager| manager.clone());
    let mut mgr = manager_ref
        .lock()
        .map_err(|_| BluetoothError::InternalError)?;
    mgr.as_mut()
        .ok_or(BluetoothError::NotInitialized)?
        .start_scan(options)
        .await
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

/// Connect to a device
pub fn connect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.lock().map_err(|_| BluetoothError::InternalError)?;
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .connect_device(device_id)
    })
}

/// Test if Bluetooth is available on Linux
pub fn test_availability() -> bool {
    // Test if we can access the D-Bus system bus - indicates BlueZ is available
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current()
            .block_on(async { zbus::Connection::system().await.is_ok() })
    })
}

/// Disconnect from a device
pub fn disconnect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.lock().map_err(|_| BluetoothError::InternalError)?;
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .disconnect_device(device_id)
    })
}
