//! Core icon type system for Bevy applications
//!
//! Provides foundational types for icon management including:
//! - Icon sizing standards
//! - Universal icon type categories
//! - Cross-platform theme detection
//! - Icon caching infrastructure
//! - Async icon extraction from OS
//!
//! # Modules
//! - [`types`] - IconSize, IconType enums
//! - [`theme`] - ThemeColors, IconTheme resources
//! - [`cache`] - IconCache resource for loaded icons
//! - [`components`] - IconInteractionState, IconAnimation components
//! - [`events`] - Icon extraction and state change events
//! - [`extraction`] - Async icon extraction systems

pub mod types;
pub mod theme;
pub mod cache;
pub mod components;
pub mod events;
pub mod extraction;

// Re-export public types
pub use types::{IconSize, IconType};
pub use theme::{ThemeColors, IconTheme};
pub use cache::IconCache;
pub use components::{IconInteractionState, IconComponent, IconAnimation};
pub use events::{
    IconExtractionRequest,
    IconExtractionResult,
    IconColorChangeEvent,
    IconSizeChangeEvent,
    IconStateChangeEvent,
    IconAnimationCompleteEvent,
    IconAnimationType,
};
pub use extraction::{
    IconExtractionInProgress,
    process_icon_extraction_requests,
    poll_icon_extraction_tasks,
    process_icon_extraction_results,
};
