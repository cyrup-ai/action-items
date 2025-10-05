//! Declarative menu builder API

use muda::{Menu, MenuItem, CheckMenuItem, PredefinedMenuItem, Submenu, MenuId};
use muda::accelerator::{Accelerator, Code, Modifiers};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MenuBuilderError {
    #[error("Failed to append menu item: {0}")]
    AppendFailed(String),
    #[error("Failed to parse keyboard shortcut: {0}")]
    ShortcutParseFailed(String),
}

/// Result of building a menu - contains Menu and event mappings
pub struct MenuBuilderResult<T: Clone> {
    pub menu: Menu,
    /// Maps muda MenuId -> app event for routing
    pub event_map: HashMap<MenuId, T>,
    /// Maps string IDs to MenuIds (for dynamic updates)
    pub item_ids: HashMap<String, MenuId>,
    /// Maps string IDs to MenuIds (for check items)
    pub check_item_ids: HashMap<String, MenuId>,
}

pub struct MenuBuilder<T: Clone> {
    menu: Menu,
    event_map: HashMap<MenuId, T>,
    item_ids: HashMap<String, MenuId>,
    check_item_ids: HashMap<String, MenuId>,
}

impl<T: Clone> MenuBuilder<T> {
    pub fn new() -> Self {
        Self {
            menu: Menu::new(),
            event_map: HashMap::new(),
            item_ids: HashMap::new(),
            check_item_ids: HashMap::new(),
        }
    }
    
    /// Add macOS app menu (automatically includes About, Services, Quit, etc.)
    #[cfg(target_os = "macos")]
    pub fn app_menu(self, app_name: &str) -> Self {
        let self_mut = self;
        let app_m = Submenu::new(app_name, true);
        if let Err(e) = app_m.append_items(&[
            &PredefinedMenuItem::about(None, None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::services(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::hide(None),
            &PredefinedMenuItem::hide_others(None),
            &PredefinedMenuItem::show_all(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::quit(None),
        ]) {
            tracing::error!("Failed to append app menu items: {}", e);
        }
        if let Err(e) = self_mut.menu.append(&app_m) {
            tracing::error!("Failed to append app menu: {}", e);
        }
        self_mut
    }
    
    /// Add File menu with custom builder
    pub fn file_menu(mut self, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> Self {
        let mut submenu = SubmenuBuilder::new("&File", true);
        builder(&mut submenu);
        let result = submenu.build();
        if let Err(e) = self.menu.append(&result.submenu) {
            tracing::error!("Failed to append file menu: {}", e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    /// Add standard Edit menu (Undo, Redo, Cut, Copy, Paste, Select All)
    pub fn edit_menu_standard(self) -> Self {
        let self_mut = self;
        let edit_m = Submenu::new("&Edit", true);
        if let Err(e) = edit_m.append_items(&[
            &PredefinedMenuItem::undo(None),
            &PredefinedMenuItem::redo(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::cut(None),
            &PredefinedMenuItem::copy(None),
            &PredefinedMenuItem::paste(None),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::select_all(None),
        ]) {
            tracing::error!("Failed to append edit menu items: {}", e);
        }
        if let Err(e) = self_mut.menu.append(&edit_m) {
            tracing::error!("Failed to append edit menu: {}", e);
        }
        self_mut
    }
    
    /// Add custom Edit menu
    pub fn edit_menu(mut self, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> Self {
        let mut submenu = SubmenuBuilder::new("&Edit", true);
        builder(&mut submenu);
        let result = submenu.build();
        if let Err(e) = self.menu.append(&result.submenu) {
            tracing::error!("Failed to append edit menu: {}", e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    /// Add View menu
    pub fn view_menu(mut self, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> Self {
        let mut submenu = SubmenuBuilder::new("&View", true);
        builder(&mut submenu);
        let result = submenu.build();
        if let Err(e) = self.menu.append(&result.submenu) {
            tracing::error!("Failed to append view menu: {}", e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    /// Add Window menu
    pub fn window_menu(mut self, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> Self {
        let mut submenu = SubmenuBuilder::new("&Window", true);
        builder(&mut submenu);
        let result = submenu.build();
        if let Err(e) = self.menu.append(&result.submenu) {
            tracing::error!("Failed to append window menu: {}", e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    /// Add standard Window menu (Minimize, Maximize, Close, Fullscreen)
    pub fn window_menu_standard(self) -> Self {
        let self_mut = self;
        let window_m = Submenu::new("&Window", true);
        if let Err(e) = window_m.append_items(&[
            &PredefinedMenuItem::minimize(None),
            &PredefinedMenuItem::maximize(None),
            &PredefinedMenuItem::close_window(Some("Close")),
            &PredefinedMenuItem::separator(),
            &PredefinedMenuItem::fullscreen(None),
            &PredefinedMenuItem::bring_all_to_front(None),
        ]) {
            tracing::error!("Failed to append window menu items: {}", e);
        }
        if let Err(e) = self_mut.menu.append(&window_m) {
            tracing::error!("Failed to append window menu: {}", e);
        }
        self_mut
    }
    
    /// Add custom menu with any name
    pub fn custom_menu(mut self, name: &str, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> Self {
        let mut submenu = SubmenuBuilder::new(name, true);
        builder(&mut submenu);
        let result = submenu.build();
        if let Err(e) = self.menu.append(&result.submenu) {
            tracing::error!("Failed to append custom menu '{}': {}", name, e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    pub fn build(self) -> MenuBuilderResult<T> {
        MenuBuilderResult {
            menu: self.menu,
            event_map: self.event_map,
            item_ids: self.item_ids,
            check_item_ids: self.check_item_ids,
        }
    }
}

impl<T: Clone> Default for MenuBuilder<T> {
    fn default() -> Self {
        Self::new()
    }
}

/// Result of building a submenu
pub struct SubmenuBuilderResult<T: Clone> {
    pub submenu: Submenu,
    pub event_map: HashMap<MenuId, T>,
    pub item_ids: HashMap<String, MenuId>,
    pub check_item_ids: HashMap<String, MenuId>,
}

pub struct SubmenuBuilder<T: Clone> {
    submenu: Submenu,
    event_map: HashMap<MenuId, T>,
    item_ids: HashMap<String, MenuId>,
    check_item_ids: HashMap<String, MenuId>,
}

impl<T: Clone> SubmenuBuilder<T> {
    pub fn new(name: &str, enabled: bool) -> Self {
        Self {
            submenu: Submenu::new(name, enabled),
            event_map: HashMap::new(),
            item_ids: HashMap::new(),
            check_item_ids: HashMap::new(),
        }
    }
    
    /// Add menu item with event mapping
    /// - `label`: Display text
    /// - `id`: String ID for dynamic updates
    /// - `event`: App event to emit when clicked
    /// - `enabled`: Whether item is initially enabled
    pub fn item(&mut self, label: &str, id: &str, event: T, enabled: bool) -> &mut Self {
        let menu_item = MenuItem::new(label, enabled, None);
        let menu_id = menu_item.id().clone();
        
        // Append to submenu first
        if let Err(e) = self.submenu.append(&menu_item) {
            tracing::error!("Failed to append menu item '{}': {}", label, e);
        }
        
        // Store in thread-local
        crate::resources::MENU_ITEMS.with(|items| {
            items.borrow_mut().insert(id.to_string(), menu_item);
        });
        
        // Store MenuId in builder for Resource
        self.event_map.insert(menu_id.clone(), event);
        self.item_ids.insert(id.to_string(), menu_id);
        self
    }
    
    /// Add menu item with keyboard shortcut
    pub fn item_with_shortcut(
        &mut self,
        label: &str,
        id: &str,
        event: T,
        enabled: bool,
        shortcut: &str,
    ) -> &mut Self {
        let accelerator = parse_shortcut(shortcut);
        let menu_item = MenuItem::new(label, enabled, accelerator);
        let menu_id = menu_item.id().clone();
        
        // Append to submenu first
        if let Err(e) = self.submenu.append(&menu_item) {
            tracing::error!("Failed to append menu item '{}' with shortcut '{}': {}", label, shortcut, e);
        }
        
        // Store in thread-local
        crate::resources::MENU_ITEMS.with(|items| {
            items.borrow_mut().insert(id.to_string(), menu_item);
        });
        
        // Store MenuId in builder for Resource
        self.event_map.insert(menu_id.clone(), event);
        self.item_ids.insert(id.to_string(), menu_id);
        self
    }
    
    /// Add checkable menu item
    pub fn check_item(
        &mut self,
        label: &str,
        id: &str,
        event: T,
        enabled: bool,
        checked: bool,
    ) -> &mut Self {
        let check_item = CheckMenuItem::new(label, enabled, checked, None);
        let menu_id = check_item.id().clone();
        
        // Append to submenu first
        if let Err(e) = self.submenu.append(&check_item) {
            tracing::error!("Failed to append check menu item '{}': {}", label, e);
        }
        
        // Store in thread-local
        crate::resources::CHECK_MENU_ITEMS.with(|items| {
            items.borrow_mut().insert(id.to_string(), check_item);
        });
        
        // Store MenuId in builder for Resource
        self.event_map.insert(menu_id.clone(), event);
        self.check_item_ids.insert(id.to_string(), menu_id);
        self
    }
    
    /// Add separator
    pub fn separator(&mut self) -> &mut Self {
        if let Err(e) = self.submenu.append(&PredefinedMenuItem::separator()) {
            tracing::error!("Failed to append separator: {}", e);
        }
        self
    }
    
    /// Add nested submenu
    pub fn submenu(&mut self, name: &str, builder: impl FnOnce(&mut SubmenuBuilder<T>)) -> &mut Self {
        let mut sub = SubmenuBuilder::new(name, true);
        builder(&mut sub);
        let result = sub.build();
        if let Err(e) = self.submenu.append(&result.submenu) {
            tracing::error!("Failed to append submenu '{}': {}", name, e);
        }
        self.event_map.extend(result.event_map);
        self.item_ids.extend(result.item_ids);
        self.check_item_ids.extend(result.check_item_ids);
        self
    }
    
    pub fn build(self) -> SubmenuBuilderResult<T> {
        SubmenuBuilderResult {
            submenu: self.submenu,
            event_map: self.event_map,
            item_ids: self.item_ids,
            check_item_ids: self.check_item_ids,
        }
    }
}

/// Parse keyboard shortcut string (e.g., "Cmd+N", "Ctrl+Shift+S", "Alt+F4")
/// Supports: Cmd/Super/Meta, Ctrl/Control, Shift, Alt/Option
fn parse_shortcut(shortcut: &str) -> Option<Accelerator> {
    let parts: Vec<&str> = shortcut.split('+').collect();
    if parts.is_empty() {
        return None;
    }
    
    let mut modifiers = Modifiers::empty();
    let mut code = None;
    
    for part in parts {
        match part.trim().to_lowercase().as_str() {
            "cmd" | "super" | "meta" | "command" => modifiers |= Modifiers::SUPER,
            "ctrl" | "control" => modifiers |= Modifiers::CONTROL,
            "shift" => modifiers |= Modifiers::SHIFT,
            "alt" | "option" => modifiers |= Modifiers::ALT,
            key => code = parse_key_code(key),
        }
    }
    
    code.map(|c| Accelerator::new(Some(modifiers), c))
}

/// Parse individual key code
fn parse_key_code(key: &str) -> Option<Code> {
    match key.to_uppercase().as_str() {
        "A" => Some(Code::KeyA),
        "B" => Some(Code::KeyB),
        "C" => Some(Code::KeyC),
        "D" => Some(Code::KeyD),
        "E" => Some(Code::KeyE),
        "F" => Some(Code::KeyF),
        "G" => Some(Code::KeyG),
        "H" => Some(Code::KeyH),
        "I" => Some(Code::KeyI),
        "J" => Some(Code::KeyJ),
        "K" => Some(Code::KeyK),
        "L" => Some(Code::KeyL),
        "M" => Some(Code::KeyM),
        "N" => Some(Code::KeyN),
        "O" => Some(Code::KeyO),
        "P" => Some(Code::KeyP),
        "Q" => Some(Code::KeyQ),
        "R" => Some(Code::KeyR),
        "S" => Some(Code::KeyS),
        "T" => Some(Code::KeyT),
        "U" => Some(Code::KeyU),
        "V" => Some(Code::KeyV),
        "W" => Some(Code::KeyW),
        "X" => Some(Code::KeyX),
        "Y" => Some(Code::KeyY),
        "Z" => Some(Code::KeyZ),
        "0" | "DIGIT0" => Some(Code::Digit0),
        "1" | "DIGIT1" => Some(Code::Digit1),
        "2" | "DIGIT2" => Some(Code::Digit2),
        "3" | "DIGIT3" => Some(Code::Digit3),
        "4" | "DIGIT4" => Some(Code::Digit4),
        "5" | "DIGIT5" => Some(Code::Digit5),
        "6" | "DIGIT6" => Some(Code::Digit6),
        "7" | "DIGIT7" => Some(Code::Digit7),
        "8" | "DIGIT8" => Some(Code::Digit8),
        "9" | "DIGIT9" => Some(Code::Digit9),
        "F1" => Some(Code::F1),
        "F2" => Some(Code::F2),
        "F3" => Some(Code::F3),
        "F4" => Some(Code::F4),
        "F5" => Some(Code::F5),
        "F6" => Some(Code::F6),
        "F7" => Some(Code::F7),
        "F8" => Some(Code::F8),
        "F9" => Some(Code::F9),
        "F10" => Some(Code::F10),
        "F11" => Some(Code::F11),
        "F12" => Some(Code::F12),
        "ESCAPE" | "ESC" => Some(Code::Escape),
        "ENTER" | "RETURN" => Some(Code::Enter),
        "SPACE" => Some(Code::Space),
        "TAB" => Some(Code::Tab),
        "BACKSPACE" => Some(Code::Backspace),
        "DELETE" | "DEL" => Some(Code::Delete),
        "ARROWUP" | "UP" => Some(Code::ArrowUp),
        "ARROWDOWN" | "DOWN" => Some(Code::ArrowDown),
        "ARROWLEFT" | "LEFT" => Some(Code::ArrowLeft),
        "ARROWRIGHT" | "RIGHT" => Some(Code::ArrowRight),
        "HOME" => Some(Code::Home),
        "END" => Some(Code::End),
        "PAGEUP" | "PGUP" => Some(Code::PageUp),
        "PAGEDOWN" | "PGDOWN" => Some(Code::PageDown),
        "PLUS" | "EQUAL" => Some(Code::Equal),
        "MINUS" => Some(Code::Minus),
        "COMMA" => Some(Code::Comma),
        "PERIOD" => Some(Code::Period),
        "SLASH" => Some(Code::Slash),
        "BACKSLASH" => Some(Code::Backslash),
        "SEMICOLON" => Some(Code::Semicolon),
        "QUOTE" => Some(Code::Quote),
        "BRACKETLEFT" | "LEFTBRACKET" => Some(Code::BracketLeft),
        "BRACKETRIGHT" | "RIGHTBRACKET" => Some(Code::BracketRight),
        _ => {
            tracing::warn!("Unknown key code: {}", key);
            None
        }
    }
}
