//! Search input handling systems
//!
//! Real KeyboardInput handling using EventReader patterns from bevy/examples/input/text_input.rs

use bevy::input::keyboard::{Key, KeyboardInput};
use bevy::prelude::*;

use crate::ui::components::SearchInput;

/// Event fired when search query changes - PUBLIC for main app access
#[derive(Event, Clone)]
pub struct SearchQueryChanged {
    pub query: String,
}

/// Search input system using REAL KeyboardInput patterns from text_input.rs example
#[inline]
pub fn search_input_system(
    mut events: EventReader<KeyboardInput>,
    mut search_text: Query<&mut Text, With<SearchInput>>,
    mut search_results: EventWriter<SearchQueryChanged>,
) {
    let Ok(mut text) = search_text.single_mut() else {
        return;
    };

    for event in events.read() {
        // Only trigger changes when the key is first pressed
        if !event.state.is_pressed() {
            continue;
        }

        let mut query_changed = false;

        match (&event.logical_key, &event.text) {
            (Key::Backspace, _) => {
                if **text == "Search..." {
                    // Clear placeholder
                    **text = String::new();
                } else {
                    text.pop();
                }
                query_changed = true;
            },
            (Key::Escape, _) => {
                **text = "Search...".to_string();
                query_changed = true;
            },
            (_, Some(inserted_text)) => {
                // Clear placeholder on first input
                if **text == "Search..." {
                    **text = String::new();
                }

                // Make sure the text doesn't have any control characters
                if inserted_text.chars().all(is_printable_char) {
                    text.push_str(inserted_text);
                    query_changed = true;
                }
            },
            _ => continue,
        }

        // Send search event when query changes
        if query_changed {
            search_results.write(SearchQueryChanged {
                query: if **text == "Search..." {
                    String::new()
                } else {
                    text.to_string()
                },
            });
        }
    }
}

/// From text_input.rs example - check if character is printable
fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}

/// System to show/hide results container based on search input
#[inline]
pub fn results_visibility_system(
    search_text: Query<&Text, (With<SearchInput>, Changed<Text>)>,
    mut results: Query<&mut Visibility, With<crate::ui::components::ResultsContainer>>,
) {
    let Ok(text) = search_text.single() else {
        return;
    };
    let Ok(mut visibility) = results.single_mut() else {
        return;
    };

    *visibility = if text.is_empty() || **text == "Search..." {
        Visibility::Hidden
    } else {
        Visibility::Visible
    };
}
