//! Example demonstrating clipboard operations with the arboard-based implementation

use action_items_ecs_clipboard::{ArboardManager, ClipboardData, ClipboardFormat};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing arboard-based clipboard implementation...");

    // Test text operations
    println!("\n=== Testing Text Operations ===");
    let test_text = "Hello from arboard-based clipboard!";

    match ArboardManager::set_text(test_text.to_string()).await {
        Ok(()) => println!("✓ Successfully set text to clipboard"),
        Err(e) => println!("✗ Failed to set text: {}", e),
    }

    match ArboardManager::get_text().await {
        Ok(text) => {
            if text == test_text {
                println!("✓ Successfully retrieved text: '{}'", text);
            } else {
                println!(
                    "✗ Text mismatch. Expected: '{}', Got: '{}'",
                    test_text, text
                );
            }
        },
        Err(e) => println!("✗ Failed to get text: {}", e),
    }

    // Test HTML operations
    println!("\n=== Testing HTML Operations ===");
    let test_html = "<h1>Hello HTML!</h1><p>This is a test.</p>";
    let alt_text = "Hello HTML!\nThis is a test.";

    match ArboardManager::set_html(test_html.to_string(), Some(alt_text.to_string())).await {
        Ok(()) => println!("✓ Successfully set HTML to clipboard"),
        Err(e) => println!("✗ Failed to set HTML: {}", e),
    }

    match ArboardManager::get_html().await {
        Ok(html) => println!("✓ Successfully retrieved HTML: '{}'", html),
        Err(e) => println!("✗ Failed to get HTML: {}", e),
    }

    // Test format checking
    println!("\n=== Testing Format Detection ===");
    let available_formats = ArboardManager::available_formats().await;
    println!("Available formats: {:?}", available_formats);

    for format in &[
        ClipboardFormat::Text,
        ClipboardFormat::Html,
        ClipboardFormat::Files,
    ] {
        let has_format = ArboardManager::has_format(*format).await;
        println!("Has {:?}: {}", format, has_format);
    }

    #[cfg(feature = "image-data")]
    {
        let has_image = ArboardManager::has_format(ClipboardFormat::Image).await;
        println!("Has Image: {}", has_image);
    }

    // Test generic get/set operations
    println!("\n=== Testing Generic Operations ===");
    let data = ClipboardData::Text("Generic text test".to_string());

    match ArboardManager::set(data.clone()).await {
        Ok(()) => println!("✓ Successfully set data with generic method"),
        Err(e) => println!("✗ Failed to set data with generic method: {}", e),
    }

    match ArboardManager::get(ClipboardFormat::Text).await {
        Ok(retrieved_data) => match retrieved_data {
            ClipboardData::Text(text) => {
                println!(
                    "✓ Successfully retrieved data with generic method: '{}'",
                    text
                );
            },
            _ => println!("✗ Retrieved wrong data type"),
        },
        Err(e) => println!("✗ Failed to get data with generic method: {}", e),
    }

    // Test clear operation
    println!("\n=== Testing Clear Operation ===");
    match ArboardManager::clear().await {
        Ok(()) => println!("✓ Successfully cleared clipboard"),
        Err(e) => println!("✗ Failed to clear clipboard: {}", e),
    }

    // Verify clear worked
    match ArboardManager::get_text().await {
        Ok(text) => println!("⚠ Clipboard still has text after clear: '{}'", text),
        Err(_) => println!("✓ Clipboard successfully cleared"),
    }

    println!("\nClipboard test completed!");
    Ok(())
}
