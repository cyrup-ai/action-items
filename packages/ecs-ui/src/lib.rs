#![doc = include_str!("../README.md")]
#![allow(clippy::type_complexity)]
#![recursion_limit = "256"]

//! Lunex UI system decomposed into focused modules for maintainability
//!
//! This library provides a comprehensive UI system built on Bevy ECS patterns.
//! The implementation is organized into specialized modules:
//!
//! - [`resources`] - UI resources and events (UiTheme, DirtyLayout, UiEvent)
//! - [`traits`] - Extension traits and helper traits (UiCommands, ImageTextureConstructor)
//! - [`components`] - Core UI components (Dimension, UiDepth, UiEmbedding)
//! - [`layout`] - Layout system and computation (UiLayout, UiLayoutRoot)
//! - [`state`] - UI state management (UiState, UiBase, state traits)
//! - [`color`] - Color management and styling (UiColor, color blending)
//! - [`text`] - Text handling systems (UiTextSize, text layout)
//! - [`camera`] - Camera integration systems (UiFetchFromCamera)
//! - [`plugins`] - Plugin definitions and system sets
//!
//! Additional modules from the original codebase:
//! - [`cursor`] - Cursor management and interaction
//! - [`layouts`] - Layout type definitions and algorithms
//! - [`picking`] - UI picking and interaction systems
//! - [`states`] - State components and transitions
//! - [`textanim`] - Text animation systems
//! - [`units`] - UI unit types and value system

// Core type imports for internal use

// Bevy framework imports
pub(crate) use bevy::app::PluginGroupBuilder;
pub(crate) use bevy::app::prelude::*;
pub(crate) use bevy::asset::RenderAssetUsages;
pub(crate) use bevy::asset::prelude::*;
pub(crate) use bevy::color::prelude::*;
pub(crate) use bevy::gizmos::gizmos::Gizmos;
pub(crate) use bevy::picking::backend::{HitData, PointerHits};
pub(crate) use bevy::picking::pointer::{PointerId, PointerLocation};
pub(crate) use bevy::picking::prelude::{Pickable, Pointer};
pub(crate) use bevy::prelude::*;
pub(crate) use bevy::render::camera::{Camera, ClearColorConfig, RenderTarget};
pub(crate) use bevy::render::mesh::MeshAabb;
pub(crate) use bevy::render::primitives::Aabb;
pub(crate) use bevy::render::render_resource::{
    Extent3d, TextureDimension, TextureFormat, TextureUsages,
};
pub(crate) use bevy::render::view::visibility::VisibilityClass;
pub(crate) use bevy::render::view::{self, Visibility};
pub(crate) use bevy::sprite::Anchor;
pub(crate) use bevy::text::{TextLayoutInfo, update_text2d_layout};
pub(crate) use bevy::transform::TransformSystem;
#[cfg(feature = "text3d")]
pub(crate) use bevy_rich_text3d::*;
pub(crate) use colored::Colorize;

// Import and re-export the existing modules
mod cursor;
pub use cursor::*;
mod layouts;
pub use layouts::*;
mod picking;
pub use picking::*;
mod states;
pub use states::*;
mod textanim;
pub use textanim::*;
mod units;
pub use units::*;

// Import and re-export the new decomposed modules
pub mod resources;
pub use resources::*;

pub mod traits;
pub use traits::*;

pub mod components;
pub use components::*;

pub mod layout;
pub use layout::*;

pub mod state;
pub use state::*;

pub mod color;
pub use color::*;

pub mod text;
pub use text::*;

pub mod camera;
pub use camera::*;

// Animation system for UI transitions
pub mod animations;

// Performance monitoring system
pub mod performance;

pub mod plugins;
pub use plugins::*;

// Visibility event system for UI components
pub mod visibility;
pub use visibility::{
    UiAnimationCompleteEvent, UiComponentTarget, UiVisibilityAnimation,
    UiVisibilityAnimationType, UiVisibilityEvent, VisibilityPlugin,
};

// Theme system for all UI plugins
pub mod theme;
pub use theme::{ColorPalette, FontScale, ShadowElevation, SpacingScale, Theme, ThemeProvider};

// Gradient system for theme-based gradients
pub mod gradients;
pub use gradients::{
    GradientComponent,
    GradientComponentType,
    GradientInteractionState,
    GradientPlugin,
    GradientTheme,
    InteractiveGradient,
    // Gradient systems
    animate_gradient_transitions_system,
    apply_gradient_system,
    dynamic_gradient_theme_system,
    gradient_accessibility_system,
    interactive_gradient_system,
    optimize_gradient_performance_system,
    validate_gradient_theme_system,
};

// Responsive layout system for viewport-based sizing
pub mod responsive;
pub use responsive::{
    // Components
    ContentConstraints,
    TextTruncation,
    TruncationState,
    ViewportResponsiveContainer,
    // Plugin
    ResponsivePlugin,
    // Systems (for advanced usage)
    content_constraints_system,
    text_truncation_system,
    update_viewport_responsive_container_system,
};

// Accessibility system
pub mod accessibility;
pub use accessibility::{
    AccessibilityPlugin,
    AccessibleElement,
    FocusableElement,
    AccessibilityManager,
    FocusChanged,
    ScreenReaderAnnouncement,
};

// Performance monitoring system exports
pub use performance::{
    PerformancePlugin, 
    PerformanceMetrics, 
    PerformanceManager, 
    VirtualizedEntity,
    PerformanceWarning,
    WarningLevel,
};

// Icon system for file and content type icons
pub mod icons;
pub use icons::{IconCache, IconSize, IconTheme, IconType, ThemeColors};

// Public API prelude module for convenient imports
pub mod prelude {
    // Default plugins and system sets
    // Export stuff from other crates
    pub use bevy::sprite::Anchor;
    #[cfg(feature = "text3d")]
    pub use bevy::text::cosmic_text::Weight;
    #[cfg(feature = "text3d")]
    pub use bevy_rich_text3d::*;

    // Debug plugins
    pub use crate::UiLunexDebugPlugin;
    // Import other module preludes
    pub use crate::cursor::prelude::*;
    pub use crate::layouts::prelude::*;
    pub use crate::states::prelude::*;
    pub use crate::units::*;
    // Theme system types
    pub use crate::theme::{Theme, ThemeProvider, ColorPalette, FontScale, SpacingScale, ShadowElevation};
    // Gradient types
    pub use crate::gradients::{
        GradientComponent,
        GradientComponentType,
        GradientInteractionState,
        GradientPlugin,
        GradientTheme,
        InteractiveGradient,
    };
    // Visibility events
    pub use crate::visibility::{
        UiAnimationCompleteEvent, UiComponentTarget, UiVisibilityAnimation,
        UiVisibilityAnimationType, UiVisibilityEvent, VisibilityPlugin,
    };
    // Responsive layout components
    pub use crate::responsive::{
        ContentConstraints,
        ResponsivePlugin,
        TextTruncation,
        TruncationState,
        ViewportResponsiveContainer,
    };
    // Accessibility
    pub use crate::accessibility::{
        AccessibilityPlugin,
        AccessibleElement,
        FocusableElement,
        LiveRegion,
        FocusStyle,
    };
    // Performance monitoring
    pub use crate::performance::{
        PerformancePlugin,
        PerformanceMetrics,
        PerformanceManager,
        VirtualizedEntity,
        PerformanceWarning,
        WarningLevel,
    };
    // Icon types
    pub use crate::icons::{IconCache, IconSize, IconTheme, IconType, ThemeColors};
    // Animation system types
    pub use crate::animations::{
        AnimationPlugin,
        AnimationState,
        AnimationSequence,
        EasingFunction,
        WindowAnimation,
        ListAnimation,
        ItemAnimation,
    };
    // All standard components from decomposed modules
    pub use crate::{
        CameraTextureRenderConstructor,

        // Core components
        Dimension,
        DirtyLayout,
        ImageTextureConstructor,
        LayoutCache,

        RecomputeUiLayout,
        // Animation support
        TextAnimator,
        UiBase,
        // Color and styling
        UiColor,
        // Traits
        UiCommands,
        UiDepth,
        UiEmbedding,
        // Events and resources
        UiEvent,
        UiFetchFromCamera,
        UiImageSize,
        // Layout system
        UiLayout,
        UiLayoutRoot,
        UiMeshPlane2d,
        UiMeshPlane3d,
        UiRoot3d,

        UiSourceCamera,

        // State management
        UiState,
        UiStateTrait,

        UiTextSize,
        UiTheme,

        UiThemeChanged,
    };
    pub use crate::{UiLunexPlugins, UiSystems};
}
