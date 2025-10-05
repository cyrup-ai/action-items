//! Responsive Layout Systems for Wizard UI
//!
//! Provides systems for adaptive UI sizing based on viewport dimensions.
//! Implements responsive breakpoints for modal sizing and permission card grid layout.

#![allow(dead_code)]

use bevy::prelude::*;
use bevy::window::WindowResized;
use action_items_ecs_ui::prelude::*;
use action_items_ecs_ui::Ab;

use crate::types::PermissionType;
use crate::wizard::{WizardRoot, PermissionCard};

/// System to update modal responsiveness based on window size
/// 
/// Adapts modal size based on viewport dimensions:
/// - Large screens (>1200px): 75% width, 65% height (smaller modal)
/// - Medium screens (>800px): 85% width, 75% height (medium modal)
/// - Small screens (≤800px): 95% width, 85% height (large modal)
pub fn update_modal_responsiveness(
    mut modal_query: Query<&mut UiLayout, With<WizardRoot>>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let (width_ratio, height_ratio) = match window.width() {
            w if w > 1200.0 => (Vw(75.0), Vh(65.0)),  // Large screens: smaller modal
            w if w > 800.0 => (Vw(85.0), Vh(75.0)),   // Medium screens: medium modal  
            _ => (Vw(95.0), Vh(85.0)),                 // Small screens: large modal
        };
        
        for mut layout in modal_query.iter_mut() {
            *layout = UiLayout::window()
                .size((width_ratio, height_ratio))
                .pos((Vw(50.0), Vh(50.0)))
                .anchor(Anchor::Center)
                .pack();
        }
    }
}

/// System to update permission card grid layout based on screen size
/// 
/// Adapts card grid based on viewport dimensions:
/// - Large screens (>1200px): 3 columns, 30% width cards, 20% height
/// - Medium screens (>800px): 2 columns, 45% width cards, 25% height
/// - Small screens (≤800px): 1 column, 90% width cards, 20% height
pub fn update_permission_grid_layout(
    mut card_query: Query<(&PermissionCard, &mut UiLayout)>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let (columns, card_width, card_height, margin) = match window.width() {
            w if w > 1200.0 => (3, Rl(30.0), Rl(20.0), 5.0),   // Large: 3 columns
            w if w > 800.0 => (2, Rl(45.0), Rl(25.0), 7.5),    // Medium: 2 columns
            _ => (1, Rl(90.0), Rl(20.0), 5.0),                 // Small: 1 column
        };
        
        for (card, mut layout) in card_query.iter_mut() {
            let card_index = get_permission_index(card.permission_type);
            let col = card_index % columns;
            let row = card_index / columns;
            
            let x_pos = Rl(margin + (col as f32 * (100.0 - 2.0 * margin) / columns as f32));
            let y_pos = Rl(30.0 + (row as f32 * 30.0));
            
            *layout = UiLayout::window()
                .size((card_width, card_height))
                .pos((x_pos, y_pos))
                .pack();
        }
    }
}

/// System to update navigation button responsiveness
/// 
/// Adapts navigation button size and positioning based on screen size:
/// - Large screens: Standard button size (120px width)
/// - Medium screens: Medium button size (100px width)
/// - Small screens: Large touch-friendly buttons (140px width)
pub fn update_navigation_responsiveness(
    mut button_query: Query<&mut UiLayout, (With<crate::wizard::WizardNavigationButton>, Without<PermissionCard>)>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let button_width = match window.width() {
            w if w > 1200.0 => Ab(120.0),  // Large screens: standard buttons
            w if w > 800.0 => Ab(100.0),   // Medium screens: medium buttons
            _ => Ab(140.0),                // Small screens: large touch buttons
        };
        
        for mut layout in button_query.iter_mut() {
            *layout = UiLayout::window()
                .size((button_width, Ab(40.0)))
                .pack();
        }
    }
}

/// System to update progress indicator responsiveness
/// 
/// Adapts progress indicator size and detail level based on screen size:
/// - Large screens: Full progress bar with detailed text
/// - Medium screens: Standard progress bar with basic text
/// - Small screens: Compact progress indicator
pub fn update_progress_indicator_responsiveness(
    mut progress_query: Query<(&mut crate::wizard::WizardProgressIndicator, &mut UiLayout)>,
    windows: Query<&Window>,
) {
    if let Ok(window) = windows.single() {
        let (width, height, show_details) = match window.width() {
            w if w > 1200.0 => (Rl(60.0), Ab(30.0), true),   // Large: detailed progress
            w if w > 800.0 => (Rl(70.0), Ab(25.0), true),    // Medium: standard progress
            _ => (Rl(85.0), Ab(20.0), false),                // Small: compact progress
        };
        
        for (mut indicator, mut layout) in progress_query.iter_mut() {
            indicator.show_details = show_details;
            
            *layout = UiLayout::window()
                .size((width, height))
                .pos((Vw(50.0), Vh(10.0)))
                .anchor(Anchor::TopCenter)
                .pack();
        }
    }
}

/// System to handle window resize events and trigger responsive updates
/// 
/// Listens for window resize events and marks components for responsive updates.
/// Provides efficient batched updates when window size changes.
pub fn handle_window_resize_events(
    mut resize_events: EventReader<WindowResized>,
    mut resize_flag: Local<bool>,
) {
    for _event in resize_events.read() {
        *resize_flag = true;
    }
}

/// System to process responsive updates when needed
/// 
/// Processes batched responsive updates triggered by window resize events.
/// Ensures efficient updates without constant recalculation.
pub fn process_responsive_updates(
    resize_flag: Local<bool>,
    mut modal_query: Query<&mut UiLayout, With<WizardRoot>>,
    mut card_query: Query<(&PermissionCard, &mut UiLayout), Without<WizardRoot>>,
    windows: Query<&Window>,
) {
    if !*resize_flag {
        return;
    }
    
    if let Ok(window) = windows.single() {
        // Update modal responsiveness
        let (modal_width, modal_height) = match window.width() {
            w if w > 1200.0 => (Vw(75.0), Vh(65.0)),
            w if w > 800.0 => (Vw(85.0), Vh(75.0)),
            _ => (Vw(95.0), Vh(85.0)),
        };
        
        for mut layout in modal_query.iter_mut() {
            *layout = UiLayout::window()
                .size((modal_width, modal_height))
                .pos((Vw(50.0), Vh(50.0)))
                .anchor(Anchor::Center)
                .pack();
        }
        
        // Update card grid layout
        let (columns, card_width, card_height, margin) = match window.width() {
            w if w > 1200.0 => (3, Rl(30.0), Rl(20.0), 5.0),
            w if w > 800.0 => (2, Rl(45.0), Rl(25.0), 7.5),
            _ => (1, Rl(90.0), Rl(20.0), 5.0),
        };
        
        for (card, mut layout) in card_query.iter_mut() {
            let card_index = get_permission_index(card.permission_type);
            let col = card_index % columns;
            let row = card_index / columns;
            
            let x_pos = Rl(margin + (col as f32 * (100.0 - 2.0 * margin) / columns as f32));
            let y_pos = Rl(30.0 + (row as f32 * 30.0));
            
            *layout = UiLayout::window()
                .size((card_width, card_height))
                .pos((x_pos, y_pos))
                .pack();
        }
    }
}

/// System to update responsive font sizing based on screen size
/// 
/// Adapts font sizes for better readability across different screen sizes:
/// - Large screens: Standard font sizes
/// - Medium screens: Slightly larger fonts
/// - Small screens: Large, touch-friendly fonts
pub fn update_responsive_font_sizing(
    mut text_query: Query<&mut Text>,
    windows: Query<&Window>,
    mut last_window_size: Local<Option<(f32, f32)>>,
) {
    if let Ok(window) = windows.single() {
        let current_size = (window.width(), window.height());
        
        // Only update if window size changed significantly
        if let Some(last_size) = *last_window_size {
            let size_change = (current_size.0 - last_size.0).abs() + (current_size.1 - last_size.1).abs();
            if size_change < 50.0 {
                return; // Skip small changes to avoid constant updates
            }
        }
        
        let font_scale = match window.width() {
            w if w > 1200.0 => 1.0,    // Large screens: standard size
            w if w > 800.0 => 1.1,     // Medium screens: slightly larger
            _ => 1.3,                  // Small screens: much larger for touch
        };
        
        for _text in text_query.iter_mut() {
            // In Bevy 0.16, Text is a wrapper around a single string
            // Font size is handled through TextFont component, not Text directly
            // For now, we'll skip font scaling until proper TextFont integration
            // TODO: Integrate with TextFont component for proper font scaling
            let _ = font_scale; // Suppress unused variable warning
        }
        
        *last_window_size = Some(current_size);
    }
}

/// Helper function to get consistent permission index for grid positioning
fn get_permission_index(permission: PermissionType) -> usize {
    match permission {
        PermissionType::Accessibility => 0,
        PermissionType::ScreenCapture => 1,
        PermissionType::InputMonitoring => 2,
        PermissionType::Camera => 3,
        PermissionType::Microphone => 4,
        PermissionType::FullDiskAccess => 5,
        PermissionType::WiFi => 6,
        _ => 7, // Fallback for unknown permissions
    }
}