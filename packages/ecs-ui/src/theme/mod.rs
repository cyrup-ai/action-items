//! Theme system modules
//!
//! Modular organization of the theme system with colors, typography, spacing, shadows, and
//! utilities.

pub mod colors;
pub mod provider;
pub mod shadows;
pub mod spacing;
pub mod typography;
pub mod utils;

// Re-export main types and functions
pub use colors::ColorPalette;
pub use provider::{Theme, ThemeProvider};
pub use shadows::ShadowElevation;
pub use spacing::SpacingScale;
pub use typography::FontScale;

// Note: Gradient types temporarily disabled until Bevy 0.17+ adds gradient support
