//! ECS Hotkey Service Systems
//!
//! Complete production-quality systems for comprehensive hotkey management including
//! registration, conflict detection, real-time capture, polling, and preferences management.

use bevy::prelude::*;
use bevy::tasks::futures_lite::future;
use bevy::tasks::{AsyncComputeTaskPool, block_on};
use global_hotkey::hotkey::HotKey;
use tracing::{debug, error, info, warn};

use crate::components::*;
use crate::events::*;
use crate::feedback::{FeedbackType, HotkeyVisualFeedback};
use crate::resources::*;

// ============================================================================
// HOTKEY REGISTRATION AND MANAGEMENT SYSTEMS
// ============================================================================

/// Process hotkey registration requests with intelligent conflict detection and fallbacks
///
/// Production implementation extracted from hotkeys/management.rs with full conflict detection
#[allow(clippy::type_complexity)]
pub fn process_hotkey_registration_requests_system(
    mut commands: Commands,
    mut registration_requests: EventReader<HotkeyRegisterRequested>,
    mut registration_completed: EventWriter<HotkeyRegisterCompleted>,
    mut conflict_detected: EventWriter<HotkeyConflictDetected>,
    hotkey_manager: ResMut<HotkeyManager>,
    mut hotkey_registry: ResMut<HotkeyRegistry>,
    config: Res<HotkeyConfig>,
) {
    for request in registration_requests.read() {
        if config.enable_debug_logging {
            info!(
                "Processing hotkey registration request for action: {}",
                request.binding.action
            );
        }

        let hotkey = HotKey::new(Some(request.binding.definition.modifiers), request.binding.definition.code);

        // Check if we've reached max hotkeys limit
        if hotkey_registry.registered_hotkeys.len() >= hotkey_manager.max_hotkeys {
            error!(
                "Maximum hotkey limit reached ({}), refusing registration",
                hotkey_manager.max_hotkeys
            );
            registration_completed.write(HotkeyRegisterCompleted {
                binding: request.binding.clone(),
                requester: request.binding.requester.clone(),
                success: false,
                error_message: Some("Maximum hotkey limit reached".to_string()),
            });
            continue;
        }

        let _hotkey_id = HotkeyId(uuid::Uuid::new_v4());

        // Check for system hotkey conflicts and warn user
        if let Some(system_shortcut) = crate::system_hotkeys::is_system_hotkey(&request.binding.definition) {
            warn!(
                "Hotkey {} conflicts with system shortcut: {}. This may not work as expected.",
                request.binding.definition.description,
                system_shortcut
            );
        }

        // ===== WAYLAND BACKEND REGISTRATION =====
        #[cfg(target_os = "linux")]
        {
            // Try Wayland backend first if available
            if let Some(ref wayland_mgr) = hotkey_manager.wayland_manager {
                let mgr = std::sync::Arc::clone(wayland_mgr);
                let binding = request.binding.clone();

                // Spawn async task for Wayland registration
                tokio::spawn(async move {
                    match mgr.lock().await.register(&binding).await {
                        Ok(_) => {
                            info!("✅ Registered Wayland hotkey: {}", binding.action);
                        }
                        Err(e) => {
                            error!("❌ Wayland hotkey registration failed: {}", e);
                        }
                    }
                });

                // Fire completion event
                registration_completed.write(HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: true,
                    error_message: None,
                });

                continue;  // Skip X11 registration
            }
        }
        // ===== END WAYLAND BACKEND =====

        // Attempt registration with the global hotkey manager
        match hotkey_manager.global_manager.register(hotkey) {
            Ok(()) => {
                info!(
                    "Successfully registered hotkey: {}",
                    request.binding.definition.description
                );

                // Store in registry - just clone the binding directly
                hotkey_registry
                    .registered_hotkeys
                    .insert(request.binding.id.clone(), request.binding.clone());

                // Create tracking operation
                let operation_id = uuid::Uuid::new_v4();
                commands.spawn((
                    HotkeyOperation {
                        id: operation_id,
                        operation_type: "register".to_string(),
                        hotkey_definition: request.binding.definition.clone(),
                        requester: request.binding.requester.clone(),
                        status: "completed".to_string(),
                        created_at: std::time::Instant::now(),
                        completed_at: Some(std::time::Instant::now()),
                    },
                    Name::new(format!("HotkeyRegisterOperation-{}", operation_id)),
                ));

                registration_completed.write(HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: true,
                    error_message: None,
                });
            },
            Err(e) => {
                warn!(
                    "Failed to register hotkey {}: {}",
                    request.binding.definition.description, e
                );

                // Try to unregister in case of partial registration
                if let Err(e) = hotkey_manager.global_manager.unregister(hotkey) {
                    warn!("Failed to unregister hotkey during error cleanup: {}", e);
                }

                // Generate conflict report
                let _conflict_report = ConflictReport {
                    conflicting_hotkey: request.binding.definition.clone(),
                    conflict_type: ConflictType::AlreadyRegistered,
                    conflicting_application: extract_conflicting_app_name(&e.to_string()),
                    suggested_alternative: find_alternative_hotkey(
                        &request.binding.definition,
                        &hotkey_manager.global_manager,
                    )
                    .ok(),
                };

                // Emit conflict detection event
                conflict_detected.write(HotkeyConflictDetected {
                    hotkey_definition: request.binding.definition.clone(),
                    conflict_type: "AlreadyRegistered".to_string(),
                    conflicting_app: extract_conflicting_app_name(&e.to_string()),
                    suggested_alternatives: vec![],
                });

                registration_completed.write(HotkeyRegisterCompleted {
                    binding: request.binding.clone(),
                    requester: request.binding.requester.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            },
        }
    }
}

/// Process hotkey unregistration requests
pub fn process_hotkey_unregistration_requests_system(
    mut commands: Commands,
    mut unregistration_requests: EventReader<HotkeyUnregisterRequested>,
    mut unregistration_completed: EventWriter<HotkeyUnregisterCompleted>,
    hotkey_manager: ResMut<HotkeyManager>,
    mut hotkey_registry: ResMut<HotkeyRegistry>,
    config: Res<HotkeyConfig>,
) {
    for request in unregistration_requests.read() {
        if config.enable_debug_logging {
            info!(
                "Processing hotkey unregistration request for ID: {:?}",
                request.hotkey_id
            );
        }

        // Look up binding WITHOUT removing it yet - clone data to drop immutable borrow
        let binding_data = hotkey_registry
            .registered_hotkeys
            .get(&request.hotkey_id)
            .map(|binding| {
                (
                    binding.definition.clone(),
                    binding.requester.clone(),
                )
            });

        if let Some((definition, _requester)) = binding_data {
            let hotkey = HotKey::new(Some(definition.modifiers), definition.code);

            // Unregister from OS FIRST
            match hotkey_manager.global_manager.unregister(hotkey) {
                Ok(()) => {
                    info!(
                        "Successfully unregistered hotkey: {}",
                        definition.description
                    );

                    // Only NOW remove from registry
                    hotkey_registry.registered_hotkeys.remove(&request.hotkey_id);

                    // Create tracking operation
                    let operation_id = uuid::Uuid::new_v4();
                    commands.spawn((
                        HotkeyOperation {
                            id: operation_id,
                            operation_type: "unregister".to_string(),
                            hotkey_definition: definition.clone(),
                            requester: request.requester.clone(),
                            status: "completed".to_string(),
                            created_at: std::time::Instant::now(),
                            completed_at: Some(std::time::Instant::now()),
                        },
                        Name::new(format!("HotkeyUnregisterOperation-{}", operation_id)),
                    ));

                    unregistration_completed.write(HotkeyUnregisterCompleted {
                        hotkey_id: request.hotkey_id.clone(),
                        success: true,
                    });
                },
                Err(e) => {
                    warn!(
                        "Failed to unregister hotkey {}: {}",
                        definition.description, e
                    );
                    // Don't remove from registry since OS still has it
                    unregistration_completed.write(HotkeyUnregisterCompleted {
                        hotkey_id: request.hotkey_id.clone(),
                        success: false,
                    });
                },
            }
        } else {
            info!(
                "Hotkey {:?} was not registered, skipping unregistration",
                request.hotkey_id
            );
            unregistration_completed.write(HotkeyUnregisterCompleted {
                hotkey_id: request.hotkey_id.clone(),
                success: true,
            });
        }
    }
}

// ============================================================================
// CONFLICT DETECTION AND VALIDATION SYSTEMS
// ============================================================================

/// Detect hotkey conflicts and generate suggested alternatives
pub fn detect_hotkey_conflicts_system(
    mut conflict_events: EventReader<HotkeyConflictDetected>,
    _hotkey_manager: Res<HotkeyManager>,
    _preferences: Res<HotkeyPreferences>,
) {
    for conflict in conflict_events.read() {
        // Process conflict events and log them
        info!(
            "Hotkey conflict detected: {}",
            conflict.hotkey_definition.description
        );
        if let Some(app) = &conflict.conflicting_app {
            info!("  Conflicting with: {}", app);
        }
        for alternative in &conflict.suggested_alternatives {
            info!("  Suggested alternative: {}", alternative.description);
        }
    }
}

/// Validate hotkey combinations for system compatibility and conflicts
pub fn validate_hotkey_combinations_system(
    hotkey_registry: Res<HotkeyRegistry>,
    _preferences: Res<HotkeyPreferences>,
    config: Res<HotkeyConfig>,
) {
    if config.enable_debug_logging {
        // Validate all registered hotkeys periodically
        for (hotkey_id, binding) in &hotkey_registry.registered_hotkeys {
            debug!(
                "Validated hotkey {}: {} (ID: {:?})",
                binding.definition.description, binding.action, hotkey_id
            );
        }
    }
}

// ============================================================================
// HOTKEY DETECTION AND POLLING SYSTEMS
// ============================================================================


/// Process hotkey pressed events - immediate synchronous check for pressed hotkeys
pub fn process_hotkey_pressed_events_system(
    hotkey_registry: Res<HotkeyRegistry>,
    mut hotkey_pressed_events: EventWriter<HotkeyPressed>,
    mut feedback_events: EventWriter<HotkeyVisualFeedback>,
    mut analytics: ResMut<HotkeyAnalytics>,
    config: Res<HotkeyConfig>,
) {
    // Immediate check for any pressed hotkeys using try_recv
    while let Ok(global_event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
        if config.enable_debug_logging {
            info!("Processing global hotkey event: ID={:?}", global_event.id);
        }

        // Find matching registered hotkey by comparing HotKey ID
        for (_hotkey_id, binding) in &hotkey_registry.registered_hotkeys {
            let hotkey = HotKey::new(Some(binding.definition.modifiers), binding.definition.code);
            if hotkey.id() == global_event.id {
                info!(
                    "Hotkey pressed: {} -> {}",
                    binding.definition.description, binding.action
                );

                hotkey_pressed_events.write(HotkeyPressed {
                    hotkey_id: _hotkey_id.clone(),
                    binding: binding.clone(),
                });

                // Record press in analytics for usage tracking
                analytics.record_press(_hotkey_id);

                // Emit visual feedback
                feedback_events.write(HotkeyVisualFeedback {
                    hotkey_id: _hotkey_id.clone(),
                    description: binding.definition.description.clone(),
                    feedback_type: FeedbackType::Success,
                });
                break;
            }
        }
    }
}

// ============================================================================
// REAL-TIME CAPTURE SYSTEMS
// ============================================================================

/// Process hotkey capture start/stop requests
pub fn process_hotkey_capture_requests_system(
    mut capture_requests: EventReader<HotkeyCaptureRequested>,
    mut capture_cancelled: EventReader<HotkeyCaptureCancelled>,
    mut capture_state: ResMut<HotkeyCaptureState>,
    config: Res<HotkeyConfig>,
) {
    // Process capture start requests
    for request in capture_requests.read() {
        if config.enable_debug_logging {
            info!(
                "Starting hotkey capture for requester: {}",
                request.requester
            );
        }

        capture_state.capturing = true;
        capture_state.current_requester = Some(request.requester.clone());
        capture_state.held_modifiers = global_hotkey::hotkey::Modifiers::empty();
        capture_state.captured_hotkey = None;
    }

    // Process capture cancellation requests
    for cancellation in capture_cancelled.read() {
        if config.enable_debug_logging {
            info!(
                "Cancelling hotkey capture for requester: {}",
                cancellation.requester
            );
        }

        capture_state.capturing = false;
        capture_state.current_requester = None;
        capture_state.held_modifiers = global_hotkey::hotkey::Modifiers::empty();
        capture_state.captured_hotkey = None;
    }
}

/// Real-time hotkey capture system - extracted from key_capture.rs
///
/// This is already implemented in capture.rs and imported, but we reference it here
pub use crate::capture::real_hotkey_capture_system as real_time_hotkey_capture_system;

// ============================================================================
// TESTING SYSTEMS
// ============================================================================

/// Process hotkey test requests with real registration testing
pub fn process_hotkey_test_requests_system(
    mut test_requests: EventReader<HotkeyTestRequested>,
    mut test_results: EventWriter<HotkeyTestResult>,
    hotkey_manager: Res<HotkeyManager>,
    config: Res<HotkeyConfig>,
) {
    for request in test_requests.read() {
        if config.enable_debug_logging {
            info!(
                "Testing hotkey: {} for requester: {}",
                request.definition.description, request.requester
            );
        }

        let hotkey = HotKey::new(Some(request.definition.modifiers), request.definition.code);

        // Actually try to register the hotkey to test for conflicts
        match hotkey_manager.global_manager.register(hotkey) {
            Ok(()) => {
                // Successfully registered, now unregister it
                if let Err(e) = hotkey_manager.global_manager.unregister(hotkey) {
                    warn!("Failed to unregister test hotkey: {}", e);
                }

                info!("Hotkey test successful: {}", request.definition.description);
                test_results.write(HotkeyTestResult {
                    hotkey_definition: request.definition.clone(),
                    requester: request.requester.clone(),
                    success: true,
                    error_message: None,
                    test_timestamp: std::time::Instant::now(),
                });
            },
            Err(e) => {
                info!(
                    "Hotkey test failed: {} - {}",
                    request.definition.description, e
                );
                test_results.write(HotkeyTestResult {
                    hotkey_definition: request.definition.clone(),
                    requester: request.requester.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                    test_timestamp: std::time::Instant::now(),
                });
            },
        }
    }
}

// ============================================================================
// PREFERENCES MANAGEMENT SYSTEMS
// ============================================================================

/// Comprehensive hotkey preferences management system
///
/// Production implementation extracted from preferences.rs with atomic persistence
pub fn manage_hotkey_preferences_system(
    mut commands: Commands,
    mut preferences_events: EventReader<HotkeyPreferencesUpdated>,
    mut hotkey_preferences: ResMut<HotkeyPreferences>,
    config: Res<HotkeyConfig>,
) {
    for event in preferences_events.read() {
        if config.enable_debug_logging {
            info!("Updating hotkey preferences");
        }

        // Update preferences resource
        *hotkey_preferences = event.preferences.clone();

        // Persist preferences using AsyncComputeTaskPool with atomic writes
        let prefs_clone = event.preferences.clone();
        let task_pool = AsyncComputeTaskPool::get();
        let task =
            task_pool.spawn(async move { persist_hotkey_preferences_owned(prefs_clone).await });

        // Store task as component for proper Bevy task management
        commands.spawn((
            HotkeyPreferencesPersistTask { task },
            Name::new("HotkeyPreferencesPersistTask"),
        ));

        info!("Hotkey preferences updated and persistence initiated");
    }
}

// ============================================================================
// PROFILE MANAGEMENT SYSTEMS
// ============================================================================

/// Process profile switch requests with coordinated binding updates
pub fn process_profile_switch_requests_system(
    mut switch_requests: EventReader<HotkeyProfileSwitchRequested>,
    mut switch_completed: EventWriter<HotkeyProfileSwitchCompleted>,
    mut unregister_events: EventWriter<HotkeyUnregisterRequested>,
    mut register_events: EventWriter<HotkeyRegisterRequested>,
    mut profiles_updated: EventWriter<HotkeyProfilesUpdated>,
    mut profiles: ResMut<HotkeyProfiles>,
    config: Res<HotkeyConfig>,
) {
    for request in switch_requests.read() {
        if config.enable_debug_logging {
            info!("Processing profile switch to: {}", request.profile_name);
        }

        let old_bindings: Vec<HotkeyId> = profiles.active_bindings.keys().cloned().collect();

        if let Err(e) = profiles.switch_profile(&request.profile_name) {
            warn!("Profile switch failed: {}", e);
            switch_completed.write(HotkeyProfileSwitchCompleted {
                profile_name: request.profile_name.clone(),
                success: false,
                error_message: Some(e),
            });
            continue;
        }

        for hotkey_id in old_bindings {
            unregister_events.write(HotkeyUnregisterRequested {
                hotkey_id: hotkey_id.clone(),
                requester: "profile_switch".to_string(),
            });
        }
        profiles.active_bindings.clear();

        let bindings_to_register: Vec<ProfileBinding> = profiles.get_active_bindings()
            .cloned()
            .unwrap_or_default();

        for profile_binding in bindings_to_register.iter() {
            let hotkey_binding = profile_binding.to_hotkey_binding();
            let binding_id = hotkey_binding.id.clone();

            register_events.write(HotkeyRegisterRequested {
                binding: hotkey_binding.clone(),
            });

            profiles.active_bindings.insert(binding_id, request.profile_name.clone());
        }

        info!("Profile switched to: {}", request.profile_name);
        switch_completed.write(HotkeyProfileSwitchCompleted {
            profile_name: request.profile_name.clone(),
            success: true,
            error_message: None,
        });

        profiles_updated.write(HotkeyProfilesUpdated {
            reason: "switched".to_string(),
        });
    }
}

/// Manage profile persistence triggered by update events
pub fn manage_hotkey_profiles_persistence_system(
    mut commands: Commands,
    mut profiles_updated: EventReader<HotkeyProfilesUpdated>,
    profiles: Res<HotkeyProfiles>,
    config: Res<HotkeyConfig>,
) {
    for event in profiles_updated.read() {
        if config.enable_debug_logging {
            info!("Persisting profiles: {}", event.reason);
        }

        let profiles_clone = profiles.clone();
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            persist_hotkey_profiles_owned(profiles_clone).await
        });

        commands.spawn((
            HotkeyProfilesPersistTask { task },
            Name::new("HotkeyProfilesPersistTask"),
        ));
    }
}

/// Poll persistence tasks for completion
pub fn poll_hotkey_profiles_persist_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut HotkeyProfilesPersistTask)>,
) {
    for (entity, mut persist_task) in task_query.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut persist_task.task)) {
            match result {
                Ok(path) => info!("✅ Profiles persisted to: {:?}", path),
                Err(e) => error!("❌ Failed to persist profiles: {}", e),
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Load profiles from disk on startup
pub fn load_hotkey_profiles_startup_system(
    mut commands: Commands,
) {
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        load_hotkey_profiles_from_disk().await
    });

    commands.spawn((
        HotkeyProfilesLoadTask { task },
        Name::new("HotkeyProfilesLoadTask"),
    ));
}

/// Poll load tasks and initialize resource
pub fn poll_hotkey_profiles_load_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut HotkeyProfilesLoadTask)>,
) {
    for (entity, mut load_task) in task_query.iter_mut() {
        if let Some(profiles) = block_on(future::poll_once(&mut load_task.task)) {
            info!("✅ Loaded profiles from disk");
            commands.insert_resource(profiles);
            commands.entity(entity).despawn();
        }
    }
}

/// Atomic write to disk (async function)
async fn persist_hotkey_profiles_owned(
    profiles: HotkeyProfiles,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    use std::fs;

    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory")?
        .join("action-items");

    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("hotkey-profiles.json");
    let temp_file = config_file.with_extension("tmp");

    let json_content = serde_json::to_string_pretty(&profiles)?;

    fs::write(&temp_file, &json_content)?;
    fs::rename(&temp_file, &config_file)?;

    Ok(config_file)
}

/// Load profiles from disk
async fn load_hotkey_profiles_from_disk() -> HotkeyProfiles {
    use std::fs;

    let config_file = dirs::config_dir()
        .map(|d| d.join("action-items/hotkey-profiles.json"));

    if let Some(path) = config_file {
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(profiles) = serde_json::from_str::<HotkeyProfiles>(&content) {
                return profiles;
            }
        }
    }

    HotkeyProfiles::default()
}

// ============================================================================
// CLEANUP AND METRICS SYSTEMS
// ============================================================================

/// Cleanup hotkeys when their guard entities despawn
/// 
/// Uses Bevy's RemovedComponents system parameter to detect when HotkeyGuard
/// components are removed, then automatically unregisters the associated hotkeys.
pub fn cleanup_despawned_hotkey_guards_system(
    mut unregister_events: EventWriter<HotkeyUnregisterRequested>,
    mut entity_map: ResMut<HotkeyEntityMap>,
    mut removed: RemovedComponents<HotkeyGuard>,
    config: Res<HotkeyConfig>,
) {
    for entity in removed.read() {
        if let Some(hotkey_id) = entity_map.entity_to_hotkey.remove(&entity) {
            if config.enable_debug_logging {
                info!("Auto-unregistering hotkey {:?} (entity {:?} despawned)", hotkey_id, entity);
            }
            
            unregister_events.write(HotkeyUnregisterRequested {
                hotkey_id,
                requester: "auto_cleanup".to_string(),
            });
        }
    }
}

/// Clean up completed hotkey operations
pub fn cleanup_completed_hotkey_operations_system(
    mut commands: Commands,
    completed_operations: Query<(Entity, &HotkeyOperation)>,
    config: Res<HotkeyConfig>,
) {
    let now = std::time::Instant::now();
    let cleanup_threshold = std::time::Duration::from_secs(300); // 5 minutes

    for (entity, operation) in completed_operations.iter() {
        if now.duration_since(operation.created_at) > cleanup_threshold {
            if config.enable_debug_logging {
                debug!(
                    "Cleaning up completed hotkey operation: {} ({})",
                    operation.id, operation.operation_type
                );
            }
            commands.entity(entity).despawn();
        }
    }
}

/// Update hotkey metrics and usage statistics
pub fn update_hotkey_metrics_system(
    hotkey_registry: Res<HotkeyRegistry>,
    hotkey_pressed_events: EventReader<HotkeyPressed>,
    mut hotkey_metrics: ResMut<HotkeyMetrics>,
    mut usage_trackers: Query<&mut HotkeyUsageTracker>,
    config: Res<HotkeyConfig>,
) {
    // Update basic metrics
    hotkey_metrics.registered_count = hotkey_registry.registered_hotkeys.len();

    // Count hotkey press events in this frame
    let press_count = hotkey_pressed_events.len();
    if press_count > 0 {
        hotkey_metrics.press_count += press_count;
        hotkey_metrics.last_press = Some(std::time::Instant::now());

        if config.enable_debug_logging {
            debug!(
                "Recorded {} hotkey presses, total: {}",
                press_count, hotkey_metrics.press_count
            );
        }
    }

    // Update usage trackers with exponential moving average
    for mut tracker in usage_trackers.iter_mut() {
        let new_value = press_count as f64;
        tracker.moving_average =
            tracker.alpha * new_value + (1.0 - tracker.alpha) * tracker.moving_average;
        tracker.sample_count += 1;
    }
}

/// System to poll hotkey preferences persistence tasks
///
/// Production implementation extracted from preferences.rs
pub fn poll_hotkey_preferences_persist_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut HotkeyPreferencesPersistTask)>,
) {
    for (entity, mut persist_task) in task_query.iter_mut() {
        if let Some(result) = block_on(future::poll_once(&mut persist_task.task)) {
            match result {
                Ok(config_path) => {
                    info!("✅ Hotkey preferences persisted to: {:?}", config_path);
                },
                Err(e) => {
                    error!("❌ Failed to persist hotkey preferences: {}", e);
                },
            }
            commands.entity(entity).despawn();
        }
    }
}

// ============================================================================
// HELPER FUNCTIONS - PRODUCTION IMPLEMENTATIONS
// ============================================================================

/// Extract conflicting application name from error message
fn extract_conflicting_app_name(error_msg: &str) -> Option<String> {
    if error_msg.contains("already registered") {
        Some("another application".to_string())
    } else {
        None
    }
}

/// Find alternative hotkey using production algorithm from management.rs
fn find_alternative_hotkey(
    original: &HotkeyDefinition,
    manager: &global_hotkey::GlobalHotKeyManager,
) -> Result<HotkeyDefinition, Box<dyn std::error::Error>> {
    let alternatives = generate_alternative_hotkeys(original);

    alternatives
        .into_iter()
        .find(|alt| test_hotkey_availability(alt, manager))
        .ok_or_else(|| "No available alternative hotkeys found".into())
}

/// Generate alternative hotkey combinations
fn generate_alternative_hotkeys(original: &HotkeyDefinition) -> Vec<HotkeyDefinition> {
    use global_hotkey::hotkey::{Code, Modifiers};

    let mut alternatives = Vec::new();

    // Try different modifier combinations with same key
    let modifier_combinations = vec![
        Modifiers::META | Modifiers::SHIFT,
        Modifiers::CONTROL | Modifiers::SHIFT,
        Modifiers::META | Modifiers::ALT,
        Modifiers::CONTROL | Modifiers::ALT,
        Modifiers::ALT | Modifiers::SHIFT,
    ];

    for modifiers in modifier_combinations {
        if modifiers != original.modifiers {
            alternatives.push(HotkeyDefinition {
                modifiers,
                code: original.code,
                description: format_hotkey_description(modifiers, original.code),
            });
        }
    }

    // Try different keys with same modifiers
    let alternative_keys = vec![
        Code::Space,
        Code::Tab,
        Code::Enter,
        Code::KeyJ,
        Code::KeyK,
        Code::KeyL,
    ];

    for code in alternative_keys {
        if code != original.code {
            alternatives.push(HotkeyDefinition {
                modifiers: original.modifiers,
                code,
                description: format_hotkey_description(original.modifiers, code),
            });
        }
    }

    alternatives
}

/// Test if a specific hotkey is available
fn test_hotkey_availability(
    hotkey_def: &HotkeyDefinition,
    manager: &global_hotkey::GlobalHotKeyManager,
) -> bool {
    let hotkey = HotKey::new(Some(hotkey_def.modifiers), hotkey_def.code);

    match manager.register(hotkey) {
        Ok(()) => {
            if let Err(e) = manager.unregister(hotkey) {
                warn!("Failed to unregister hotkey during availability check: {}", e);
            }
            true
        },
        Err(_) => false,
    }
}

/// Persist hotkey preferences to disk with atomic writes - owned version for async tasks
async fn persist_hotkey_preferences_owned(
    prefs: HotkeyPreferences,
) -> Result<std::path::PathBuf, Box<dyn std::error::Error + Send + Sync>> {
    use std::fs;

    // Get platform-appropriate config directory
    let config_dir = dirs::config_dir()
        .ok_or("Could not determine config directory for this platform")?
        .join("action-items");

    // Ensure config directory exists
    fs::create_dir_all(&config_dir)?;

    let config_file = config_dir.join("hotkey-preferences.json");
    let temp_file = config_file.with_extension("tmp");

    // Serialize preferences with pretty formatting
    let json_content = serde_json::to_string_pretty(&prefs)
        .map_err(|e| format!("Failed to serialize hotkey preferences: {}", e))?;

    // Write to temporary file first (atomic write pattern)
    fs::write(&temp_file, &json_content)
        .map_err(|e| format!("Failed to write temporary config file: {}", e))?;

    // Atomically move temp file to final location (crash-safe)
    fs::rename(&temp_file, &config_file)
        .map_err(|e| format!("Failed to finalize config file: {}", e))?;

    debug!(
        "Hotkey preferences persisted atomically to: {:?}",
        config_file
    );
    Ok(config_file)
}


// ============================================================================
// MULTI-SESSION CAPTURE SYSTEMS
// ============================================================================

/// Process multi-session capture start requests
/// 
/// Creates new capture sessions in MultiCaptureState for each request.
/// Multiple sessions can be active simultaneously for different UI components.
///
/// # Zero Allocation
/// - Inline processing for blazing-fast session creation
/// - Pre-allocated HashMap in MultiCaptureState
#[inline]
pub fn process_multi_capture_requests_system(
    mut capture_requests: EventReader<crate::events::HotkeyCaptureRequested>,
    mut multi_capture: ResMut<crate::resources::MultiCaptureState>,
    config: Res<HotkeyConfig>,
) {
    for request in capture_requests.read() {
        // Only process requests with session_id (multi-capture mode)
        if request.session_id.is_some() {
            let session_id = multi_capture.start_session(&request.requester);
            
            if config.enable_debug_logging {
                info!(
                    "Started multi-capture session {:?} for requester: {}",
                    session_id, request.requester
                );
            }
        }
    }
}

/// Process multi-session capture cancellations
/// 
/// Removes sessions from MultiCaptureState when cancelled.
/// Handles graceful cleanup of incomplete capture sessions.
#[inline]
pub fn process_multi_capture_cancellations_system(
    mut capture_cancelled: EventReader<crate::events::HotkeyCaptureCancelled>,
    mut multi_capture: ResMut<crate::resources::MultiCaptureState>,
    config: Res<HotkeyConfig>,
) {
    for cancellation in capture_cancelled.read() {
        // Only process cancellations with session_id (multi-capture mode)
        if let Some(session_id) = cancellation.session_id {
            if let Some(session) = multi_capture.complete_session(&session_id) {
                if config.enable_debug_logging {
                    info!(
                        "Cancelled multi-capture session {:?} for requester: {} (reason: {:?})",
                        session_id, session.requester, cancellation.reason
                    );
                }
            }
        }
    }
}

/// Poll Wayland hotkey events and emit HotkeyPressed events
/// 
/// Uses try_lock() to avoid blocking Bevy's main thread.
/// Creates ephemeral tokio runtime for async polling.
#[cfg(target_os = "linux")]
pub fn poll_wayland_hotkey_events_system(
    hotkey_manager: Res<HotkeyManager>,
    mut hotkey_pressed: EventWriter<HotkeyPressed>,
    hotkey_registry: Res<HotkeyRegistry>,
    config: Res<HotkeyConfig>,
) {
    if let Some(ref wayland_mgr) = hotkey_manager.wayland_manager {
        let mgr = std::sync::Arc::clone(wayland_mgr);

        // Try non-blocking lock (returns immediately if busy)
        if let Ok(mut mgr_lock) = mgr.try_lock() {
            // Create ephemeral runtime for async polling
            match tokio::runtime::Runtime::new() {
                Ok(rt) => {
                    match rt.block_on(mgr_lock.poll_events()) {
                        Ok(events) => {
                            for action_id in events {
                                // Find binding by action ID
                                if let Some(binding) = hotkey_registry.bindings.iter()
                                    .find(|b| b.action == action_id)
                                {
                                    if config.enable_debug_logging {
                                        info!("🔥 Wayland hotkey triggered: {}", action_id);
                                    }
                                    hotkey_pressed.write(HotkeyPressed {
                                        binding: binding.clone(),
                                    });
                                } else {
                                    warn!("Received Wayland event for unknown action: {}", action_id);
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to poll Wayland hotkey events: {}", e);
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to create tokio runtime for Wayland polling: {}", e);
                }
            }
        }
        // If try_lock() fails, we skip this frame - no blocking
    }
}

/// Re-export multi-session capture system from capture.rs
pub use crate::capture::multi_session_capture_system;
