//! ECS Native Menu Service
//!
//! Cross-platform native menu system for Bevy ECS applications.
//! Supports macOS NSMenu, Windows menu bars, and Linux GTK menus.
//!
//! # Example
//!
//! ```rust,ignore
//! use bevy::prelude::*;
//! use action_items_ecs_native_menu::*;
//!
//! #[derive(Event, Clone, Debug)]
//! enum AppMenuEvent {
//!     NewDocument,
//!     Save,
//!     Quit,
//! }
//!
//! fn main() {
//!     App::new()
//!         .add_plugins(DefaultPlugins)
//!         .add_plugins(NativeMenuPlugin::<AppMenuEvent>::new()
//!             .with_menu_builder(|| {
//!                 MenuBuilder::new()
//!                     #[cfg(target_os = "macos")]
//!                     .app_menu("My App")
//!                     .file_menu(|m| {
//!                         m.item_with_shortcut("New", "new", AppMenuEvent::NewDocument, true, "Cmd+N");
//!                         m.item_with_shortcut("Save", "save", AppMenuEvent::Save, true, "Cmd+S");
//!                         m.separator();
//!                         m.item("Quit", "quit", AppMenuEvent::Quit, true);
//!                     })
//!                     .edit_menu_standard()
//!                     .build()
//!             })
//!         )
//!         .add_systems(Update, handle_menu_events)
//!         .run();
//! }
//!
//! fn handle_menu_events(mut events: EventReader<MenuItemClicked<AppMenuEvent>>) {
//!     for event in events.read() {
//!         match event.item {
//!             AppMenuEvent::NewDocument => println!("New document"),
//!             AppMenuEvent::Save => println!("Save"),
//!             AppMenuEvent::Quit => std::process::exit(0),
//!         }
//!     }
//! }
//! ```
//!
//! # Platform-Specific Notes
//!
//! ## macOS
//! - Menu is global (NSApp) - one menu bar for the entire app
//! - Use `menu.init_for_nsapp()` once during startup
//! - Window menu can be set with `submenu.set_as_windows_menu_for_nsapp()`
//!
//! ## Windows
//! - Each window gets its own menu bar
//! - Use `menu.init_for_hwnd(hwnd)` for each window
//! - Keyboard accelerators require `TranslateAcceleratorW` in message loop
//!   - This may not work with Bevy's event loop abstraction
//!
//! ## Linux
//! - Requires GTK integration
//! - Use `menu.init_for_gtk_window(&gtk_window, Some(&box))`
//! - May need additional GTK initialization in Bevy app
//!
//! # Dynamic Menu Updates
//!
//! Menus can be updated dynamically by sending events:
//!
//! ```rust,ignore
//! // Disable a menu item
//! events.send(MenuItemSetEnabled {
//!     item_id: "save".to_string(),
//!     enabled: false,
//! });
//!
//! // Check a menu item
//! events.send(MenuItemSetChecked {
//!     item_id: "sidebar".to_string(),
//!     checked: true,
//! });
//! ```

pub mod builders;
pub mod events;
pub mod plugin;
pub mod resources;
pub mod systems;
pub mod window_integration;

// Re-export main types
pub use builders::{MenuBuilder, MenuBuilderResult, SubmenuBuilder, SubmenuBuilderResult, MenuBuilderError};
pub use events::*;
pub use plugin::NativeMenuPlugin;
pub use resources::{MenuResource, MenuConfig, PlatformMenuConfig, set_global_menu, get_global_menu};
pub use systems::{poll_menu_events, update_menu_item_enabled, update_menu_item_checked};

// Platform-specific window integration
#[cfg(target_os = "windows")]
pub use window_integration::initialize_for_windows;

#[cfg(target_os = "linux")]
pub use window_integration::initialize_for_linux;

// Re-export muda types for convenience
pub use muda;
pub use muda::{Menu, MenuItem, CheckMenuItem, Submenu, PredefinedMenuItem, MenuId};
pub use muda::accelerator::{Accelerator, Code, Modifiers};
