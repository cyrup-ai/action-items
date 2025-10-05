//! Activation event handling
//!
//! This module handles window activation events, event processing,
//! and event dispatching logic.

use bevy::prelude::*;
use tracing::{debug, info};

use super::types::{ActivationReason, WindowActivationEvent};

/// Event reader type alias for cleaner code
#[allow(dead_code)]
pub type WindowActivationEventReader = EventReader<'static, 'static, WindowActivationEvent>;

/// Send a window activation event
#[allow(dead_code)]
pub fn send_activation_event(
    mut events: EventWriter<WindowActivationEvent>,
    reason: ActivationReason,
) {
    let event = WindowActivationEvent {
        reason: reason.clone(),
    };
    events.write(event);
    info!("Window activation event sent: {:?}", reason);
}

/// Helper function to trigger activation from global hotkey
#[allow(dead_code)]
pub fn trigger_global_hotkey_activation(events: EventWriter<WindowActivationEvent>) {
    send_activation_event(events, ActivationReason::GlobalHotkey);
}

/// Helper function to trigger activation from user request
#[allow(dead_code)]
pub fn trigger_user_request_activation(events: EventWriter<WindowActivationEvent>) {
    send_activation_event(events, ActivationReason::UserRequest);
}

/// Helper function to trigger activation from application start
#[allow(dead_code)]
pub fn trigger_application_start_activation(events: EventWriter<WindowActivationEvent>) {
    send_activation_event(events, ActivationReason::ApplicationStart);
}

/// Process activation events with policy validation
#[allow(dead_code)]
pub fn process_activation_events(
    mut events: EventReader<WindowActivationEvent>,
    mut processed_events: EventWriter<WindowActivationEvent>,
) {
    for event in events.read() {
        // Validate activation policy
        if super::policies::validate_activation_policy(&event.reason)
            && super::policies::is_activation_permitted()
        {
            // Forward the event for actual processing
            processed_events.write(event.clone());
            debug!(
                "Activation event processed and forwarded: {:?}",
                event.reason
            );
        } else {
            debug!("Activation event blocked by policy: {:?}", event.reason);
        }
    }
}
