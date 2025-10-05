//! Wizard Systems Module
//!
//! Contains all Bevy systems for wizard functionality including progress tracking,
//! permission management, navigation, and UI updates. Organized for optimal
//! performance and clear separation of concerns.

pub mod progress;
pub mod permissions;
pub mod navigation;
pub mod ui_updates;
pub mod responsive;
pub mod setup;

// Re-export commonly used systems
pub use progress::*;
pub use permissions::*;
pub use navigation::*;
pub use ui_updates::*;
pub use responsive::*;
pub use setup::*;

use bevy::prelude::*;

/// System set for wizard checking and initialization
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WizardCheckSet;

/// System set for wizard UI updates and rendering
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WizardUISet;

/// System set for wizard progress tracking and state transitions
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WizardProgressSet;

/// System set for permission checking and requesting
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WizardPermissionSet;

/// System set for wizard navigation and user input
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, SystemSet)]
pub struct WizardNavigationSet;