# ECS Bluetooth

Cross-platform Bluetooth Low Energy (BLE) operations for ECS applications.

## Features

- **Cross-platform support**: macOS (CoreBluetooth), Windows (WinRT), Linux (BlueZ)
- **Bevy ECS integration**: Resource-based manager with reactive event system
- **Device discovery**: Scan for nearby BLE devices with filtering options
- **Connection management**: Connect and disconnect from BLE devices
- **Event-driven**: Reactive updates for device state changes

## Usage

### Basic Setup

```rust
use bevy::prelude::*;
use action_items_ecs_bluetooth::{BluetoothPlugin, BluetoothManager, ScanOptions};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(BluetoothPlugin)
        .add_systems(Startup, setup_bluetooth)
        .run();
}

fn setup_bluetooth(manager: Res<BluetoothManager>) {
    // Start scanning for devices
    let options = ScanOptions::default();
    if let Err(e) = manager.start_scan(options) {
        eprintln!("Failed to start scan: {:?}", e);
    }
}
```

### Device Management

```rust
fn bluetooth_system(manager: Res<BluetoothManager>) {
    // Get all discovered devices
    if let Ok(devices) = manager.devices() {
        for device in devices {
            println!("Device: {:?} - {:?}", device.name(), device.state);
            
            // Connect to a specific device
            if device.name() == Some("MyDevice") && !device.is_connected() {
                if let Err(e) = manager.connect_device(device.id()) {
                    eprintln!("Failed to connect: {:?}", e);
                }
            }
        }
    }
}
```

## Platform Support

### macOS
- ✅ Device scanning with CoreBluetooth
- ✅ Device connection/disconnection
- ✅ Adapter state monitoring
- ✅ Full implementation

### Windows
- ✅ Complete implementation with Windows Runtime APIs
- ✅ Device scanning and connection management via BluetoothLEAdvertisementWatcher
- ✅ Advertisement monitoring and filtering with real async operations
- ✅ Proper connection/disconnection using BluetoothLEDevice.FromBluetoothAddressAsync
- ✅ Service UUID filtering and manufacturer data extraction
- Requires Windows 10+ with BLE support

### Linux
- ✅ Complete implementation with BlueZ D-Bus integration
- ✅ Device discovery and connection via zbus BlueZ proxies
- ✅ Real D-Bus signal handling for InterfacesAdded events
- ✅ Adapter management and property monitoring through BlueZAdapter1 proxy
- ✅ GATT service and characteristic support via D-Bus device paths
- Requires BlueZ 5.0+

## Requirements

- Rust 2021 edition
- Bevy 0.14+
- Platform-specific Bluetooth permissions

## License

MIT OR Apache-2.0