# Task 8: Window Capture System

## Implementation Details

**File**: `ui/src/ui/window_capture.rs`  
**Lines**: 165-255  
**Architecture**: Cross-platform window capture with hotkey integration and sharing functionality  
**Integration**: ScreenCaptureAPI, HotkeySystem, ShareManager  

### Core Implementation

```rust
#[derive(Resource, Clone, Debug)]
pub struct WindowCaptureManager {
    pub capture_settings: CaptureSettings,
    pub hotkey_binding: Option<HotkeyBinding>,
    pub capture_history: VecDeque<CaptureRecord>,
    pub sharing_options: SharingOptions,
    pub permission_status: PermissionStatus,
    pub active_captures: HashMap<String, CaptureOperation>,
}

#[derive(Clone, Debug)]
pub struct CaptureSettings {
    pub target: CaptureTarget,
    pub format: ImageFormat,
    pub quality: f32,
    pub include_cursor: bool,
    pub auto_save: bool,
    pub save_location: PathBuf,
    pub filename_template: String,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CaptureTarget {
    ApplicationWindow,
    ActiveWindow,
    SelectedWindow,
    FullScreen,
    SelectedArea,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ImageFormat {
    PNG,
    JPEG,
    WEBP,
    BMP,
}

#[derive(Clone, Debug)]
pub struct CaptureRecord {
    pub id: String,
    pub captured_at: chrono::DateTime<chrono::Utc>,
    pub target: CaptureTarget,
    pub file_path: Option<PathBuf>,
    pub file_size: u64,
    pub dimensions: (u32, u32),
    pub shared_to: Vec<ShareDestination>,
}

pub fn window_capture_system(
    mut capture_manager: ResMut<WindowCaptureManager>,
    mut capture_events: EventReader<CaptureRequestEvent>,
    mut hotkey_events: EventReader<HotkeyEvent>,
    mut share_events: EventWriter<ShareRequestEvent>,
    mut ui_events: EventWriter<UINotificationEvent>,
    windows: Query<&Window>,
) {
    // Handle hotkey-triggered captures
    for hotkey_event in hotkey_events.read() {
        if let Some(ref binding) = capture_manager.hotkey_binding {
            if hotkey_event.matches_binding(binding) {
                capture_events.send(CaptureRequestEvent::HotkeyTriggered {
                    target: capture_manager.capture_settings.target,
                    timestamp: std::time::Instant::now(),
                });
            }
        }
    }

    // Process capture requests
    for capture_request in capture_events.read() {
        match capture_request {
            CaptureRequestEvent::Manual { target } => {
                start_capture_operation(*target, &mut capture_manager, &ui_events);
            }
            CaptureRequestEvent::HotkeyTriggered { target, timestamp } => {
                start_capture_operation(*target, &mut capture_manager, &ui_events);
            }
            CaptureRequestEvent::ShareAndCapture { target, destinations } => {
                let capture_id = start_capture_operation(*target, &mut capture_manager, &ui_events);
                
                if let Some(id) = capture_id {
                    // Schedule sharing after capture completes
                    share_events.send(ShareRequestEvent::ShareCapture {
                        capture_id: id,
                        destinations: destinations.clone(),
                    });
                }
            }
        }
    }

    // Update active capture operations
    update_active_captures(&mut capture_manager, &ui_events);
}

fn start_capture_operation(
    target: CaptureTarget,
    manager: &mut WindowCaptureManager,
    ui_events: &EventWriter<UINotificationEvent>,
) -> Option<String> {
    // Check permissions
    if manager.permission_status != PermissionStatus::Granted {
        ui_events.send(UINotificationEvent::Error {
            message: "Screen capture permission required".to_string(),
            action: Some(UIAction::OpenPermissionSettings),
        });
        return None;
    }

    let capture_id = generate_capture_id();
    let operation = CaptureOperation {
        id: capture_id.clone(),
        target,
        status: CaptureStatus::Starting,
        started_at: std::time::Instant::now(),
        progress: 0.0,
    };

    manager.active_captures.insert(capture_id.clone(), operation);

    // Start platform-specific capture
    match target {
        CaptureTarget::ApplicationWindow => {
            start_application_window_capture(&capture_id, &manager.capture_settings);
        }
        CaptureTarget::ActiveWindow => {
            start_active_window_capture(&capture_id, &manager.capture_settings);
        }
        CaptureTarget::SelectedWindow => {
            start_window_selection_capture(&capture_id, &manager.capture_settings);
        }
        CaptureTarget::FullScreen => {
            start_fullscreen_capture(&capture_id, &manager.capture_settings);
        }
        CaptureTarget::SelectedArea => {
            start_area_selection_capture(&capture_id, &manager.capture_settings);
        }
    }

    ui_events.send(UINotificationEvent::Info {
        message: format!("Capturing {}...", target.display_name()),
        duration: Some(std::time::Duration::from_secs(3)),
    });

    Some(capture_id)
}

#[cfg(target_os = "macos")]
fn start_application_window_capture(capture_id: &str, settings: &CaptureSettings) {
    use core_graphics::display::*;
    use core_graphics::image::*;
    
    std::thread::spawn({
        let capture_id = capture_id.to_string();
        let settings = settings.clone();
        
        move || {
            unsafe {
                // Get current application window
                let window_list = CGWindowListCopyWindowInfo(
                    kCGWindowListOptionOnScreenOnly,
                    kCGNullWindowID
                );
                
                if window_list.is_null() {
                    complete_capture_with_error(&capture_id, "Failed to get window list".to_string());
                    return;
                }
                
                // Find our application window
                let app_pid = std::process::id();
                let target_window = find_window_for_pid(window_list, app_pid);
                
                if let Some(window_id) = target_window {
                    // Capture the specific window
                    let image = CGWindowListCreateImage(
                        CGRectNull,
                        kCGWindowListOptionIncludingWindow,
                        window_id,
                        kCGWindowImageDefault | if settings.include_cursor {
                            kCGWindowImageIncludeCursor
                        } else {
                            0
                        },
                    );
                    
                    if !image.is_null() {
                        let result = save_capture_image(image, &capture_id, &settings);
                        complete_capture_operation(&capture_id, result);
                    } else {
                        complete_capture_with_error(&capture_id, "Failed to capture window".to_string());
                    }
                } else {
                    complete_capture_with_error(&capture_id, "Application window not found".to_string());
                }
            }
        }
    });
}

#[cfg(target_os = "windows")]
fn start_application_window_capture(capture_id: &str, settings: &CaptureSettings) {
    use winapi::um::winuser::*;
    use winapi::um::wingdi::*;
    
    std::thread::spawn({
        let capture_id = capture_id.to_string();
        let settings = settings.clone();
        
        move || {
            unsafe {
                // Get the current process window
                let hwnd = GetForegroundWindow();
                if hwnd.is_null() {
                    complete_capture_with_error(&capture_id, "Failed to get foreground window".to_string());
                    return;
                }
                
                // Get window dimensions
                let mut rect = RECT { left: 0, top: 0, right: 0, bottom: 0 };
                if GetWindowRect(hwnd, &mut rect) == 0 {
                    complete_capture_with_error(&capture_id, "Failed to get window rect".to_string());
                    return;
                }
                
                let width = rect.right - rect.left;
                let height = rect.bottom - rect.top;
                
                // Create device contexts
                let hdc_window = GetDC(hwnd);
                let hdc_mem = CreateCompatibleDC(hdc_window);
                
                if hdc_window.is_null() || hdc_mem.is_null() {
                    complete_capture_with_error(&capture_id, "Failed to create device contexts".to_string());
                    return;
                }
                
                // Create bitmap and capture
                let hbmp = CreateCompatibleBitmap(hdc_window, width, height);
                let hbmp_old = SelectObject(hdc_mem, hbmp as *mut _);
                
                let result = BitBlt(hdc_mem, 0, 0, width, height, hdc_window, 0, 0, SRCCOPY);
                
                if result != 0 {
                    let save_result = save_windows_bitmap(hbmp, &capture_id, &settings, width as u32, height as u32);
                    complete_capture_operation(&capture_id, save_result);
                } else {
                    complete_capture_with_error(&capture_id, "Failed to capture window content".to_string());
                }
                
                // Cleanup
                SelectObject(hdc_mem, hbmp_old);
                DeleteObject(hbmp as *mut _);
                DeleteDC(hdc_mem);
                ReleaseDC(hwnd, hdc_window);
            }
        }
    });
}

fn save_capture_image(
    image_data: &[u8],
    capture_id: &str,
    settings: &CaptureSettings,
    dimensions: (u32, u32),
) -> Result<CaptureRecord, String> {
    let timestamp = chrono::Utc::now();
    let filename = settings.filename_template
        .replace("{timestamp}", &timestamp.format("%Y%m%d_%H%M%S").to_string())
        .replace("{id}", capture_id);
    
    let file_path = settings.save_location.join(format!("{}.{}", 
        filename, 
        settings.format.extension()
    ));

    // Ensure directory exists
    if let Some(parent) = file_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create save directory: {}", e))?;
    }

    // Convert and save image
    let processed_image = match settings.format {
        ImageFormat::PNG => process_as_png(image_data, dimensions, 1.0)?,
        ImageFormat::JPEG => process_as_jpeg(image_data, dimensions, settings.quality)?,
        ImageFormat::WEBP => process_as_webp(image_data, dimensions, settings.quality)?,
        ImageFormat::BMP => process_as_bmp(image_data, dimensions)?,
    };

    std::fs::write(&file_path, processed_image)
        .map_err(|e| format!("Failed to save capture: {}", e))?;

    let file_size = std::fs::metadata(&file_path)
        .map(|m| m.len())
        .unwrap_or(0);

    Ok(CaptureRecord {
        id: capture_id.to_string(),
        captured_at: timestamp,
        target: CaptureTarget::ApplicationWindow, // Will be set correctly by caller
        file_path: Some(file_path),
        file_size,
        dimensions,
        shared_to: Vec::new(),
    })
}
```

### Hotkey Integration

**Reference**: `./docs/bevy/examples/input/keyboard_input.rs:425-458`

```rust
pub fn capture_hotkey_system(
    mut capture_manager: ResMut<WindowCaptureManager>,
    mut hotkey_events: EventReader<HotkeyRegistrationEvent>,
    mut ui_events: EventWriter<UINotificationEvent>,
) {
    for event in hotkey_events.read() {
        match event {
            HotkeyRegistrationEvent::RegisterCapture { key_combination } => {
                let binding = HotkeyBinding {
                    key_combination: key_combination.clone(),
                    action: HotkeyAction::WindowCapture,
                    enabled: true,
                    global: true,
                };

                match register_global_hotkey(&binding) {
                    Ok(_) => {
                        capture_manager.hotkey_binding = Some(binding);
                        ui_events.send(UINotificationEvent::Success {
                            message: format!("Capture hotkey registered: {}", key_combination),
                        });
                    }
                    Err(e) => {
                        ui_events.send(UINotificationEvent::Error {
                            message: format!("Failed to register capture hotkey: {}", e),
                            action: Some(UIAction::OpenHotkeySettings),
                        });
                    }
                }
            }
            HotkeyRegistrationEvent::UnregisterCapture => {
                if let Some(binding) = capture_manager.hotkey_binding.take() {
                    unregister_global_hotkey(&binding);
                    ui_events.send(UINotificationEvent::Info {
                        message: "Capture hotkey unregistered".to_string(),
                        duration: Some(std::time::Duration::from_secs(2)),
                    });
                }
            }
        }
    }
}
```

### Settings Interface

**Reference**: `./docs/bevy/examples/ui/ui_buttons.rs:485-528`

```rust
// Window Capture section
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        row_gap: Val::Px(16.0),
        ..default()
    },
    background_color: Color::rgba(0.08, 0.08, 0.08, 1.0).into(),
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
},
children: &[
    // Section header and description
    (TextBundle::from_section(
        "Window Capture",
        TextStyle {
            font: asset_server.load("fonts/Inter-SemiBold.ttf"),
            font_size: 16.0,
            color: Color::rgb(0.95, 0.95, 0.95),
        },
    ),),
    (TextBundle::from_section(
        "Capture the Raycast window to share it or add a screenshot of your extension to the Store.",
        TextStyle {
            font: asset_server.load("fonts/Inter-Regular.ttf"),
            font_size: 12.0,
            color: Color::rgb(0.7, 0.7, 0.7),
        },
    ),),
    
    // Record Hotkey button
    (ButtonBundle {
        style: Style {
            width: Val::Px(150.0),
            height: Val::Px(36.0),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            border: UiRect::all(Val::Px(1.0)),
            ..default()
        },
        background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(),
        border_color: Color::rgba(0.4, 0.4, 0.4, 1.0).into(),
        border_radius: BorderRadius::all(Val::Px(6.0)),
        ..default()
    },
    children: &[
        (IconBundle {
            icon: Icon::Keyboard,
            color: Color::rgb(0.8, 0.8, 0.8),
            size: 16.0,
            ..default()
        },),
        (TextBundle::from_section(
            match &capture_manager.hotkey_binding {
                Some(binding) => format!("Hotkey: {}", binding.key_combination),
                None => "Record Hotkey".to_string(),
            },
            TextStyle {
                font: asset_server.load("fonts/Inter-Medium.ttf"),
                font_size: 12.0,
                color: Color::rgb(0.9, 0.9, 0.9),
            },
        ),),
    ]),
    
    // Capture options
    (SettingsRowBundle {
        label: "Capture Target".to_string(),
        control: ControlType::Dropdown {
            options: vec![
                "Application Window".to_string(),
                "Active Window".to_string(),
                "Selected Window".to_string(),
                "Full Screen".to_string(),
                "Selected Area".to_string(),
            ],
            selected: match capture_manager.capture_settings.target {
                CaptureTarget::ApplicationWindow => 0,
                CaptureTarget::ActiveWindow => 1,
                CaptureTarget::SelectedWindow => 2,
                CaptureTarget::FullScreen => 3,
                CaptureTarget::SelectedArea => 4,
            },
        },
        ..default()
    },),
    
    // Format and quality settings
    (SettingsRowBundle {
        label: "Image Format".to_string(),
        control: ControlType::Dropdown {
            options: vec!["PNG".to_string(), "JPEG".to_string(), "WebP".to_string()],
            selected: match capture_manager.capture_settings.format {
                ImageFormat::PNG => 0,
                ImageFormat::JPEG => 1,
                ImageFormat::WEBP => 2,
                ImageFormat::BMP => 0, // Fallback to PNG
            },
        },
        ..default()
    },),
]
```

### Architecture Notes

- Cross-platform window capture with native API integration (macOS: Core Graphics, Windows: GDI)
- Flexible capture targets: application window, active window, screen areas, full screen
- Global hotkey support for quick capture without UI interaction
- Multiple image format support with quality control for compression formats
- Permission handling and user-friendly error messages for system restrictions
- Automatic file naming with customizable templates and timestamp integration
- Integration with sharing systems for immediate distribution to various destinations
- Capture history tracking for easy access to previous screenshots

**Bevy Examples**: `./docs/bevy/examples/window/screenshot.rs:125-162`, `./docs/bevy/examples/input/keyboard_input.rs:285-322`  
**Integration Points**: HotkeySystem, PermissionManager, ShareSystem  
**Dependencies**: ScreenCaptureAPI, HotkeyManager, FileSystem