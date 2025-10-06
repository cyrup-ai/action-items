use bevy::prelude::*;
use bevy::tasks::{AsyncComputeTaskPool, Task, block_on, poll_once};
use bevy::winit::EventLoopProxyWrapper;
use ecs_hotkey::{HotkeyManager, HotkeyPreferences};
use tracing::{info, warn, error};
use action_items_ecs_permissions::{PermissionRequest, PermissionType, PermissionStatus, PermissionResource};

use crate::events::GlobalHotkeyEvent;

/// Component to track hotkey event polling task
#[derive(Component)]
#[allow(dead_code)]
pub struct HotkeyPollingTask {
    pub task: Task<()>,
}

/// Marker component for the hotkey manager entity
#[derive(Component)]
pub struct HotkeyManagerEntity;

/// Setup global hotkey callback using proper Bevy EventLoopProxyWrapper pattern
/// Follows custom_user_event.rs example for external event integration
pub fn setup_global_hotkey_callback(
    mut commands: Commands,
    event_loop_proxy: Res<EventLoopProxyWrapper<GlobalHotkeyEvent>>,
    mut hotkey_manager: ResMut<HotkeyManager>,
    hotkey_prefs: Res<HotkeyPreferences>,
    permission_resource: Res<PermissionResource>,
    _permission_requests: EventWriter<PermissionRequest>,
) {
    info!("Setting up component-based global hotkey callback with AsyncComputeTaskPool...");
    
    // Check accessibility permissions first on macOS
    #[cfg(target_os = "macos")]
    {
        let accessibility_status = permission_resource.manager.check_permission(PermissionType::Accessibility);
        match accessibility_status {
            Ok(PermissionStatus::Authorized) => {
                info!("✅ Accessibility permissions granted - global hotkeys will work");
            },
            Ok(PermissionStatus::NotDetermined) => {
                warn!("🔐 Accessibility permissions not determined - global hotkeys will NOT work");
                warn!("Please grant accessibility permissions through the setup wizard to enable hotkeys");
                return; // Exit early, no hotkeys available
            },
            Ok(PermissionStatus::Denied) => {
                error!("❌ Accessibility permissions denied - global hotkeys will NOT work");
                error!("Please enable accessibility permissions in System Preferences → Privacy & Security → Accessibility");
                return;
            },
            Ok(PermissionStatus::Restricted) => {
                error!("🚫 Accessibility permissions restricted by system policy - global hotkeys will NOT work");
                return;
            },
            Ok(PermissionStatus::Unknown) => {
                warn!("❓ Accessibility permission status unknown - global hotkeys will NOT work");
                warn!("Please grant accessibility permissions through the setup wizard to enable hotkeys");
                return; // Exit early, no hotkeys available
            },
            Err(e) => {
                error!("💥 Failed to check accessibility permissions: {} - attempting to continue", e);
            },
        }
    }
    
    // Get the first preferred hotkey combination for setup
    if let Some(preferred_hotkey) = hotkey_prefs.preferred_combinations.first() {
        info!("Using preferred hotkey: {}", preferred_hotkey.description);
    } else {
        warn!("No preferred hotkey combinations found in preferences");
        return;
    }

    let event_proxy = event_loop_proxy.clone();
    
    // Register the actual hotkey using the HotkeyManager from ECS
    let (registered_hotkey, hotkey_description) = match ecs_hotkey::conflict::register_launcher_hotkey(&mut hotkey_manager, &hotkey_prefs) {
        Ok((hotkey, description)) => {
            info!("Successfully registered hotkey: {}", description);
            (hotkey, description)
        },
        Err(e) => {
            warn!("Failed to register any hotkey: {}. Using fallback Space key.", e);
            // Create fallback hotkey - the registration happens through the ECS hotkey manager
            let fallback_hotkey = global_hotkey::hotkey::HotKey::new(None, global_hotkey::hotkey::Code::Space);
            
            // Register the fallback hotkey using the global manager from hotkey_manager
            match hotkey_manager.global_manager.register(fallback_hotkey) {
                Ok(()) => {
                    info!("Successfully registered fallback Space hotkey");
                    (fallback_hotkey, "Space (fallback)".to_string())
                },
                Err(fallback_err) => {
                    warn!("Even fallback hotkey registration failed: {}. Hotkeys will not work.", fallback_err);
                    return;
                }
            }
        }
    };
    
    // Extract the hotkey ID for use in the async task
    let hotkey_id = registered_hotkey.id();
    let description_for_task = hotkey_description.clone();

    // Create task using AsyncComputeTaskPool instead of std::thread::spawn
    let task_pool = AsyncComputeTaskPool::get();
    let task = task_pool.spawn(async move {
        // Async hotkey event polling
        loop {
            // Use try_recv to avoid blocking in async context
            if let Ok(global_event) = global_hotkey::GlobalHotKeyEvent::receiver().try_recv() {
                info!("Raw global hotkey event received: ID={:?}", global_event.id);
                if global_event.id == hotkey_id {
                    info!("Matching hotkey event ({})! Forwarding to Bevy...", description_for_task);
                    let bevy_event = GlobalHotkeyEvent {
                        id: global_event.id,
                    };

                    match event_proxy.send_event(bevy_event) {
                        Ok(_) => info!("Successfully sent hotkey event to Bevy"),
                        Err(e) => {
                            warn!("Failed to send hotkey event to Bevy: {:?}", e);
                            break; // Exit on send failure
                        },
                    }
                } else {
                    info!("Non-matching hotkey event (expected ID: {:?})", hotkey_id);
                }
            }

            // Small delay to prevent busy polling using std::thread::sleep instead of tokio
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    });

    // Spawn entity with task component - proper Bevy ECS pattern
    commands.spawn((
        HotkeyManagerEntity,
        HotkeyPollingTask { task },
        Name::new("HotkeyPollingTask"),
    ));

    info!("Global hotkey task spawned using AsyncComputeTaskPool with component-based management!");
}

/// System to poll hotkey tasks and handle completion
#[allow(dead_code)]
pub fn poll_hotkey_tasks(
    mut commands: Commands,
    mut task_query: Query<(Entity, &mut HotkeyPollingTask), With<HotkeyManagerEntity>>,
) {
    for (entity, mut hotkey_task) in task_query.iter_mut() {
        // Poll task without blocking using poll_once
        if let Some(_result) = block_on(poll_once(&mut hotkey_task.task)) {
            info!("Hotkey task completed successfully");
            // Remove completed task component - proper cleanup
            commands.entity(entity).remove::<HotkeyPollingTask>();
        }
    }
}
