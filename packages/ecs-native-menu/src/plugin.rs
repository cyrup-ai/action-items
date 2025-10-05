//! Native menu plugin for Bevy ECS

use bevy::prelude::*;
use std::marker::PhantomData;
use crate::{
    builders::MenuBuilderResult,
    events::*,
    resources::MenuResource,
    systems::*,
};

/// Native menu plugin that integrates cross-platform menus with Bevy ECS
/// 
/// Generic over the app's menu event type T
/// 
/// # Example
/// 
/// ```rust,ignore
/// use bevy::prelude::*;
/// use action_items_ecs_native_menu::*;
/// 
/// #[derive(Event, Clone, Debug)]
/// enum AppMenuEvent {
///     NewDocument,
///     Save,
///     Quit,
/// }
/// 
/// App::new()
///     .add_plugins(NativeMenuPlugin::<AppMenuEvent>::new()
///         .with_menu_builder(|| {
///             MenuBuilder::new()
///                 .file_menu(|m| {
///                     m.item_with_shortcut("New", "new", AppMenuEvent::NewDocument, true, "Cmd+N");
///                 })
///                 .build()
///         })
///     )
///     .run();
/// ```
pub struct NativeMenuPlugin<T: Event + Clone + Send + Sync + 'static> {
    menu_builder: Option<Box<dyn Fn() -> MenuBuilderResult<T> + Send + Sync>>,
    _phantom: PhantomData<T>,
}

impl<T: Event + Clone + Send + Sync + 'static> NativeMenuPlugin<T> {
    /// Create a new menu plugin
    pub fn new() -> Self {
        Self {
            menu_builder: None,
            _phantom: PhantomData,
        }
    }
    
    /// Set the menu builder function
    /// 
    /// The builder function should return a MenuBuilderResult containing
    /// the menu, event mappings, and item references
    pub fn with_menu_builder<F>(mut self, builder: F) -> Self
    where
        F: Fn() -> MenuBuilderResult<T> + Send + Sync + 'static,
    {
        self.menu_builder = Some(Box::new(builder));
        self
    }
}

impl<T: Event + Clone + Send + Sync + 'static> Default for NativeMenuPlugin<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Event + Clone + Send + Sync + 'static> Plugin for NativeMenuPlugin<T> {
    fn build(&self, app: &mut App) {
        // Build menu if builder is provided
        if let Some(builder) = &self.menu_builder {
            tracing::info!("Building native menu");
            let menu_result = builder();
            
            tracing::debug!(
                "Menu built with {} event mappings",
                menu_result.event_map.len()
            );
            
            // Store menu in thread-local for all platforms
            crate::resources::set_global_menu(menu_result.menu.clone());
            
            // Platform-specific initialization
            #[cfg(target_os = "macos")]
            {
                menu_result.menu.init_for_nsapp();
                tracing::info!("Menu initialized for macOS NSApp");
            }
            
            #[cfg(target_os = "windows")]
            {
                tracing::info!("Menu created. Call initialize_for_windows() with HWND to display.");
            }
            
            #[cfg(target_os = "linux")]
            {
                tracing::info!("Menu created. Call initialize_for_linux() with GTK window to display.");
            }
            
            // Insert resource with mappings
            app.insert_resource(MenuResource::new(
                menu_result.event_map,
                menu_result.item_ids,
                menu_result.check_item_ids,
            ));
        } else {
            tracing::warn!("NativeMenuPlugin created without menu builder - no menu will be displayed");
        }
        
        // Register events
        app.add_event::<MenuItemClicked<T>>();
        app.add_event::<MenuItemSetEnabled>();
        app.add_event::<MenuItemSetChecked>();
        
        // Add systems
        app.add_systems(Update, poll_menu_events::<T>);
        app.add_systems(Update, update_menu_item_enabled::<T>);
        app.add_systems(Update, update_menu_item_checked::<T>);
        
        tracing::info!("NativeMenuPlugin initialized");
    }
}
