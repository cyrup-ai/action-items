//! Search result type alias
//!
//! This module provides a type alias for `SearchResult` that points to `ActionItem`.
//! Provides domain-specific naming for search contexts.

use super::ActionItem;

/// Alias for `ActionItem` for search-specific contexts
///
/// Prefer `ActionItem` in new code for consistency.
pub type SearchResult = ActionItem;
