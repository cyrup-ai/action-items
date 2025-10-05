//! ECS Hotkey Service Integration
//!
//! Systems for integrating with the ECS hotkey service instead of manual hotkey management.

use action_items_core::LauncherEvent;
use bevy::prelude::*;
use ecs_hotkey::{
    HotkeyBinding, HotkeyPressed, HotkeyRegisterCompleted, HotkeyRegisterRequested,
    get_default_hotkey_combinations,
};
use tracing::{error, info};

use crate::app_main::AppState;
use crate::window::activation::{ActivationReason, WindowActivationEvent};

/// Startup system to register launcher hotkey using ECS service
pub fn register_launcher_hotkey_system(mut hotkey_events: EventWriter<HotkeyRegisterRequested>) {
    info!("Registering launcher hotkey via ECS hotkey service");

    // Get default hotkey combinations from ECS service
    let preferred_combinations = get_default_hotkey_combinations();

    if let Some(primary_hotkey) = preferred_combinations.first() {
        let binding = HotkeyBinding::new(primary_hotkey.clone(), "launcher_toggle")
            .with_requester("action_items_launcher");

        hotkey_events.write(HotkeyRegisterRequested {
            binding,
            requester: "action_items_launcher".to_string(),
            action: "launcher_toggle".to_string(),
            definition: primary_hotkey.clone(),
        });

        info!(
            "Requested hotkey registration: {}",
            primary_hotkey.description
        );
    } else {
        error!("No default hotkey combinations available");
    }
}

/// System to handle hotkey registration completion
pub fn handle_hotkey_registration_system(
    mut registration_events: EventReader<HotkeyRegisterCompleted>,
) {
    for event in registration_events.read() {
        if event.binding.action == "launcher_toggle" {
            if event.success {
                info!(
                    "✅ Launcher hotkey registered successfully: {}",
                    event.binding.definition.description
                );
                info!("🚀 Action Items Launcher is ready!");
                info!(
                    "📋 Press {} to activate the launcher from anywhere",
                    event.binding.definition.description
                );
                info!("⚡ The launcher will appear instantly and is ready for your commands");

                #[cfg(target_os = "macos")]
                {
                    info!("🔒 IMPORTANT: If global hotkeys don't work on macOS:");
                    info!("   1. Open System Preferences → Privacy & Security → Accessibility");
                    info!("   2. Add and enable 'action_items' or your terminal app");
                    info!("   3. Restart this app after granting permissions");
                }
            } else {
                error!(
                    "❌ Failed to register launcher hotkey: {}",
                    event.error_message.as_deref().unwrap_or("Unknown error")
                );
            }
        }
    }
}

/// System to handle hotkey press events from ECS service
pub fn handle_launcher_hotkey_press_system(
    mut hotkey_events: EventReader<HotkeyPressed>,
    mut launcher_events: EventWriter<LauncherEvent>,
    _app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut activation_events: EventWriter<WindowActivationEvent>,
) {
    for event in hotkey_events.read() {
        if event.binding.action == "launcher_toggle" {
            info!(
                "Launcher hotkey pressed: {}",
                event.binding.definition.description
            );

            // Activate launcher
            next_state.set(AppState::LauncherActive);

            // Send launcher event
            launcher_events.write(LauncherEvent::new(
                action_items_core::LauncherEventType::ShowLauncher,
            ));

            // Send window activation event
            activation_events.write(WindowActivationEvent {
                reason: ActivationReason::GlobalHotkey,
            });
        }
    }
}
