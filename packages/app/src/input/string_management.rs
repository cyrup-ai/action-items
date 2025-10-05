use bevy::prelude::*;
use tracing::{debug, info};

use super::InteractiveTextInput;
use super::focus::InputFocus;

/// System to initialize string interner with common strings for zero allocation
/// Pre-interns frequently used strings to eliminate runtime allocation overhead
#[inline]
pub fn initialize_string_interner_system(
    focus: ResMut<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
) {
    // Pre-intern common UI strings for maximum performance
    let common_strings = [
        "Type to search...",
        "",
        " ",
        "Search",
        "No results",
        "Loading...",
    ];

    // Intern all common strings in batch
    for text in common_strings.iter() {
        focus.intern_string(text);
    }

    // Update all existing text inputs with proper interned placeholder
    let placeholder_key = focus.placeholder_text_key();
    for mut input in text_input_query.iter_mut() {
        input.placeholder_key = placeholder_key;

        // Initialize text key if text is already present
        if !input.text.is_empty() {
            input.update_text_key(&focus);
        }
    }

    debug!(
        "String interner initialized with {} pre-interned strings",
        common_strings.len()
    );
}

/// System to efficiently intern frequently used search strings
/// Maintains zero-allocation performance for repeated text patterns
#[inline]
pub fn intern_search_strings_system(
    focus: Res<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput, Changed<InteractiveTextInput>>,
) {
    for mut input in text_input_query.iter_mut() {
        // Only intern if text has changed and isn't already interned
        if !input.text.is_empty() && input.text_key.is_none() {
            input.update_text_key(&focus);
        }
        // Clear key if text was cleared
        else if input.text.is_empty() && input.text_key.is_some() {
            input.text_key = Some(focus.empty_text_key());
        }
    }
}

/// System to manage string interner memory usage and cleanup
/// Performs periodic maintenance to optimize memory usage
#[inline]
pub fn manage_string_interner_memory_system(
    _commands: Commands,
    focus: Res<InputFocus>,
    _text_input_query: Query<&InteractiveTextInput>,
) {
    // Get interner statistics for monitoring
    let interner_len = focus.string_interner.len();

    // Log memory usage periodically (every 1000 interned strings)
    if interner_len.is_multiple_of(1000) && interner_len > 0 {
        debug!("String interner contains {} unique strings", interner_len);
    }

    // Note: ThreadedRodeo automatically handles memory efficiently,
    // so we don't need to implement manual cleanup for most cases.
    // The interner uses a compact representation and handles deduplication.
}

/// System to optimize text storage for long-lived search queries
/// Converts frequently accessed strings to interned form for better performance
#[inline]
pub fn optimize_frequent_strings_system(
    focus: Res<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput>,
) {
    // Define patterns that are likely to be repeated
    let common_search_patterns = [
        // Common single character searches
        "a",
        "b",
        "c",
        "d",
        "e",
        "f",
        "g",
        "h",
        "i",
        "j",
        "k",
        "l",
        "m",
        "n",
        "o",
        "p",
        "q",
        "r",
        "s",
        "t",
        "u",
        "v",
        "w",
        "x",
        "y",
        "z",
        // Common word prefixes
        "app",
        "calc",
        "file",
        "mail",
        "term",
        "code",
        "web",
        "doc",
        "img",
        "video",
        "music",
        "photo",
        "browser",
        "editor",
        // Common application names that users might search for
        "chrome",
        "firefox",
        "safari",
        "code",
        "terminal",
        "finder",
        "calculator",
        "calendar",
        "notes",
        "preview",
        "mail",
    ];

    // Pre-intern common patterns to avoid future allocations
    for pattern in common_search_patterns.iter() {
        focus.intern_string(pattern);
    }

    // Update any text inputs that match these patterns
    for mut input in text_input_query.iter_mut() {
        if !input.text.is_empty() {
            let text_lower = input.text.to_lowercase();
            if common_search_patterns.contains(&text_lower.as_str()) {
                input.update_text_key(&focus);
            }
        }
    }
}

/// System to handle string interning for IME composition text
/// Manages temporary composition strings with minimal allocation overhead
#[inline]
pub fn handle_ime_string_interning_system(
    focus: Res<InputFocus>,
    mut text_input_query: Query<&mut InteractiveTextInput, Changed<InteractiveTextInput>>,
) {
    for mut input in text_input_query.iter_mut() {
        // For IME composition, we generally don't intern the preedit text
        // since it's temporary and changes frequently. However, we can
        // intern the final committed text when composition completes.

        if input.ime_preedit.is_empty() && !input.text.is_empty() && input.text_key.is_none() {
            // IME composition likely just completed - intern the final text
            input.update_text_key(&focus);
        }
    }
}

/// System to provide string interner statistics for debugging and monitoring
/// Reports memory usage and performance metrics for the string interning system
#[inline]
pub fn report_string_interner_stats_system(
    focus: Res<InputFocus>,
    text_input_query: Query<&InteractiveTextInput>,
) {
    // Only run this occasionally to avoid performance overhead
    static mut STATS_COUNTER: u32 = 0;
    unsafe {
        STATS_COUNTER += 1;
        if STATS_COUNTER.is_multiple_of(3600) {
            // Every ~1 minute at 60 FPS
            let interner_len = focus.string_interner.len();
            let active_inputs = text_input_query.iter().count();
            let inputs_with_interned_text = text_input_query
                .iter()
                .filter(|input| input.text_key.is_some())
                .count();

            info!(
                "String interner stats: {} unique strings, {} active inputs, {} with interned text",
                interner_len, active_inputs, inputs_with_interned_text
            );
        }
    }
}
