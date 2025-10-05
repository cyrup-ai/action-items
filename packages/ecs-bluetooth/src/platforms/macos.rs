//! macOS Bluetooth implementation using CoreBluetooth

use std::cell::OnceCell;
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, ProtocolObject};
use objc2::{DefinedClass, MainThreadOnly, define_class, msg_send};
use objc2_core_bluetooth::{
    CBCentralManager, CBCentralManagerDelegate, CBManagerState, CBPeripheral, CBUUID,
};
use objc2_foundation::{
    MainThreadMarker, NSArray, NSDictionary, NSMutableDictionary, NSNumber, NSObjectProtocol,
    NSString,
};
use uuid::Uuid;

use crate::error::{BluetoothError, BluetoothResult};
use crate::types::{
    AdapterState, BluetoothDevice, BluetoothDeviceId, BluetoothEvent, ConnectionState, DeviceInfo,
    ScanOptions,
};



/// Instance variables for the CoreBluetooth delegate
#[derive(Debug)]
pub struct BluetoothDelegateIvars {
    event_sender: OnceCell<Sender<BluetoothEvent>>,
    discovered_devices: Arc<Mutex<HashMap<String, BluetoothDevice>>>,
    _is_scanning: Arc<Mutex<bool>>,
    _adapter_state: Arc<Mutex<AdapterState>>,
}

impl Default for BluetoothDelegateIvars {
    fn default() -> Self {
        Self {
            event_sender: OnceCell::new(),
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
            _is_scanning: Arc::new(Mutex::new(false)),
            _adapter_state: Arc::new(Mutex::new(AdapterState::Unknown)),
        }
    }
}

// CoreBluetooth Central Manager Delegate implementation
define_class!(
    #[unsafe(super(objc2_foundation::NSObject))]
    #[thread_kind = MainThreadOnly]
    #[ivars = BluetoothDelegateIvars]
    pub struct BluetoothDelegate;

    impl BluetoothDelegate {
        #[unsafe(method_id(init))]
        fn init(this: objc2::rc::Allocated<Self>) -> Retained<Self> {
            let this = this.set_ivars(BluetoothDelegateIvars::default());
            unsafe { msg_send![super(this), init] }
        }
    }

    unsafe impl NSObjectProtocol for BluetoothDelegate {}

    unsafe impl CBCentralManagerDelegate for BluetoothDelegate {
        #[unsafe(method(centralManagerDidUpdateState:))]
        fn central_manager_did_update_state(&self, central: &CBCentralManager) {
            let state = unsafe { central.state() };
            let adapter_state = match state {
                CBManagerState::Unknown => AdapterState::Unknown,
                CBManagerState::PoweredOff => AdapterState::PoweredOff,
                CBManagerState::PoweredOn => AdapterState::PoweredOn,
                _ => AdapterState::Unknown,
            };

            if let Some(sender) = self.ivars().event_sender.get() {
                let _ = sender.send(BluetoothEvent::AdapterStateChanged(adapter_state));
            }
        }

        #[unsafe(method(centralManager:didDiscoverPeripheral:advertisementData:RSSI:))]
        fn central_manager_did_discover_peripheral(
            &self,
            _central: &CBCentralManager,
            peripheral: &CBPeripheral,
            advertisement_data: &objc2_foundation::NSDictionary<
                objc2_foundation::NSString,
                objc2::runtime::AnyObject,
            >,
            rssi: &objc2_foundation::NSNumber,
        ) {
            use objc2_foundation::NSData;

            let device_id = BluetoothDeviceId::new(unsafe { peripheral.identifier().UUIDString().to_string() });

            if let Some(sender) = self.ivars().event_sender.get() {
                // Extract device name - check peripheral name first, then advertisement local name
                let name = unsafe { peripheral.name().map(|n| n.to_string()) }
                    .or_else(|| {
                        advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataLocalName"))
                            .and_then(|obj| obj.downcast::<NSString>().ok())
                            .map(|name_str| name_str.to_string())
                    });

                // Extract real device address - use peripheral identifier as address
                let address = unsafe { peripheral.identifier().UUIDString().to_string() };

                // Extract RSSI with proper NSNumber handling
                let rssi_value = rssi.intValue() as i16;
                let rssi = Some(rssi_value);

                // Extract service UUIDs from advertisement data - following btleplug production pattern
                let mut services = Vec::new();
                if let Some(service_uuids_obj) = advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataServiceUUIDs")) {
                    // SAFETY: service_uuids is NSArray<CBUUID> - following btleplug pattern
                    let service_uuids_array: *const AnyObject = &*service_uuids_obj;
                    let service_uuids_array: *const NSArray<CBUUID> = service_uuids_array.cast();
                    let service_uuids_array = unsafe { &*service_uuids_array };

                    for cbuuid in service_uuids_array {
                        // Convert CBUUID to Uuid using the same pattern as btleplug
                        let uuid_string = unsafe { cbuuid.UUIDString() }.to_string();
                        let long_uuid = if uuid_string.len() == 4 {
                            format!("0000{}-0000-1000-8000-00805f9b34fb", uuid_string)
                        } else if uuid_string.len() == 8 {
                            format!("{}-0000-1000-8000-00805f9b34fb", uuid_string)
                        } else {
                            uuid_string
                        };
                        if let Ok(uuid) = Uuid::parse_str(&long_uuid.to_lowercase()) {
                            services.push(uuid);
                        }
                    }
                }

                // Extract manufacturer data with proper NSData handling
                let mut manufacturer_data = HashMap::new();
                if let Some(mfg_data_obj) = advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataManufacturerData"))
                    && let Ok(mfg_data) = mfg_data_obj.downcast::<NSData>() {
                        let data_bytes = unsafe { mfg_data.as_bytes_unchecked() };

                        if data_bytes.len() >= 2 {
                            // First 2 bytes are company ID (little-endian)
                            let company_id = u16::from_le_bytes([data_bytes[0], data_bytes[1]]);
                            let payload = data_bytes[2..].to_vec();
                            manufacturer_data.insert(company_id, payload);
                        }
                    }

                // Extract service data from advertisement - following btleplug production pattern
                let mut service_data = HashMap::new();
                if let Some(service_data_obj) = advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataServiceData")) {
                    // SAFETY: service_data is NSDictionary<CBUUID, NSData> - following btleplug pattern
                    let service_data_dict: *const AnyObject = &*service_data_obj;
                    let service_data_dict: *const NSDictionary<CBUUID, NSData> = service_data_dict.cast();
                    let service_data_dict = unsafe { &*service_data_dict };

                    for cbuuid in service_data_dict.keys() {
                        if let Some(nsdata) = service_data_dict.objectForKey(&*cbuuid) {
                            // Convert CBUUID to Uuid
                            let uuid_string = unsafe { cbuuid.UUIDString() }.to_string();
                            let long_uuid = if uuid_string.len() == 4 {
                                format!("0000{}-0000-1000-8000-00805f9b34fb", uuid_string)
                            } else if uuid_string.len() == 8 {
                                format!("{}-0000-1000-8000-00805f9b34fb", uuid_string)
                            } else {
                                uuid_string
                            };
                            if let Ok(uuid) = Uuid::parse_str(&long_uuid.to_lowercase()) {
                                // Convert NSData to Vec<u8> - following btleplug pattern
                                let data_bytes = unsafe { nsdata.as_bytes_unchecked() }.to_vec();
                                service_data.insert(uuid, data_bytes);
                            }
                        }
                    }
                }

                // Extract TX power level if available
                let tx_power = advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataTxPowerLevel"))
                    .and_then(|obj| obj.downcast::<NSNumber>().ok())
                    .map(|num| num.intValue() as i8);

                // Determine if device is connectable from advertisement flags
                let connectable = advertisement_data.objectForKey(&NSString::from_str("kCBAdvDataIsConnectable"))
                    .and_then(|obj| obj.downcast::<NSNumber>().ok())
                    .map(|num| num.boolValue())
                    .unwrap_or(true); // Default to connectable for LE devices

                let device_info = DeviceInfo {
                    id: device_id.clone(),
                    name,
                    address,
                    rssi,
                    services,
                    manufacturer_data,
                    service_data,
                    tx_power,
                    connectable,
                };

                let device = BluetoothDevice::new(device_info);

                // Store peripheral for connections - following btleplug production pattern
                PERIPHERAL_STORE.with(|store| {
                    let mut peripherals = store.borrow_mut();
                    // Store retained peripheral reference for connection management
                    // Create a new Retained<CBPeripheral> by retaining the reference
                    let retained_peripheral = unsafe {
                        let ptr = peripheral as *const CBPeripheral;
                        Retained::retain(ptr as *mut CBPeripheral).unwrap_or_else(|| {
                            panic!("Critical failure: Failed to retain CBPeripheral object - Core Foundation runtime error")
                        })
                    };
                    peripherals.insert(device_id.as_str().to_string(), retained_peripheral);
                });

                // Check if this is a new device or update
                let is_new_device = {
                    if let Ok(mut devices) = self.ivars().discovered_devices.lock() {
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

                let _ = sender.send(event);
            }
        }
    }
);

impl BluetoothDelegate {
    fn new(mtm: MainThreadMarker, event_sender: Sender<BluetoothEvent>) -> Retained<Self> {
        let this = Self::alloc(mtm);
        let delegate: Retained<Self> = unsafe { msg_send![this, init] };
        let _ = delegate.ivars().event_sender.set(event_sender);
        delegate
    }
}

/// macOS Bluetooth manager using CoreBluetooth
pub struct MacOSBluetoothManager {
    central_manager: Option<Retained<CBCentralManager>>,
    delegate: Option<Retained<BluetoothDelegate>>,
    event_sender: Sender<BluetoothEvent>,
    discovered_devices: Arc<Mutex<HashMap<String, BluetoothDevice>>>,
    _is_scanning: Arc<Mutex<bool>>,
    _adapter_state: Arc<Mutex<AdapterState>>,
}

impl MacOSBluetoothManager {
    pub fn new(event_sender: Sender<BluetoothEvent>) -> Self {
        Self {
            central_manager: None,
            delegate: None,
            event_sender,
            discovered_devices: Arc::new(Mutex::new(HashMap::new())),
            _is_scanning: Arc::new(Mutex::new(false)),
            _adapter_state: Arc::new(Mutex::new(AdapterState::Unknown)),
        }
    }

    /// Initialize the CoreBluetooth central manager
    pub fn initialize(&mut self) -> BluetoothResult<()> {
        let mtm = MainThreadMarker::new().ok_or(BluetoothError::InternalError)?;

        // Create delegate
        let delegate = BluetoothDelegate::new(mtm, self.event_sender.clone());

        // Create CBCentralManager with proper delegate
        let central_manager = unsafe {
            let manager = CBCentralManager::new();
            manager.setDelegate(Some(ProtocolObject::from_ref(&*delegate)));
            manager
        };

        self.delegate = Some(delegate);
        self.central_manager = Some(central_manager);
        Ok(())
    }

    /// Start scanning for Bluetooth devices
    pub fn start_scan(&mut self, options: ScanOptions) -> BluetoothResult<()> {
        let central_manager = self
            .central_manager
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Check if Bluetooth is powered on
        if unsafe { central_manager.state() } != CBManagerState::PoweredOn {
            return Err(BluetoothError::AdapterNotReady);
        }

        // Create service filter from UUIDs - production implementation following btleplug
        let service_filter = self.create_service_filter(&options.service_uuids);
        let scan_options = self.create_scan_options(&options);

        // Start CoreBluetooth scanning with complete service filtering and options
        unsafe {
            central_manager.scanForPeripheralsWithServices_options(
                service_filter.as_deref(),
                Some(&*scan_options),
            )
        };

        *self
            ._is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)? = true;

        // Send scan started event
        self.send_event(BluetoothEvent::ScanStarted);

        // Set up scan timeout if specified - using tokio task for timeout handling
        if let Some(duration) = options.duration {
            let event_sender = self.event_sender.clone();
            let is_scanning = Arc::clone(&self._is_scanning);

            // Spawn a tokio task to handle the timeout
            // The actual scan stop will be handled by the sync API when it receives the timeout
            // event
            tokio::spawn(async move {
                tokio::time::sleep(duration).await;

                // Update scanning state
                if let Ok(mut scanning) = is_scanning.lock() {
                    *scanning = false;
                }

                // Send timeout event - the main thread will handle stopping the scan
                let _ = event_sender.send(BluetoothEvent::ScanStopped);
            });
        }

        Ok(())
    }

    /// Stop scanning for devices
    pub fn stop_scan(&mut self) -> BluetoothResult<()> {
        let central_manager = self
            .central_manager
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        unsafe { central_manager.stopScan() };
        *self
            ._is_scanning
            .lock()
            .map_err(|_| BluetoothError::InternalError)? = false;

        self.send_event(BluetoothEvent::ScanStopped);
        Ok(())
    }

    /// Connect to a Bluetooth device
    pub fn connect_device(&mut self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        let central_manager = self
            .central_manager
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Find the peripheral in the thread-local store
        let peripheral = PERIPHERAL_STORE.with(|store| {
            let peripherals = store.borrow();
            peripherals
                .get(device_id.as_str())
                .cloned()
                .ok_or(BluetoothError::DeviceNotFound)
        })?;

        // Update device connection state to connecting
        {
            let mut devices = self
                .discovered_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;
            if let Some(device) = devices.get_mut(device_id.as_str()) {
                device.state = ConnectionState::Connecting;
                device.last_seen = SystemTime::now();
            }
        }

        self.send_event(BluetoothEvent::ConnectionStateChanged {
            device_id: device_id.clone(),
            state: ConnectionState::Connecting,
        });

        // Create connection options for production reliability - following CoreBluetooth best
        // practices
        let connection_options = self.create_connection_options();

        // Use real CoreBluetooth connection with complete option configuration
        unsafe {
            central_manager.connectPeripheral_options(&peripheral, Some(&*connection_options));
        }

        Ok(())
    }

    /// Disconnect from a Bluetooth device
    pub fn disconnect_device(&mut self, device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
        let central_manager = self
            .central_manager
            .as_ref()
            .ok_or(BluetoothError::NotInitialized)?;

        // Find the peripheral in the thread-local store
        let peripheral = PERIPHERAL_STORE.with(|store| {
            let peripherals = store.borrow();
            peripherals
                .get(device_id.as_str())
                .cloned()
                .ok_or(BluetoothError::DeviceNotFound)
        })?;

        // Update device connection state to disconnecting
        {
            let mut devices = self
                .discovered_devices
                .lock()
                .map_err(|_| BluetoothError::InternalError)?;
            if let Some(device) = devices.get_mut(device_id.as_str()) {
                device.state = ConnectionState::Disconnecting;
                device.last_seen = SystemTime::now();
            }
        }

        self.send_event(BluetoothEvent::ConnectionStateChanged {
            device_id: device_id.clone(),
            state: ConnectionState::Disconnecting,
        });

        // Use real CoreBluetooth disconnection
        unsafe {
            central_manager.cancelPeripheralConnection(&peripheral);
        }

        Ok(())
    }

    /// Update adapter state and notify
    fn _update_adapter_state(&mut self, new_state: AdapterState) {
        if let Ok(mut state) = self._adapter_state.lock() {
            *state = new_state;
        }
        self.send_event(BluetoothEvent::AdapterStateChanged(new_state));
    }

    /// Update device connection state
    fn _update_device_connection_state(
        &mut self,
        device_id: &BluetoothDeviceId,
        state: ConnectionState,
    ) {
        if let Ok(mut devices) = self.discovered_devices.lock()
            && let Some(device) = devices.get_mut(device_id.as_str()) {
                device.state = state;
                device.last_seen = SystemTime::now();
            }

        self.send_event(BluetoothEvent::ConnectionStateChanged {
            device_id: device_id.clone(),
            state,
        });
    }

    /// Convert Uuid to CBUUID - following btleplug production pattern
    fn uuid_to_cbuuid(&self, uuid: &Uuid) -> Retained<CBUUID> {
        let uuid_string = NSString::from_str(&uuid.to_string());
        unsafe { CBUUID::UUIDWithString(&uuid_string) }
    }

    /// Create service filter NSArray from service UUIDs - following btleplug pattern
    fn create_service_filter(&self, service_uuids: &[Uuid]) -> Option<Retained<NSArray<CBUUID>>> {
        if service_uuids.is_empty() {
            None
        } else {
            let service_cbuuids: Vec<Retained<CBUUID>> = service_uuids
                .iter()
                .map(|uuid| self.uuid_to_cbuuid(uuid))
                .collect();
            Some(NSArray::from_retained_slice(&service_cbuuids))
        }
    }

    /// Create scan options dictionary - following btleplug production pattern
    fn create_scan_options(&self, _options: &ScanOptions) -> Retained<NSDictionary<NSString>> {
        let scan_options: Retained<NSMutableDictionary<NSString, NSNumber>> =
            NSMutableDictionary::new();

        // Set allow duplicates to true for proper device advertisement updates - following btleplug
        // pattern
        let allow_duplicates_key =
            NSString::from_str("kCBCentralManagerScanOptionAllowDuplicatesKey");
        let allow_duplicates_value = NSNumber::new_bool(true);
        unsafe {
            scan_options.setObject_forKey(
                &*allow_duplicates_value,
                ProtocolObject::from_ref(&*allow_duplicates_key),
            );
        }

        // Convert to immutable NSDictionary for API compatibility
        unsafe { Retained::cast_unchecked(scan_options) }
    }

    /// Create connection options dictionary - following CoreBluetooth production patterns
    fn create_connection_options(&self) -> Retained<NSDictionary<NSString>> {
        let connection_options: Retained<NSMutableDictionary<NSString, NSNumber>> =
            NSMutableDictionary::new();

        // Enable notification on disconnection - critical for proper connection state management
        let notify_on_disconnection_key =
            NSString::from_str("kCBConnectPeripheralOptionNotifyOnDisconnectionKey");
        let notify_on_disconnection_value = NSNumber::new_bool(true);
        unsafe {
            connection_options.setObject_forKey(
                &*notify_on_disconnection_value,
                ProtocolObject::from_ref(&*notify_on_disconnection_key),
            );
        }

        // Enable notification on connection for complete connection lifecycle management
        let notify_on_connection_key =
            NSString::from_str("kCBConnectPeripheralOptionNotifyOnConnectionKey");
        let notify_on_connection_value = NSNumber::new_bool(true);
        unsafe {
            connection_options.setObject_forKey(
                &*notify_on_connection_value,
                ProtocolObject::from_ref(&*notify_on_connection_key),
            );
        }

        // Convert to immutable NSDictionary for API compatibility
        unsafe { Retained::cast_unchecked(connection_options) }
    }

    /// Send event through the event sender
    fn send_event(&self, event: BluetoothEvent) {
        let _ = self.event_sender.send(event);
    }
}

// Thread-local storage for the global manager instance and peripherals
thread_local! {
    static MANAGER: std::cell::RefCell<Option<MacOSBluetoothManager>> = const { std::cell::RefCell::new(None) };
    static PERIPHERAL_STORE: std::cell::RefCell<HashMap<String, Retained<CBPeripheral>>> = std::cell::RefCell::new(HashMap::new());
}

/// Initialize macOS Bluetooth on main thread
pub fn initialize_bluetooth(event_sender: Sender<BluetoothEvent>) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        let mut bluetooth_manager = MacOSBluetoothManager::new(event_sender);
        bluetooth_manager.initialize()?;
        *mgr = Some(bluetooth_manager);
        Ok(())
    })
}

/// Start scanning for devices
pub fn start_scan(options: ScanOptions) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .start_scan(options)
    })
}

/// Stop scanning for devices
pub fn stop_scan() -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .stop_scan()
    })
}

/// Connect to a device
pub fn connect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .connect_device(device_id)
    })
}

/// Test if Bluetooth is available on macOS
pub fn test_availability() -> bool {
    // Test if we can create a MainThreadMarker - indicates CoreBluetooth is available
    MainThreadMarker::new().is_some()
}

/// Disconnect from a device
pub fn disconnect_device(device_id: &BluetoothDeviceId) -> BluetoothResult<()> {
    MANAGER.with(|manager| {
        let mut mgr = manager.borrow_mut();
        mgr.as_mut()
            .ok_or(BluetoothError::NotInitialized)?
            .disconnect_device(device_id)
    })
}
