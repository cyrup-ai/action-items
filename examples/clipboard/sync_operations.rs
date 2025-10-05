//! Synchronous clipboard operations example
//! 
//! Shows how to use the clipboard resource directly for immediate operations.

use action_items_ecs_clipboard::{ClipboardData, ClipboardFormat, ClipboardPlugin};
use bevy::prelude::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClipboardPlugin)
        .add_systems(Startup, demonstrate_sync_operations)
        .run();
}

fn demonstrate_sync_operations(
    clipboard_res: Option<Res<action_items_ecs_clipboard::ClipboardResource>>,
) {
    let Some(clipboard_res) = clipboard_res else {
        error!("Clipboard not available on this platform");
        return;
    };

    info!("=== Synchronous Clipboard Operations Demo ===");

    // Set some text
    let test_text = "Hello from sync clipboard operations!";
    match clipboard_res.set_sync(ClipboardData::Text(test_text.to_string())) {
        Ok(()) => info!("✅ Set text: '{}'", test_text),
        Err(e) => error!("❌ Failed to set text: {}", e),
    }

    // Read it back
    match clipboard_res.get_sync(ClipboardFormat::Text) {
        Ok(ClipboardData::Text(text)) => info!("✅ Got text: '{}'", text),
        Ok(other) => info!("📋 Got different format: {:?}", std::mem::discriminant(&other)),
        Err(e) => error!("❌ Failed to get text: {}", e),
    }

    // Check available formats
    let formats = clipboard_res.available_formats_sync();
    info!("📋 Available formats: {:?}", formats);

    // Check specific format
    let has_text = clipboard_res.has_format_sync(ClipboardFormat::Text);
    info!("📝 Has text format: {}", has_text);

    // Set HTML content
    let html_data = ClipboardData::Html {
        html: "<p>This is <em>HTML</em> content!</p>".to_string(),
        alt_text: Some("This is HTML content!".to_string()),
    };
    
    match clipboard_res.set_sync(html_data) {
        Ok(()) => info!("✅ Set HTML content"),
        Err(e) => error!("❌ Failed to set HTML: {}", e),
    }

    // Clear clipboard
    match clipboard_res.clear_sync() {
        Ok(()) => info!("🗑️  Clipboard cleared"),
        Err(e) => error!("❌ Failed to clear clipboard: {}", e),
    }

    // Check if still has content
    let formats_after_clear = clipboard_res.available_formats_sync();
    info!("📋 Formats after clear: {:?}", formats_after_clear);

    info!("=== Demo Complete ===");
}