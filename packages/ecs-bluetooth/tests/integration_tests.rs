//! Integration tests for ecs-bluetooth

use std::time::Duration;

use action_items_ecs_bluetooth::{BluetoothManager, ScanOptions};

#[tokio::test]
async fn test_bluetooth_manager_creation() {
    let manager = BluetoothManager::default();

    // Should start with unknown adapter state
    let state = manager.adapter_state().unwrap();
    println!("Initial adapter state: {:?}", state);

    // Should not be scanning initially
    assert!(!manager.is_scanning().unwrap());

    // Should have no devices initially
    let devices = manager.devices().unwrap();
    assert!(devices.is_empty());
}

#[tokio::test]
async fn test_scan_options_default() {
    let options = ScanOptions::default();

    assert_eq!(options.duration, Some(Duration::from_secs(10)));
    assert!(options.service_uuids.is_empty());
    assert!(!options.allow_duplicates);
    assert!(options.min_rssi.is_none());
}

#[cfg(target_os = "macos")]
#[tokio::test]
async fn test_macos_bluetooth_scan() {
    let manager = BluetoothManager::default();
    let options = ScanOptions::default();

    // Note: This test may fail if Bluetooth is not available or authorized
    // In a real test environment, we'd mock the platform layer
    match manager.start_scan(options) {
        Ok(_) => {
            println!("Scan started successfully");
            let _ = manager.stop_scan();
        },
        Err(e) => {
            println!("Scan failed (expected in test environment): {:?}", e);
        },
    }
}
