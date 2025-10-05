# Advanced_Menu_2 Task 8: Window Capture System

## Task Overview
Implement comprehensive screenshot and sharing functionality with window selection, area capture, annotation tools, and direct sharing capabilities for productivity workflows.

## Implementation Requirements

### Core Components
```rust
// Window capture system
#[derive(Resource, Reflect, Debug)]
pub struct WindowCaptureResource {
    pub capture_engine: CaptureEngine,
    pub annotation_tools: AnnotationToolset,
    pub sharing_manager: SharingManager,
    pub capture_history: CaptureHistory,
}

#[derive(Reflect, Debug)]
pub struct CaptureEngine {
    pub supported_modes: Vec<CaptureMode>,
    pub active_captures: HashMap<String, ActiveCapture>,
    pub capture_settings: CaptureSettings,
    pub output_formats: Vec<ImageFormat>,
}

#[derive(Reflect, Debug)]
pub enum CaptureMode {
    FullScreen,
    ActiveWindow,
    WindowSelection,
    AreaSelection,
    ScrollingCapture,
    TimedCapture { delay: Duration },
}

#[derive(Component, Reflect, Debug)]
pub struct WindowCaptureComponent {
    pub mode_selector: Entity,
    pub preview_area: Entity,
    pub annotation_toolbar: Entity,
    pub sharing_panel: Entity,
    pub capture_history_list: Entity,
}

#[derive(Reflect, Debug)]
pub struct ActiveCapture {
    pub capture_id: String,
    pub mode: CaptureMode,
    pub start_time: DateTime<Utc>,
    pub status: CaptureStatus,
    pub progress: f32,
    pub output_path: Option<PathBuf>,
}

#[derive(Reflect, Debug)]
pub enum CaptureStatus {
    Preparing,
    Capturing,
    Processing,
    Complete,
    Failed { error: String },
    Cancelled,
}

pub fn window_capture_system(
    mut capture_res: ResMut<WindowCaptureResource>,
    capture_events: EventReader<CaptureEvent>,
    mut capture_result_events: EventWriter<CaptureResultEvent>,
) {
    for capture_event in capture_events.read() {
        match capture_event {
            CaptureEvent::StartCapture { mode, settings } => {
                let capture_id = initiate_capture(
                    &mut capture_res.capture_engine,
                    mode,
                    settings,
                );
                capture_result_events.send(CaptureResultEvent::CaptureStarted {
                    capture_id,
                });
            }
            CaptureEvent::CancelCapture { capture_id } => {
                cancel_capture(&mut capture_res.capture_engine, capture_id);
            }
        }
    }
}
```

### Screen Capture Implementation
```rust
// Cross-platform screen capture
async fn perform_screen_capture(
    mode: CaptureMode,
    settings: &CaptureSettings,
) -> Result<CapturedImage, CaptureError> {
    match mode {
        CaptureMode::FullScreen => {
            capture_full_screen(settings).await
        }
        CaptureMode::ActiveWindow => {
            let active_window = get_active_window().await?;
            capture_window(&active_window, settings).await
        }
        CaptureMode::WindowSelection => {
            let selected_window = prompt_window_selection().await?;
            capture_window(&selected_window, settings).await
        }
        CaptureMode::AreaSelection => {
            let selected_area = prompt_area_selection().await?;
            capture_area(&selected_area, settings).await
        }
        CaptureMode::ScrollingCapture => {
            capture_scrolling_content(settings).await
        }
        CaptureMode::TimedCapture { delay } => {
            sleep(delay).await;
            capture_full_screen(settings).await
        }
    }
}

#[cfg(target_os = "macos")]
async fn capture_full_screen(settings: &CaptureSettings) -> Result<CapturedImage, CaptureError> {
    // macOS-specific implementation using CGWindowListCreateImage
    use core_graphics::display::CGDisplay;
    use core_graphics::image::CGImage;
    
    let display = CGDisplay::main();
    let image = display.image().ok_or(CaptureError::CaptureFailed)?;
    convert_cgimage_to_captured_image(image, settings)
}

#[cfg(target_os = "windows")]
async fn capture_full_screen(settings: &CaptureSettings) -> Result<CapturedImage, CaptureError> {
    // Windows-specific implementation using Windows.Graphics.Capture
    use windows::Graphics::Capture::GraphicsCaptureItem;
    
    // Windows capture implementation
    todo!("Implement Windows screen capture")
}

#[cfg(target_os = "linux")]
async fn capture_full_screen(settings: &CaptureSettings) -> Result<CapturedImage, CaptureError> {
    // Linux-specific implementation using X11 or Wayland
    todo!("Implement Linux screen capture")
}
```

### Annotation System
```rust
// Built-in annotation tools
#[derive(Reflect, Debug)]
pub struct AnnotationToolset {
    pub available_tools: Vec<AnnotationTool>,
    pub active_annotations: Vec<Annotation>,
    pub tool_settings: ToolSettings,
}

#[derive(Reflect, Debug)]
pub enum AnnotationTool {
    Arrow,
    Rectangle,
    Circle,
    Text,
    Highlight,
    Blur,
    Crop,
    Freehand,
}

#[derive(Reflect, Debug)]
pub struct Annotation {
    pub annotation_id: String,
    pub tool_type: AnnotationTool,
    pub position: AnnotationPosition,
    pub style: AnnotationStyle,
    pub content: Option<String>,
}

#[derive(Reflect, Debug)]
pub struct AnnotationPosition {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

pub fn annotation_system(
    mut annotation_events: EventReader<AnnotationEvent>,
    mut capture_res: ResMut<WindowCaptureResource>,
) {
    for event in annotation_events.read() {
        match event {
            AnnotationEvent::AddAnnotation { tool, position, style } => {
                add_annotation(&mut capture_res.annotation_tools, tool, position, style);
            }
            AnnotationEvent::RemoveAnnotation { annotation_id } => {
                remove_annotation(&mut capture_res.annotation_tools, annotation_id);
            }
        }
    }
}
```

### Sharing Integration
```rust
// Direct sharing capabilities
#[derive(Reflect, Debug)]
pub struct SharingManager {
    pub sharing_services: Vec<SharingService>,
    pub upload_queue: VecDeque<UploadTask>,
    pub sharing_history: Vec<SharingRecord>,
}

#[derive(Reflect, Debug)]
pub enum SharingService {
    Clipboard,
    Email,
    Slack,
    Discord,
    CloudStorage { service: String },
    Custom { name: String, endpoint: String },
}

async fn share_capture(
    captured_image: &CapturedImage,
    service: &SharingService,
) -> Result<SharingResult, SharingError> {
    match service {
        SharingService::Clipboard => {
            copy_to_clipboard(captured_image).await
        }
        SharingService::Email => {
            create_email_with_attachment(captured_image).await
        }
        SharingService::CloudStorage { service } => {
            upload_to_cloud_storage(captured_image, service).await
        }
        SharingService::Custom { endpoint, .. } => {
            upload_to_custom_endpoint(captured_image, endpoint).await
        }
        _ => Err(SharingError::ServiceNotImplemented),
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `async_compute/async_compute.rs` - Async capture operations
- `ui/ui.rs` - Capture UI components
- `window/window_settings.rs` - Window management for capture

### Implementation Pattern
```rust
// Based on async_compute.rs for capture operations
fn async_capture_system(
    mut commands: Commands,
    capture_tasks: Query<Entity, With<CaptureTask>>,
) {
    for task_entity in &capture_tasks {
        let task = commands.spawn_task(async move {
            // Async screen capture with progress reporting
            perform_capture_operation().await
        });
    }
}
```

## Platform-Specific Implementation
- macOS: Core Graphics and ScreenCaptureKit integration
- Windows: Windows.Graphics.Capture API usage
- Linux: X11/Wayland screenshot capabilities
- Cross-platform fallback mechanisms

## Performance Constraints
- **ZERO ALLOCATIONS** during capture initiation
- Efficient image processing and compression
- Minimal memory footprint for large captures
- Optimized annotation rendering

## Success Criteria
- Complete window capture system implementation
- Cross-platform screenshot functionality
- No unwrap()/expect() calls in production code
- Zero-allocation capture coordination
- Comprehensive annotation and sharing features

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for capture mode logic
- Integration tests for annotation system
- Performance tests for large image handling
- Cross-platform compatibility tests