# Account_Menu Task 8: Profile Image System

## Task Overview
Implement secure profile image upload, management, and caching system supporting multiple image formats, automatic resizing, and secure storage with local caching capabilities.

## Implementation Requirements

### Core Components
```rust
// Profile image management system
#[derive(Resource, Reflect, Debug)]
pub struct ProfileImageResource {
    pub current_image: Option<ProfileImage>,
    pub image_cache: HashMap<String, CachedImage>,
    pub upload_queue: Vec<ImageUploadTask>,
    pub processing_settings: ImageProcessingSettings,
}

#[derive(Reflect, Debug, Clone)]
pub struct ProfileImage {
    pub image_id: String,
    pub url: String,
    pub local_cache_path: Option<PathBuf>,
    pub content_hash: String,
    pub file_size: u64,
    pub dimensions: ImageDimensions,
    pub format: ImageFormat,
    pub upload_timestamp: DateTime<Utc>,
}

#[derive(Reflect, Debug, Clone)]
pub struct CachedImage {
    pub original_url: String,
    pub local_path: PathBuf,
    pub cache_timestamp: DateTime<Utc>,
    pub content_hash: String,
    pub file_size: u64,
    pub access_count: u64,
}

#[derive(Reflect, Debug, Clone)]
pub struct ImageDimensions {
    pub width: u32,
    pub height: u32,
}

#[derive(Reflect, Debug, Clone)]
pub enum ImageFormat {
    Jpeg,
    Png,
    WebP,
    Avif,
}
```

### Image Upload System
```rust
// Secure image upload handling
#[derive(Component, Reflect, Debug)]
pub struct ImageUploadComponent {
    pub drop_zone_entity: Entity,
    pub preview_entity: Entity,
    pub progress_bar_entity: Option<Entity>,
    pub upload_button_entity: Entity,
    pub current_upload: Option<ImageUploadTask>,
}

#[derive(Reflect, Debug)]
pub struct ImageUploadTask {
    pub task_id: String,
    pub file_path: PathBuf,
    pub upload_progress: f32,
    pub validation_status: ValidationStatus,
    pub processing_status: ProcessingStatus,
    pub error: Option<String>,
}

#[derive(Reflect, Debug)]
pub enum ValidationStatus {
    Pending,
    Valid,
    Invalid { reason: String },
}

#[derive(Reflect, Debug)]
pub enum ProcessingStatus {
    Queued,
    Processing,
    Complete,
    Failed { error: String },
}

pub fn image_upload_system(
    mut upload_query: Query<&mut ImageUploadComponent>,
    mut image_res: ResMut<ProfileImageResource>,
    mut upload_events: EventReader<ImageUploadEvent>,
) {
    for upload_event in upload_events.read() {
        // Process upload with zero allocations
        process_image_upload(upload_event, &mut image_res);
    }
}
```

### Image Processing Pipeline
```rust
// Image processing and validation
#[derive(Resource, Reflect, Debug)]
pub struct ImageProcessingSettings {
    pub max_file_size: u64,
    pub allowed_formats: Vec<ImageFormat>,
    pub resize_dimensions: Vec<ImageDimensions>,
    pub quality_settings: QualitySettings,
    pub security_scanning: bool,
}

#[derive(Reflect, Debug)]
pub struct QualitySettings {
    pub jpeg_quality: u8,
    pub png_compression: u8,
    pub webp_quality: u8,
}

fn validate_and_process_image(
    file_path: &Path,
    settings: &ImageProcessingSettings,
) -> Result<ProcessedImage, ImageError> {
    // Image validation and processing pipeline
    let validated = validate_image_security(file_path)?;
    let resized = resize_image(&validated, &settings.resize_dimensions)?;
    let optimized = optimize_image(&resized, &settings.quality_settings)?;
    
    Ok(ProcessedImage {
        original: validated,
        resized_variants: resized,
        optimized: optimized,
    })
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `asset/asset_loading.rs` - Image asset loading patterns
- `async_compute/async_compute.rs` - Async image processing
- `ui/ui_texture_atlas.rs` - Image display in UI

### Implementation Pattern
```rust
// Based on asset_loading.rs for image handling
fn profile_image_loading_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    profile_res: Res<ProfileImageResource>,
) {
    if let Some(profile_image) = &profile_res.current_image {
        let handle = if let Some(cache_path) = &profile_image.local_cache_path {
            asset_server.load(cache_path)
        } else {
            // Fallback to remote URL loading
            load_remote_image(&profile_image.url)
        };
        
        commands.spawn(ProfileImageBundle {
            image_handle: handle,
            profile_image: profile_image.clone(),
        });
    }
}

// Based on async_compute.rs for image processing
fn async_image_processing_system(
    mut commands: Commands,
    processing_tasks: Query<Entity, With<ImageProcessingTask>>,
) {
    for task_entity in &processing_tasks {
        let task = commands.spawn_task(async move {
            // Async image processing
            process_image_async().await
        });
    }
}
```

## Security Requirements
- Image format validation and sanitization
- Malware scanning for uploaded images
- Content-based security validation
- Secure temporary file handling

## Performance Constraints
- **ZERO ALLOCATIONS** during image display
- Efficient image caching with LRU eviction
- Lazy loading of profile images
- Optimized image format selection

## Success Criteria
- Complete profile image management system
- Secure image upload and processing pipeline
- No unwrap()/expect() calls in production code
- Zero-allocation image display rendering
- Comprehensive image format support

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for image validation logic
- Integration tests for upload pipeline
- Security tests for malicious image handling
- Performance tests for image processing efficiency