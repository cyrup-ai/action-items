//! Wizard Visibility Bridge
//!
//! Bridges the permissions wizard visibility events to the launcher window visibility system.
//! This allows the wizard to request the OS window to be shown/hidden without direct dependencies.

use action_items_ecs_permissions::WizardVisibilityEvent;
use bevy::prelude::*;
use tracing::info;

use crate::window::LauncherState;

/// Bridge system to handle wizard visibility events
///
/// Listens for WizardVisibilityEvent from the permissions wizard and updates
/// the launcher window visibility state accordingly.
pub fn bridge_wizard_visibility_to_window(
    mut wizard_visibility_events: EventReader<WizardVisibilityEvent>,
    mut launcher_state: ResMut<LauncherState>,
) {
    for event in wizard_visibility_events.read() {
        info!(
            "🔗 Bridging wizard visibility event: visible={}, reason={}",
            event.visible, event.reason
        );

        launcher_state.visible = event.visible;

        if event.visible {
            // Reset focus tracking when showing the window
            launcher_state.has_gained_focus = false;
            launcher_state.show_timestamp = Some(std::time::Instant::now());
            info!("✅ Launcher window set to visible for wizard");
        } else {
            // Clear timestamp when hiding
            launcher_state.show_timestamp = None;
            info!("✅ Launcher window set to hidden after wizard");
        }
    }
}
