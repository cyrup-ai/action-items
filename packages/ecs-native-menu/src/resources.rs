//! Menu resource for holding menu state and mappings

use bevy::prelude::*;
use muda::{Menu, MenuItem, CheckMenuItem, MenuId};
use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub static MENU_ITEMS: RefCell<HashMap<String, MenuItem>> = RefCell::new(HashMap::new());
    pub static CHECK_MENU_ITEMS: RefCell<HashMap<String, CheckMenuItem>> = RefCell::new(HashMap::new());
    pub static GLOBAL_MENU: RefCell<Option<Menu>> = RefCell::new(None);
}

pub fn set_global_menu(menu: Menu) {
    GLOBAL_MENU.with(|m| {
        *m.borrow_mut() = Some(menu);
    });
}

pub fn get_global_menu() -> Option<Menu> {
    GLOBAL_MENU.with(|m| m.borrow().clone())
}

/// Main menu resource holding event mappings
/// Note: muda types (Menu, MenuItem, CheckMenuItem) use Rc and are not Send+Sync
/// They are stored in thread-local storage for dynamic updates
#[derive(Resource)]
pub struct MenuResource<T: Clone + Send + Sync + 'static> {
    /// Maps muda MenuId to app-specific event values
    /// When a menu item is clicked, look up its MenuId here to get the app event
    pub event_map: HashMap<MenuId, T>,
    
    /// Maps string IDs to MenuIds (for dynamic updates)
    pub item_ids: HashMap<String, MenuId>,
    
    /// Maps string IDs to MenuIds (for check items)
    pub check_item_ids: HashMap<String, MenuId>,
    
    /// Menu configuration
    pub config: MenuConfig,
}

impl<T: Clone + Send + Sync + 'static> MenuResource<T> {
    pub fn new(
        event_map: HashMap<MenuId, T>,
        item_ids: HashMap<String, MenuId>,
        check_item_ids: HashMap<String, MenuId>,
    ) -> Self {
        Self {
            event_map,
            item_ids,
            check_item_ids,
            config: MenuConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct MenuConfig {
    /// Whether menus are enabled
    pub enabled: bool,
    
    /// Platform-specific settings
    pub platform_config: PlatformMenuConfig,
}

impl Default for MenuConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            platform_config: PlatformMenuConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PlatformMenuConfig {
    /// macOS: Menu bar is always shown (NSApp)
    #[cfg(target_os = "macos")]
    pub use_native_menu_bar: bool,
    
    /// Windows: Whether to use native menu bar vs custom rendering
    #[cfg(target_os = "windows")]
    pub use_native_menu_bar: bool,
    
    /// Linux: Whether to use GTK native menus vs custom
    #[cfg(target_os = "linux")]
    pub use_gtk_menus: bool,
}
