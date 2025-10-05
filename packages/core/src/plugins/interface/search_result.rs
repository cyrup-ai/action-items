//! Search result type for backward compatibility
//!
//! This module provides a type alias for `SearchResult` that points to `ActionItem`.
//! It's maintained for backward compatibility with existing code.

use super::ActionItem;

/// Alias for `ActionItem` to maintain backward compatibility
///
/// Use `ActionItem` instead.
pub type SearchResult = ActionItem;
