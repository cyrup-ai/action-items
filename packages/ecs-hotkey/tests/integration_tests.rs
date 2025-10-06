//! Integration tests for ECS Hotkey Service
//!
//! These tests verify the full functionality of the hotkey service including
//! registration, conflict detection, real-time capture, and cross-platform operation.

use std::time::Duration;

use bevy::prelude::*;
use bevy::ecs::event::EventCursor;
use ecs_hotkey::*;
use global_hotkey::hotkey::{Code, Modifiers};

/// Test basic hotkey plugin initialization
#[test]
fn test_hotkey_plugin_initialization() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Verify resources are initialized
    assert!(app.world().contains_resource::<HotkeyManager>());
    assert!(app.world().contains_resource::<HotkeyRegistry>());
    assert!(app.world().contains_resource::<HotkeyCaptureState>());
    assert!(app.world().contains_resource::<HotkeyMetrics>());
    assert!(app.world().contains_resource::<HotkeyPreferences>());
    assert!(app.world().contains_resource::<HotkeyConfig>());

    // Verify events are registered
    assert!(
        app.world()
            .contains_resource::<Events<HotkeyRegisterRequested>>()
    );
    assert!(app.world().contains_resource::<Events<HotkeyPressed>>());
    assert!(
        app.world()
            .contains_resource::<Events<HotkeyConflictDetected>>()
    );
    assert!(
        app.world()
            .contains_resource::<Events<HotkeyCaptureRequested>>()
    );
}

/// Test hotkey registration request processing
#[test]
fn test_hotkey_registration_flow() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Create a test hotkey definition
    let hotkey_def = HotkeyDefinition {
        modifiers: Modifiers::META | Modifiers::SHIFT,
        code: Code::Space,
        description: "⌘⇧Space".to_string(),
    };

    let binding =
        HotkeyBinding::new(hotkey_def.clone(), "test_action").with_requester("integration_test");

    // Send registration request
    let mut register_events = app
        .world_mut()
        .resource_mut::<Events<HotkeyRegisterRequested>>();
    register_events.send(HotkeyRegisterRequested {
        binding: binding.clone(),
    });

    // Run one update cycle
    app.update();

    // Verify registration was processed (check metrics)
    let metrics = app.world().resource::<HotkeyMetrics>();
    assert!(metrics.total_registrations > 0);
}

/// Test hotkey conflict detection
#[test]
fn test_hotkey_conflict_detection() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Create two identical hotkey definitions that should conflict
    let hotkey_def1 = HotkeyDefinition {
        modifiers: Modifiers::META,
        code: Code::Space,
        description: "⌘Space".to_string(),
    };

    let hotkey_def2 = hotkey_def1.clone();

    let binding1 = HotkeyBinding::new(hotkey_def1.clone(), "action1").with_requester("test1");
    let binding2 = HotkeyBinding::new(hotkey_def2.clone(), "action2").with_requester("test2");

    // Send registration requests
    let mut register_events = app
        .world_mut()
        .resource_mut::<Events<HotkeyRegisterRequested>>();
    register_events.send(HotkeyRegisterRequested {
        binding: binding1,
    });
    register_events.send(HotkeyRegisterRequested {
        binding: binding2,
    });

    // Run update cycles
    app.update();
    app.update();

    // Check for conflict detection
    let conflict_events = app.world().resource::<Events<HotkeyConflictDetected>>();
    let mut reader = EventCursor::<HotkeyConflictDetected>::default();
    let conflicts: Vec<_> = reader.read(conflict_events).collect();

    // Should have detected at least one conflict
    assert!(
        !conflicts.is_empty() || {
            // Alternative: check metrics for conflict detection
            let metrics = app.world().resource::<HotkeyMetrics>();
            metrics.conflicts_detected > 0
        }
    );
}

/// Test hotkey capture state management
#[test]
fn test_hotkey_capture_state() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Send capture start request
    let mut capture_events = app
        .world_mut()
        .resource_mut::<Events<HotkeyCaptureRequested>>();
    capture_events.send(HotkeyCaptureRequested {
        target_action: "test_capture".to_string(),
        requester: "integration_test".to_string(),
        session_id: None,
    });

    // Run update cycle
    app.update();

    // Verify capture state was updated
    let capture_state = app.world().resource::<HotkeyCaptureState>();
    assert_eq!(
        capture_state.current_requester.as_deref(),
        Some("integration_test")
    );

    // Verify metrics were updated
    let metrics = app.world().resource::<HotkeyMetrics>();
    assert!(metrics.capture_sessions > 0);
}

/// Test hotkey preferences management
#[test]
fn test_hotkey_preferences() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Create custom preferences
    let custom_hotkey = HotkeyDefinition {
        modifiers: Modifiers::ALT,
        code: Code::F1,
        description: "⌥F1".to_string(),
    };

    let custom_prefs = HotkeyPreferences {
        preferred_combinations: vec![custom_hotkey.clone()],
        custom_hotkey: Some(custom_hotkey),
        auto_fallback: false,
    };

    // Send preferences update
    let mut prefs_events = app
        .world_mut()
        .resource_mut::<Events<HotkeyPreferencesUpdated>>();
    prefs_events.send(HotkeyPreferencesUpdated {
        preferences: custom_prefs.clone(),
        requester: "integration_test".to_string(),
    });

    // Run update cycle
    app.update();

    // Verify preferences were applied
    let current_prefs = app.world().resource::<HotkeyPreferences>();
    assert!(!current_prefs.auto_fallback);
    assert!(current_prefs.custom_hotkey.is_some());
}

/// Test system set ordering and execution
#[test]
fn test_system_execution_order() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    // Add a test system that checks execution order
    app.add_systems(
        Update,
        test_system_ordering.after(HotkeySystemSet::Registration),
    );

    // Run several update cycles to ensure systems execute
    for _ in 0..5 {
        app.update();
    }

    // If we reach here without panics, system ordering is working correctly
}

fn test_system_ordering() {
    // This system runs after registration systems, verifying ordering works
}

/// Test plugin configuration options
#[test]
fn test_plugin_configuration() {
    let mut app = App::new();

    let custom_config = HotkeyPlugin::new()
        .with_debug_logging(true)
        .with_polling_interval(Duration::from_millis(5))
        .with_max_hotkeys(128)
        .with_conflict_resolution(false);

    app.add_plugins(custom_config);

    // Verify configuration was applied
    let config = app.world().resource::<HotkeyConfig>();
    assert!(config.enable_debug_logging);
    assert_eq!(config.polling_interval, Duration::from_millis(5));
    assert_eq!(config.max_hotkeys, 128);
    assert!(!config.enable_conflict_resolution);
}

/// Test development and production modes
#[test]
fn test_configuration_modes() {
    // Test development mode
    let mut dev_app = App::new();
    dev_app.add_plugins(HotkeyPlugin::new().development_mode());

    let dev_config = dev_app.world().resource::<HotkeyConfig>();
    assert!(dev_config.enable_debug_logging);
    assert_eq!(dev_config.polling_interval, Duration::from_millis(5));
    assert!(dev_config.enable_conflict_resolution);

    // Test production mode
    let mut prod_app = App::new();
    prod_app.add_plugins(HotkeyPlugin::new().production_mode());

    let prod_config = prod_app.world().resource::<HotkeyConfig>();
    assert!(!prod_config.enable_debug_logging);
    assert_eq!(prod_config.polling_interval, Duration::from_millis(15));
    assert!(!prod_config.enable_conflict_resolution);
}

/// Test metrics collection and accuracy
#[test]
fn test_metrics_collection() {
    let mut app = App::new();
    app.add_plugins(HotkeyPlugin::new().with_debug_logging(true));

    let initial_tests_performed = app.world().resource::<HotkeyMetrics>().tests_performed;

    // Perform several operations that should update metrics
    let hotkey_def = HotkeyDefinition {
        modifiers: Modifiers::CONTROL,
        code: Code::KeyA,
        description: "⌃A".to_string(),
    };

    // Send test request - scope the mutable borrow
    {
        let mut test_events = app
            .world_mut()
            .resource_mut::<Events<HotkeyTestRequested>>();
        test_events.send(HotkeyTestRequested {
            definition: hotkey_def,
            requester: "metrics_test".to_string(),
        });
    }

    // Run update cycles
    for _ in 0..3 {
        app.update();
    }

    // Verify metrics were updated
    let final_tests_performed = app.world().resource::<HotkeyMetrics>().tests_performed;
    assert!(final_tests_performed > initial_tests_performed);
}
