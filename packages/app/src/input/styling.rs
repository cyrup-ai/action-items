use action_items_ecs_ui::theme::Theme;
use bevy::prelude::*;

use super::InteractiveTextInput;
use super::focus::InputFocus;

/// Complex query type for text input styling with multiple color components
type TextInputStylingQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static InteractiveTextInput,
        &'static mut BackgroundColor,
        &'static mut BorderColor,
    ),
    (Changed<InteractiveTextInput>, With<InteractiveTextInput>),
>;

/// Simple text input query for selection styling
type TextInputSelectionQuery<'w, 's> = Query<
    'w,
    's,
    (&'static InteractiveTextInput, &'static mut BackgroundColor),
    (Changed<InteractiveTextInput>, With<InteractiveTextInput>),
>;

/// System to apply focus-aware styling to text inputs with zero allocation
/// Updates visual styling based on focus state for optimal user feedback
#[inline]
pub fn apply_text_input_focus_styling_system(
    theme: Res<Theme>,
    mut text_input_query: TextInputStylingQuery,
) {
    for (input, mut bg_color, mut border_color) in text_input_query.iter_mut() {
        if input.is_focused && input.focus_visible {
            // Apply keyboard focus styling with accent border
            *bg_color = BackgroundColor(theme.colors.search_input_background());
            *border_color = BorderColor(theme.colors.search_focus_border());
        } else if input.is_focused {
            // Mouse focus - no visible border indicator but active background
            *bg_color = BackgroundColor(theme.colors.search_input_background());
            *border_color = BorderColor(Color::NONE);
        } else {
            // Not focused - default neutral styling
            *bg_color = BackgroundColor(theme.colors.surface_default);
            *border_color = BorderColor(Color::NONE);
        }
    }
}

/// System to update text color styling based on content state
/// Differentiates between placeholder text and actual content
#[inline]
pub fn update_text_styling_system(
    theme: Res<Theme>,
    _focus: Res<InputFocus>,
    mut text_input_query: Query<
        (&InteractiveTextInput, &mut TextColor),
        Changed<InteractiveTextInput>,
    >,
) {
    for (input, mut text_color) in text_input_query.iter_mut() {
        if input.text.is_empty() && input.ime_preedit.is_empty() {
            // Placeholder text styling - use tertiary color for subtle appearance
            *text_color = TextColor(theme.colors.text_tertiary);
        } else if !input.ime_preedit.is_empty() {
            // IME composition text - use slightly different color to indicate temporary state
            *text_color = TextColor(theme.colors.text_secondary);
        } else {
            // Normal text content - primary color for maximum readability
            *text_color = TextColor(theme.colors.text_primary);
        }
    }
}

/// System to apply selection styling for text inputs
/// Highlights selected text with appropriate background color
#[inline]
pub fn apply_text_selection_styling_system(
    theme: Res<Theme>,
    mut text_input_query: TextInputSelectionQuery,
) {
    for (input, mut bg_color) in text_input_query.iter_mut() {
        if input.has_selection() {
            // Apply selection background color
            *bg_color = BackgroundColor(theme.colors.surface_selected);
        } else if input.is_focused {
            // Focused but no selection - normal focus background
            *bg_color = BackgroundColor(theme.colors.search_input_background());
        } else {
            // Default state
            *bg_color = BackgroundColor(theme.colors.surface_default);
        }
    }
}

/// System to animate focus transitions smoothly
/// Provides professional animated feedback for focus changes
#[inline]
pub fn animate_focus_transitions_system(
    mut text_input_query: TextInputStylingQuery,
    theme: Res<Theme>,
) {
    // For now, we apply instant styling changes
    // In the future, this could interpolate between colors for smooth transitions
    for (input, mut bg_color, mut border_color) in text_input_query.iter_mut() {
        // Calculate target colors based on focus state
        let (target_bg, target_border) = if input.is_focused && input.focus_visible {
            (
                theme.colors.search_input_background(),
                theme.colors.search_focus_border(),
            )
        } else if input.is_focused {
            (theme.colors.search_input_background(), Color::NONE)
        } else {
            (theme.colors.surface_default, Color::NONE)
        };

        // For now, apply immediately - could add interpolation here later
        *bg_color = BackgroundColor(target_bg);
        *border_color = BorderColor(target_border);
    }
}

/// System to handle visual feedback for IME composition
/// Provides distinct styling for composition text to improve UX
#[inline]
pub fn apply_ime_composition_styling_system(
    theme: Res<Theme>,
    mut text_input_query: Query<
        (&InteractiveTextInput, &mut BorderColor),
        Changed<InteractiveTextInput>,
    >,
) {
    for (input, mut border_color) in text_input_query.iter_mut() {
        if !input.ime_preedit.is_empty() {
            // Show subtle border to indicate active IME composition
            *border_color = BorderColor(theme.colors.accent_blue.with_alpha(0.4));
        } else if input.is_focused && input.focus_visible {
            // Normal focus border
            *border_color = BorderColor(theme.colors.search_focus_border());
        } else {
            // No border
            *border_color = BorderColor(Color::NONE);
        }
    }
}

/// System to apply hover styling for focusable text inputs
/// Provides visual feedback on mouse hover for better UX
#[inline]
#[allow(clippy::type_complexity)]
pub fn apply_hover_styling_system(
    theme: Res<Theme>,
    mut interaction_query: Query<
        (&Interaction, &InteractiveTextInput, &mut BackgroundColor),
        (Changed<Interaction>, With<InteractiveTextInput>),
    >,
) {
    for (interaction, input, mut bg_color) in interaction_query.iter_mut() {
        match interaction {
            Interaction::Hovered => {
                if !input.is_focused {
                    // Hover effect only when not focused
                    *bg_color = BackgroundColor(theme.colors.surface_hover);
                }
            },
            Interaction::Pressed => {
                // Active press state
                *bg_color = BackgroundColor(theme.colors.surface_active);
            },
            Interaction::None => {
                if !input.is_focused {
                    // Return to default when not hovered and not focused
                    *bg_color = BackgroundColor(theme.colors.surface_default);
                }
            },
        }
    }
}

/// System to update cursor visual indicators
/// Manages cursor visibility and styling for focused inputs
#[inline]
pub fn update_cursor_styling_system(
    mut text_input_query: Query<
        (&InteractiveTextInput, &mut Visibility),
        With<InteractiveTextInput>,
    >,
) {
    // This system would manage cursor blinking and positioning
    // For now, we focus on the core styling - cursor rendering would be
    // handled by a dedicated cursor component system

    for (input, mut visibility) in text_input_query.iter_mut() {
        if input.is_focused {
            // Cursor should be visible when focused
            *visibility = Visibility::Visible;
        } else {
            // Hide cursor when not focused
            *visibility = Visibility::Hidden;
        }
    }
}
