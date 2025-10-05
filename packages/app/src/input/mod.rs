//! Input management module
//!
//! Zero-allocation input handling with blazing-fast keyboard processing, focus management,
//! IME support, and string interning for optimal performance.

// Re-export all public items
pub use focus::*;
pub use focus_systems::*;
pub use ime::*;
pub use state::*;
pub use string_management::*;
pub use styling::*;
pub use systems::*;
pub use text_handling::*;
pub use utils::*;

// Module declarations
pub mod focus;
pub mod focus_systems;
pub mod ime;
mod state;
pub mod string_management;
pub mod styling;
mod systems;
mod text_handling;
mod utils;
