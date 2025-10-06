//! Visual feedback system for hotkey activations
//!
//! Provides toast notifications when hotkeys are activated, giving users
//! clear visual confirmation of their input.

use std::time::{Duration, Instant};
use bevy::prelude::*;

use crate::resources::HotkeyId;

/// Type of visual feedback to display
#[derive(Clone, Debug, PartialEq)]
pub enum FeedbackType {
    Success,
    Warning,
    Error,
}

/// Event emitted when a hotkey activation should show visual feedback
#[derive(Event, Clone, Debug)]
pub struct HotkeyVisualFeedback {
    pub hotkey_id: HotkeyId,
    pub description: String,
    pub feedback_type: FeedbackType,
}

/// Component marking a toast notification UI entity with expiration
#[derive(Component, Clone, Debug)]
pub struct HotkeyFeedbackUI {
    pub expires_at: Instant,
    pub feedback_type: FeedbackType,
    pub index: usize,
}

/// System that spawns toast notification UI entities in response to feedback events
pub fn spawn_hotkey_feedback_system(
    mut commands: Commands,
    mut feedback_events: EventReader<HotkeyVisualFeedback>,
    existing_toasts: Query<&HotkeyFeedbackUI>,
    asset_server: Res<AssetServer>,
) {
    for event in feedback_events.read() {
        // Calculate vertical stacking position
        let toast_count = existing_toasts.iter().count();
        let y_offset = toast_count as f32 * 70.0; // 60px height + 10px gap

        // Map feedback type to color
        let bg_color = match event.feedback_type {
            FeedbackType::Success => Color::srgba(0.20, 0.78, 0.35, 0.95), // Green
            FeedbackType::Warning => Color::srgba(1.0, 0.80, 0.0, 0.95),   // Yellow
            FeedbackType::Error => Color::srgba(1.0, 0.23, 0.19, 0.95),    // Red
        };

        // Load font
        let font = asset_server.load("fonts/FiraCodeNerdFontMono-Regular.ttf");

        // Spawn toast notification entity
        commands
            .spawn((
                Node {
                    position_type: PositionType::Absolute,
                    right: Val::Px(20.0),
                    bottom: Val::Px(20.0 + y_offset),
                    width: Val::Px(280.0),
                    min_height: Val::Px(60.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                BackgroundColor(bg_color),
                BorderRadius::all(Val::Px(12.0)),
                HotkeyFeedbackUI {
                    expires_at: Instant::now() + Duration::from_secs(2),
                    feedback_type: event.feedback_type.clone(),
                    index: toast_count,
                },
            ))
            .with_children(|parent| {
                parent.spawn((
                    Text::new(format!("Hotkey activated: {}", event.description)),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                ));
            });
    }
}

/// System that despawns expired toast notification entities
pub fn cleanup_feedback_ui_system(
    mut commands: Commands,
    feedback_query: Query<(Entity, &HotkeyFeedbackUI)>,
) {
    let now = Instant::now();

    for (entity, feedback_ui) in feedback_query.iter() {
        if now >= feedback_ui.expires_at {
            commands.entity(entity).despawn();
        }
    }
}
