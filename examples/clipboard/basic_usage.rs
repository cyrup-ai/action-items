//! Basic clipboard usage example
//! 
//! Demonstrates how to read and write clipboard data using the Action Items ECS clipboard system.

use action_items_ecs_clipboard::{ClipboardData, ClipboardFormat, ClipboardPlugin};
use bevy::prelude::*;
use tokio::sync::oneshot;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(ClipboardPlugin)
        .add_systems(Startup, setup_clipboard_demo)
        .add_systems(Update, handle_keyboard_input)
        .add_systems(Update, handle_clipboard_changes)
        .run();
}

fn setup_clipboard_demo() {
    info!("=== Clipboard Demo Started ===");
    info!("Controls:");
    info!("  T - Set text to clipboard");
    info!("  G - Get text from clipboard");
    info!("  H - Set HTML to clipboard");
    info!("  C - Clear clipboard");
    info!("  F - Check available formats");
    info!("  ESC - Exit");
}

fn handle_keyboard_input(
    input: Res<ButtonInput<KeyCode>>,
    clipboard_res: Option<Res<action_items_ecs_clipboard::ClipboardResource>>,
    mut clipboard_events: EventWriter<action_items_ecs_clipboard::ClipboardRequest>,
    mut exit: EventWriter<AppExit>,
) {
    let Some(_clipboard_res) = clipboard_res else {
        if input.just_pressed(KeyCode::Escape) {
            exit.send(AppExit::Success);
        }
        return;
    };

    if input.just_pressed(KeyCode::KeyT) {
        info!("Setting text to clipboard...");
        let (sender, _receiver) = oneshot::channel();
        clipboard_events.send(action_items_ecs_clipboard::ClipboardRequest::Set {
            data: ClipboardData::Text("Hello from Bevy ECS Clipboard!".to_string()),
            response_sender: sender,
        });
    }

    if input.just_pressed(KeyCode::KeyG) {
        info!("Getting text from clipboard...");
        let (sender, _receiver) = oneshot::channel();
        clipboard_events.send(action_items_ecs_clipboard::ClipboardRequest::Get {
            format: ClipboardFormat::Text,
            response_sender: sender,
        });
    }

    if input.just_pressed(KeyCode::KeyH) {
        info!("Setting HTML to clipboard...");
        let (sender, _receiver) = oneshot::channel();
        clipboard_events.send(action_items_ecs_clipboard::ClipboardRequest::Set {
            data: ClipboardData::Html {
                html: "<h1>Hello from Bevy!</h1><p>This is <strong>HTML</strong> content.</p>".to_string(),
                alt_text: Some("Hello from Bevy! This is HTML content.".to_string()),
            },
            response_sender: sender,
        });
    }

    if input.just_pressed(KeyCode::KeyC) {
        info!("Clearing clipboard...");
        let (sender, _receiver) = oneshot::channel();
        clipboard_events.send(action_items_ecs_clipboard::ClipboardRequest::Clear {
            response_sender: sender,
        });
    }

    if input.just_pressed(KeyCode::KeyF) {
        info!("Checking available formats...");
        let (sender, _receiver) = oneshot::channel();
        clipboard_events.send(action_items_ecs_clipboard::ClipboardRequest::GetAvailableFormats {
            response_sender: sender,
        });
    }

    if input.just_pressed(KeyCode::Escape) {
        info!("Exiting...");
        exit.send(AppExit::Success);
    }
}

fn handle_clipboard_changes(
    mut events: EventReader<action_items_ecs_clipboard::ClipboardChanged>,
) {
    for event in events.read() {
        if event.has_content {
            info!("📋 Clipboard now has {} content", event.format);
        } else {
            info!("🗑️  Clipboard {} content cleared", event.format);
        }
    }
}