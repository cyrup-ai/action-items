//! Window integration for initializing menus on different platforms

pub mod bevy_windows;

#[cfg(target_os = "windows")]
pub use bevy_windows::initialize_for_windows;

#[cfg(target_os = "linux")]
pub use bevy_windows::initialize_for_linux;
