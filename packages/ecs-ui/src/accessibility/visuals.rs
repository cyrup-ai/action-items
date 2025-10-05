use bevy::prelude::*;
use crate::theme::Theme;

use super::components::{FocusableElement, FocusStyle};
use super::manager::AccessibilityManager;

/// Visual focus indication system
pub fn update_focus_visuals(
    mut focusable_query: Query<
        (&FocusableElement, &mut BorderColor, &mut Transform),
        Changed<FocusableElement>,
    >,
    theme: Res<Theme>,
    _time: Res<Time>,
) {
    for (focusable, mut border_color, mut transform) in focusable_query.iter_mut() {
        if focusable.focused {
            match focusable.focus_style {
                FocusStyle::Outline => {
                    *border_color = BorderColor(theme.colors.accent_blue);
                },
                FocusStyle::Background => {
                    // Background color would be handled by a separate system
                },
                FocusStyle::Scale => {
                    transform.scale = Vec3::splat(1.02);
                },
                FocusStyle::Combined => {
                    *border_color = BorderColor(theme.colors.accent_blue);
                    transform.scale = Vec3::splat(1.01);
                },
            }
        } else {
            // Reset focus styles
            *border_color = BorderColor(Color::NONE);
            transform.scale = Vec3::splat(1.0);
        }
    }
}

/// High contrast mode support
pub fn apply_high_contrast_styles(
    accessibility_manager: Res<AccessibilityManager>,
    mut background_query: Query<&mut BackgroundColor>,
    mut text_query: Query<&mut TextColor>,
    theme: Res<Theme>,
) {
    if !accessibility_manager.high_contrast {
        return;
    }

    // Apply high contrast colors
    for mut background in background_query.iter_mut() {
        // Increase contrast for backgrounds
        if background.0.alpha() > 0.0 {
            background.0 = if background.0.to_srgba().red > 0.5 {
                Color::WHITE
            } else {
                Color::BLACK
            };
        }
    }

    for mut text_color in text_query.iter_mut() {
        // Ensure text has maximum contrast
        text_color.0 = theme.colors.text_primary;
    }
}
