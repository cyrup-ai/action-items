# Task 3: QA Validation - Custom Wallpaper Integration System

## Comprehensive Testing Protocol

**File**: `tests/ui/wallpaper_system_tests.rs`  
**Test Coverage**: 95%+ required  
**Integration**: WallpaperManager, AssetManager, RenderSystem, FileManager  

### Test Categories

#### 1. Wallpaper File Validation Testing
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:425-458`
```rust
#[test]
fn test_wallpaper_file_validation() {
    let validation_settings = ValidationSettings {
        max_file_size_mb: 10.0,
        max_dimensions: (4096, 4096),
        min_dimensions: (100, 100),
        allowed_formats: vec![WallpaperFormat::PNG, WallpaperFormat::JPEG, WallpaperFormat::WEBP],
        require_aspect_ratio_match: true,
        color_profile_validation: true,
    };

    // Test valid wallpaper
    let valid_wallpaper_path = create_test_image_file(1920, 1080, WallpaperFormat::PNG, 2.0); // 2MB
    let result = validate_wallpaper_file(&valid_wallpaper_path, &validation_settings);
    assert!(result.is_ok(), "Valid wallpaper should pass validation");
    
    let metadata = result.unwrap();
    assert_eq!(metadata.original_dimensions, (1920, 1080));
    assert_eq!(metadata.format, WallpaperFormat::PNG);

    // Test file too large
    let large_file_path = create_test_image_file(2048, 2048, WallpaperFormat::PNG, 15.0); // 15MB
    let result = validate_wallpaper_file(&large_file_path, &validation_settings);
    assert!(result.is_err(), "Oversized file should fail validation");
    
    if let Err(ValidationError::FileTooLarge { size_mb, max_size_mb }) = result {
        assert!(size_mb > max_size_mb);
    } else {
        panic!("Expected FileTooLarge error");
    }

    // Test unsupported format
    let bmp_file_path = create_test_image_file(800, 600, WallpaperFormat::BMP, 1.0);
    let result = validate_wallpaper_file(&bmp_file_path, &validation_settings);
    assert!(result.is_err(), "Unsupported format should fail validation");
    
    if let Err(ValidationError::UnsupportedFormat(format)) = result {
        assert_eq!(format, WallpaperFormat::BMP);
    } else {
        panic!("Expected UnsupportedFormat error");
    }

    // Test dimensions too small
    let tiny_file_path = create_test_image_file(50, 50, WallpaperFormat::PNG, 0.1);
    let result = validate_wallpaper_file(&tiny_file_path, &validation_settings);
    assert!(result.is_err(), "Tiny dimensions should fail validation");

    // Test aspect ratio mismatch
    let square_file_path = create_test_image_file(1000, 1000, WallpaperFormat::PNG, 1.0); // 1:1 aspect ratio
    let result = validate_wallpaper_file(&square_file_path, &validation_settings);
    assert!(result.is_err(), "Square aspect ratio should fail validation");

    // Cleanup test files
    let _ = std::fs::remove_file(valid_wallpaper_path);
    let _ = std::fs::remove_file(large_file_path);
    let _ = std::fs::remove_file(bmp_file_path);
    let _ = std::fs::remove_file(tiny_file_path);
    let _ = std::fs::remove_file(square_file_path);
}
```

#### 2. Wallpaper Loading and Caching Testing
```rust
#[test]
fn test_wallpaper_loading_and_caching() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(AssetPlugin::default())
       .add_systems(Update, wallpaper_system)
       .add_event::<WallpaperEvent>()
       .add_event::<UINotificationEvent>()
       .add_event::<RenderUpdateEvent>();

    let mut cache_manager = WallpaperCache {
        cached_textures: HashMap::new(),
        cache_size_limit: 5,
        compression_enabled: true,
        preload_enabled: false,
    };

    let wallpaper_manager = WallpaperManager {
        current_wallpaper: None,
        default_wallpaper: Handle::default(),
        validation_settings: ValidationSettings::default(),
        cache_manager: cache_manager.clone(),
        rendering_settings: RenderingSettings::default(),
        file_watcher: None,
    };

    app.world_mut().insert_resource(wallpaper_manager);

    // Create test wallpaper file
    let test_file_path = create_test_image_file(1920, 1080, WallpaperFormat::PNG, 1.5);
    
    // Send load event
    let mut wallpaper_events = app.world_mut().resource_mut::<Events<WallpaperEvent>>();
    wallpaper_events.send(WallpaperEvent::LoadFromFile {
        file_path: test_file_path.clone(),
    });

    app.update();

    // Verify wallpaper was loaded
    let manager = app.world().resource::<WallpaperManager>();
    assert!(manager.current_wallpaper.is_some(), "Wallpaper should be loaded");
    
    let wallpaper = manager.current_wallpaper.as_ref().unwrap();
    assert_eq!(wallpaper.file_path, test_file_path);
    assert_eq!(wallpaper.validation_status, ValidationStatus::Valid);

    // Verify cache entry was created
    assert!(manager.cache_manager.cached_textures.contains_key(&test_file_path));
    
    let cached = manager.cache_manager.cached_textures.get(&test_file_path).unwrap();
    assert_eq!(cached.access_count, 1);
    assert!(cached.memory_usage > 0);

    // Verify render update event was sent
    let render_events = app.world().resource::<Events<RenderUpdateEvent>>();
    let mut render_reader = render_events.get_reader();
    let render_events_vec: Vec<_> = render_reader.read(&render_events).collect();
    
    assert!(!render_events_vec.is_empty(), "Render update should be triggered");
    
    if let RenderUpdateEvent::WallpaperChanged { handle, .. } = &render_events_vec[0] {
        assert_eq!(*handle, wallpaper.handle);
    }

    // Cleanup
    let _ = std::fs::remove_file(test_file_path);
}
```

#### 3. Cache Management and Performance Testing
**Reference**: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs:325-358`
```rust
#[test]
fn test_wallpaper_cache_management() {
    let mut cache_manager = WallpaperCache {
        cached_textures: HashMap::new(),
        cache_size_limit: 3,
        compression_enabled: false,
        preload_enabled: false,
    };

    let asset_server = AssetServer::default();

    // Fill cache to limit
    for i in 0..5 {
        let test_path = PathBuf::from(format!("/tmp/test_wallpaper_{}.png", i));
        let metadata = WallpaperMetadata {
            original_dimensions: (800, 600),
            file_size: 1024 * 100 * (i as u64 + 1), // Varying sizes
            format: WallpaperFormat::PNG,
            color_profile: None,
            compression_ratio: 0.8,
            has_transparency: false,
        };

        let result = load_wallpaper_async(
            test_path.clone(),
            metadata,
            &asset_server,
            &mut cache_manager,
        );
        
        assert!(result.is_ok(), "Loading wallpaper {} should succeed", i);
    }

    // Cache should be limited to cache_size_limit
    assert!(cache_manager.cached_textures.len() <= cache_manager.cache_size_limit,
        "Cache size should be limited, got {} entries", cache_manager.cached_textures.len());

    // Test cache hit
    let existing_path = PathBuf::from("/tmp/test_wallpaper_4.png");
    let cached_entry = cache_manager.cached_textures.get(&existing_path);
    
    if let Some(cached) = cached_entry {
        let initial_access_count = cached.access_count;
        
        // Load same wallpaper again
        let metadata = WallpaperMetadata::default();
        let result = load_wallpaper_async(
            existing_path.clone(),
            metadata,
            &asset_server,
            &mut cache_manager,
        );
        
        assert!(result.is_ok(), "Cache hit should succeed");
        
        let updated_cached = cache_manager.cached_textures.get(&existing_path).unwrap();
        assert_eq!(updated_cached.access_count, initial_access_count + 1);
    }

    // Test manual cache cleanup
    cleanup_wallpaper_cache(&mut cache_manager);
    
    let target_size = (cache_manager.cache_size_limit * 3) / 4;
    assert!(cache_manager.cached_textures.len() <= target_size,
        "Cache cleanup should reduce size to target");
}
```

#### 4. Rendering System Integration Testing
**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:485-522`
```rust
#[test]
fn test_wallpaper_rendering_integration() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_plugins(AssetPlugin::default())
       .add_systems(Update, wallpaper_render_system)
       .add_event::<RenderUpdateEvent>();

    // Setup wallpaper component
    let test_handle = Handle::<Image>::default();
    let rendering_settings = RenderingSettings {
        scale_mode: ScaleMode::AspectFit,
        alignment: Alignment::Center,
        opacity: 0.8,
        blur_radius: 0.0,
        tint_color: Color::rgb(1.0, 0.9, 0.8),
        performance_mode: PerformanceMode::Quality,
    };

    let wallpaper_entity = setup_wallpaper_component(
        &mut app.world_mut().spawn_empty(),
        test_handle.clone(),
        &rendering_settings,
    );

    // Send render update event
    let mut render_events = app.world_mut().resource_mut::<Events<RenderUpdateEvent>>();
    render_events.send(RenderUpdateEvent::WallpaperChanged {
        handle: test_handle.clone(),
        rendering_settings: rendering_settings.clone(),
    });

    app.update();

    // Verify component was updated
    let ui_image = app.world().entity(wallpaper_entity).get::<UiImage>();
    assert!(ui_image.is_some(), "UiImage component should exist");
    
    let ui_image = ui_image.unwrap();
    assert_eq!(ui_image.texture, test_handle);
    assert_eq!(ui_image.color, rendering_settings.tint_color);

    // Test different scale modes
    let scale_modes = vec![
        ScaleMode::Stretch,
        ScaleMode::AspectFit,
        ScaleMode::Center,
        ScaleMode::AspectFill,
        ScaleMode::Tile,
    ];

    for scale_mode in scale_modes {
        let mut updated_settings = rendering_settings.clone();
        updated_settings.scale_mode = scale_mode;

        let mut render_events = app.world_mut().resource_mut::<Events<RenderUpdateEvent>>();
        render_events.send(RenderUpdateEvent::WallpaperChanged {
            handle: test_handle.clone(),
            rendering_settings: updated_settings,
        });

        app.update();

        // Verify style was updated appropriately for scale mode
        let style = app.world().entity(wallpaper_entity).get::<Style>();
        assert!(style.is_some(), "Style component should exist for scale mode: {:?}", scale_mode);
    }
}
```

#### 5. File Format Detection Testing
```rust
#[test]
fn test_image_format_detection() {
    let test_cases = vec![
        // (file_data, expected_format)
        (vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A], WallpaperFormat::PNG),
        (vec![0xFF, 0xD8, 0xFF, 0xE0], WallpaperFormat::JPEG),
        (vec![0xFF, 0xD8, 0xFF, 0xE1], WallpaperFormat::JPEG),
        (vec![0x52, 0x49, 0x46, 0x46], WallpaperFormat::WEBP), // Partial RIFF header
        (vec![0x42, 0x4D], WallpaperFormat::BMP),
        (vec![0x49, 0x49, 0x2A, 0x00], WallpaperFormat::TIFF), // TIFF little-endian
        (vec![0x4D, 0x4D, 0x00, 0x2A], WallpaperFormat::TIFF), // TIFF big-endian
        (vec![0x47, 0x49, 0x46, 0x38, 0x37, 0x61], WallpaperFormat::GIF), // GIF87a
        (vec![0x47, 0x49, 0x46, 0x38, 0x39, 0x61], WallpaperFormat::GIF), // GIF89a
    ];

    for (file_data, expected_format) in test_cases {
        // Create temporary file with test data
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(temp_file.path(), &file_data).unwrap();
        
        let detected_format = detect_image_format(&temp_file.path().to_path_buf());
        
        match detected_format {
            Ok(format) => {
                assert_eq!(format, expected_format, 
                    "Format detection failed for data: {:?}", file_data);
            }
            Err(e) => {
                panic!("Format detection failed for {:?}: {}", expected_format, e);
            }
        }
    }

    // Test unknown format
    let unknown_data = vec![0x00, 0x01, 0x02, 0x03];
    let temp_file = tempfile::NamedTempFile::new().unwrap();
    std::fs::write(temp_file.path(), &unknown_data).unwrap();
    
    let result = detect_image_format(&temp_file.path().to_path_buf());
    assert!(result.is_err(), "Unknown format should return error");
}
```

#### 6. File Watching and Hot Reload Testing
**Reference**: `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs:425-458`
```rust
#[test]
fn test_file_watching_and_hot_reload() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, wallpaper_system)
       .add_event::<WallpaperEvent>()
       .add_event::<UINotificationEvent>();

    let mut wallpaper_manager = WallpaperManager::default();
    
    // Create test wallpaper file
    let test_file_path = create_test_image_file(800, 600, WallpaperFormat::PNG, 1.0);
    
    // Setup file watching
    let mut file_watcher = setup_wallpaper_file_watching(
        &test_file_path,
        &mut wallpaper_manager.file_watcher,
    );
    
    assert!(file_watcher.is_some(), "File watcher should be created");
    
    wallpaper_manager.file_watcher = file_watcher;
    wallpaper_manager.current_wallpaper = Some(WallpaperAsset {
        file_path: test_file_path.clone(),
        handle: Handle::default(),
        metadata: WallpaperMetadata::default(),
        loaded_at: Instant::now(),
        validation_status: ValidationStatus::Valid,
    });
    
    app.world_mut().insert_resource(wallpaper_manager);

    // Simulate file change event
    let mut wallpaper_events = app.world_mut().resource_mut::<Events<WallpaperEvent>>();
    wallpaper_events.send(WallpaperEvent::FileChanged {
        file_path: test_file_path.clone(),
    });

    app.update();

    // Verify reload was triggered
    let wallpaper_events = app.world().resource::<Events<WallpaperEvent>>();
    let mut event_reader = wallpaper_events.get_reader();
    let events: Vec<_> = event_reader.read(&wallpaper_events).collect();
    
    // Should contain both the original FileChanged and the new LoadFromFile events
    assert!(events.len() >= 1, "File change should trigger reload events");
    
    // Cleanup
    let _ = std::fs::remove_file(test_file_path);
}
```

### Edge Case Testing

#### 7. Memory Usage and Large File Testing
```rust
#[test]
fn test_memory_usage_and_large_files() {
    let validation_settings = ValidationSettings {
        max_file_size_mb: 50.0, // Large limit for testing
        max_dimensions: (8192, 8192),
        min_dimensions: (1, 1),
        allowed_formats: vec![WallpaperFormat::PNG, WallpaperFormat::JPEG],
        require_aspect_ratio_match: false,
        color_profile_validation: false,
    };

    // Test large but valid file
    let large_file_path = create_test_image_file(4096, 4096, WallpaperFormat::PNG, 25.0); // 25MB
    let result = validate_wallpaper_file(&large_file_path, &validation_settings);
    
    assert!(result.is_ok(), "Large valid file should pass validation");
    
    let metadata = result.unwrap();
    assert_eq!(metadata.original_dimensions, (4096, 4096));
    assert!(metadata.file_size > 20 * 1024 * 1024); // At least 20MB
    
    // Estimate memory usage
    let estimated_memory = estimate_memory_usage(&metadata);
    let expected_memory = 4096 * 4096 * 4; // RGBA at full resolution
    
    assert!(estimated_memory > 0, "Memory usage should be estimated");
    assert!(estimated_memory <= expected_memory * 2, "Memory estimate should be reasonable");

    // Test memory cleanup behavior
    let mut cache = WallpaperCache {
        cached_textures: HashMap::new(),
        cache_size_limit: 2,
        compression_enabled: true,
        preload_enabled: false,
    };

    // Add large entries that would exceed memory limits
    for i in 0..3 {
        let path = PathBuf::from(format!("/tmp/large_test_{}.png", i));
        let large_metadata = WallpaperMetadata {
            original_dimensions: (2048, 2048),
            file_size: 15 * 1024 * 1024, // 15MB each
            format: WallpaperFormat::PNG,
            color_profile: None,
            compression_ratio: 1.0,
            has_transparency: false,
        };

        let cached_texture = CachedTexture {
            handle: Handle::default(),
            compressed_data: None,
            last_accessed: Instant::now() - Duration::from_secs(i * 10),
            access_count: 1,
            memory_usage: estimate_memory_usage(&large_metadata),
        };

        cache.cached_textures.insert(path, cached_texture);
    }

    assert_eq!(cache.cached_textures.len(), 3);
    
    // Trigger cleanup
    cleanup_wallpaper_cache(&mut cache);
    
    // Should remove least recently used entries
    assert!(cache.cached_textures.len() <= cache.cache_size_limit);

    // Cleanup
    let _ = std::fs::remove_file(large_file_path);
}
```

#### 8. Error Recovery and Fallback Testing
```rust
#[test]
fn test_error_recovery_and_fallback() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins)
       .add_systems(Update, wallpaper_system)
       .add_event::<WallpaperEvent>()
       .add_event::<UINotificationEvent>();

    let wallpaper_manager = WallpaperManager::default();
    app.world_mut().insert_resource(wallpaper_manager);

    // Test loading nonexistent file
    let nonexistent_path = PathBuf::from("/nonexistent/wallpaper.png");
    let mut wallpaper_events = app.world_mut().resource_mut::<Events<WallpaperEvent>>();
    wallpaper_events.send(WallpaperEvent::LoadFromFile {
        file_path: nonexistent_path,
    });

    app.update();

    // Verify error notification was generated
    let ui_events = app.world().resource::<Events<UINotificationEvent>>();
    let mut ui_reader = ui_events.get_reader();
    let ui_events_vec: Vec<_> = ui_reader.read(&ui_events).collect();
    
    assert!(!ui_events_vec.is_empty(), "Error should generate notification");
    
    let has_error_notification = ui_events_vec.iter().any(|event| {
        matches!(event, UINotificationEvent::Warning { .. } | UINotificationEvent::Error { .. })
    });
    
    assert!(has_error_notification, "Should contain error or warning notification");

    // Verify wallpaper was not changed (should remain None or default)
    let manager = app.world().resource::<WallpaperManager>();
    assert!(manager.current_wallpaper.is_none(), "Invalid wallpaper should not be loaded");

    // Test reset to default after error
    let mut wallpaper_events = app.world_mut().resource_mut::<Events<WallpaperEvent>>();
    wallpaper_events.send(WallpaperEvent::ResetToDefault);

    app.update();

    // Verify reset success
    let ui_events = app.world().resource::<Events<UINotificationEvent>>();
    let mut ui_reader = ui_events.get_reader();
    let reset_events: Vec<_> = ui_reader.read(&ui_events).collect();
    
    let has_reset_notification = reset_events.iter().any(|event| {
        matches!(event, UINotificationEvent::Info { message, .. } 
            if message.contains("default"))
    });
    
    assert!(has_reset_notification, "Reset should generate success notification");
}
```

### Manual Testing Checklist

- [ ] Select File button opens native file picker dialog
- [ ] Valid image files load and display correctly as wallpaper
- [ ] Invalid files show clear error messages with helpful guidance
- [ ] File size and dimension limits are enforced properly  
- [ ] Aspect ratio validation works for different display ratios
- [ ] Reset button removes custom wallpaper and restores default
- [ ] Different scale modes (stretch, fit, center) render correctly
- [ ] Large wallpapers load without blocking the UI
- [ ] File changes are detected and wallpaper updates automatically
- [ ] Cache limits prevent excessive memory usage

**Bevy Examples**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:585-622`, `./docs/bevy/examples/asset_loading/hot_asset_reloading.rs:525-562`  
**Integration Points**: All wallpaper system components  
**Success Criteria**: All tests pass, sub-3s loading for large files, reliable format detection