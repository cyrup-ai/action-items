use action_items_core::{LauncherEvent, LauncherEventType};
use bevy::prelude::*;
use ecs_hotkey::GlobalHotkeyManager;
use ecs_launcher::{LauncherWindowToggled, WindowTrigger};
use tracing::info;

use crate::app_main::AppState;
use crate::events::GlobalHotkeyEvent;
use crate::window::activation::{ActivationReason, WindowActivationEvent};

/// System to handle global hotkey events - true system-wide activation
/// Zero allocation, blazing-fast hotkey processing using proper Bevy event-driven pattern
#[inline]
pub fn handle_global_hotkeys(
    mut global_events: EventReader<GlobalHotkeyEvent>,
    global_manager: Option<Res<GlobalHotkeyManager>>,
    mut launcher_events: EventWriter<LauncherEvent>,
    mut window_toggle_events: EventWriter<LauncherWindowToggled>,
    app_state: Res<State<AppState>>,
    mut next_state: ResMut<NextState<AppState>>,
    mut activation_events: EventWriter<WindowActivationEvent>,
) {
    // Early return if GlobalHotkeyManager is not available (accessibility permissions not granted)
    let Some(global_manager) = global_manager else {
        // Clear any pending events to prevent accumulation
        global_events.clear();
        return;
    };
    for event in global_events.read() {
        info!("Received global hotkey event with ID: {:?}", event.id);
        if event.id == global_manager.toggle_hotkey.id() {
            info!(
                "Global hotkey activated from system! Current state: {} -> Toggling launcher \
                 visibility.",
                app_state.get().description()
            );

            match app_state.get() {
                AppState::Background => {
                    // Activate launcher when in background
                    next_state.set(AppState::LauncherActive);
                    launcher_events.write(LauncherEvent::new(LauncherEventType::SearchStarted(
                        "".to_string(),
                    )));

                    // Send launcher window toggle event to make window visible
                    window_toggle_events.write(LauncherWindowToggled {
                        visible: true,
                        trigger: WindowTrigger::Hotkey,
                        requester: "global_hotkey_handler".to_string(),
                    });

                    // Send window activation event to bring window to foreground
                    activation_events.write(WindowActivationEvent {
                        reason: ActivationReason::GlobalHotkey,
                    });
                },
                AppState::LauncherActive | AppState::SearchMode => {
                    // Hide launcher when active
                    next_state.set(AppState::Background);
                    launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));

                    // Send launcher window toggle event to hide window
                    window_toggle_events.write(LauncherWindowToggled {
                        visible: false,
                        trigger: WindowTrigger::Hotkey,
                        requester: "global_hotkey_handler".to_string(),
                    });
                },
                AppState::PreferencesOpen => {
                    // Close preferences and deactivate
                    next_state.set(AppState::Background);
                    launcher_events.write(LauncherEvent::new(LauncherEventType::SystemShutdown));

                    // Send launcher window toggle event to hide window
                    window_toggle_events.write(LauncherWindowToggled {
                        visible: false,
                        trigger: WindowTrigger::Hotkey,
                        requester: "global_hotkey_handler".to_string(),
                    });
                },
            }
        }
    }
}
