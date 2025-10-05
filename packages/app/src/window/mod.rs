//! Window management module
//!
//! Zero-allocation window management with blazing-fast animations and focus tracking.

// Re-export all public items
#[allow(unused_imports)]
pub use activation::{
    ActivationReason, WindowActivationEvent, WindowActivationPlugin, activate_window,
};
pub use positioning::*;
pub use state::*;
pub use systems::*;
pub use ui_positioning::*;

// focused_window types used internally within positioning.rs

// Module declarations
pub mod activation;
pub mod errors;
pub mod focused_window;
pub mod positioning;
pub mod state;
pub mod systems;
mod ui_positioning;
