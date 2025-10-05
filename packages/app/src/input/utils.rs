//! Input utility functions
//!
//! Zero-allocation utility functions for blazing-fast text input validation.

/// Helper function for text input validation (from research)
/// Zero-allocation character validation with blazing-fast private use area detection
#[inline]
pub fn is_printable_char(chr: char) -> bool {
    let is_in_private_use_area = ('\u{e000}'..='\u{f8ff}').contains(&chr)
        || ('\u{f0000}'..='\u{ffffd}').contains(&chr)
        || ('\u{100000}'..='\u{10fffd}').contains(&chr);

    !is_in_private_use_area && !chr.is_ascii_control()
}
