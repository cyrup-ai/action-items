# General Menu - Theme Selection System

## Task: Implement Dynamic Theme Selection and Management

### File: `ui/src/settings/general/theme_selector.rs` (new file)

Create comprehensive theme selection interface with dynamic switching, custom theme support, and system appearance integration.

### Implementation Requirements

#### Theme Selection Component
```rust
#[derive(Component)]
pub struct ThemeSelector {
    pub current_theme: ThemeType,
    pub available_themes: Vec<ThemeDefinition>,
    pub follow_system: bool,
    pub custom_theme_path: Option<PathBuf>,
}
```

#### Theme Management System
- File: `ui/src/settings/general/theme_selector.rs` (line 1-123)
- Implement dropdown interface for theme selection
- Theme preview generation and caching
- Dynamic theme asset loading and management
- System appearance detection and monitoring

#### Theme Definition Architecture
- File: `ui/src/settings/general/theme_definition.rs` (new file, line 1-89)
- Implement `ThemeDefinition` struct with color palettes
- Asset path management for theme resources
- Theme validation and integrity checking
- Version compatibility for theme files

#### System Integration
- File: `ui/src/settings/general/system_theme.rs` (new file, line 1-67)
- macOS Dark Mode detection integration
- Automatic theme switching based on system changes
- System appearance change event handling
- Fallback theme management for system integration failures

#### Custom Theme Support
- File: `ui/src/settings/general/custom_themes.rs` (new file, line 1-134)
- Theme Studio integration button implementation
- Custom theme file loading and validation
- Theme import/export functionality
- Community theme support infrastructure

### Architecture Notes
- Integration with existing theme system in codebase
- Asset hot-reloading for theme development
- Zero-allocation theme switching
- Integration with `ui/src/ui/theme.rs` (existing theme system)

### Integration Points
- Theme Studio external application launch
- Asset loading system for theme resources
- Settings persistence for theme preferences
- UI component style update system

### Visual Requirements
- Dropdown with theme names and preview icons
- "Follow system appearance" checkbox integration
- "Open Theme Studio" button with proper styling
- Real-time theme preview during selection

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Theme Selection Components Architecture

```rust
// Core theme selection component with Reflect support
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ThemeSelector {
    pub current_theme: ThemeType,
    pub available_themes: Vec<ThemeDefinition>,
    pub follow_system: bool,
    pub custom_theme_path: Option<PathBuf>,
    pub preview_entity: Option<Entity>,
    pub dropdown_open: bool,
    pub theme_transition_progress: f32,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ThemePreviewComponent {
    pub theme_definition: ThemeDefinition,
    pub preview_rendered: bool,
    pub preview_texture: Option<Handle<Image>>,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct SystemThemeMonitor {
    pub current_system_theme: SystemThemeType,
    pub monitoring_active: bool,
    pub last_check_time: Instant,
    pub check_interval: Duration,
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum ThemeType {
    Light,
    Dark,
    System,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Reflect)]
pub enum SystemThemeType {
    Light,
    Dark,
    Unknown,
}
```

### Theme Management Resource System

```rust
// Global theme management resource
#[derive(Resource, Reflect, Debug)]
#[reflect(Resource)]
pub struct ThemeManager {
    pub current_theme: ThemeDefinition,
    pub available_themes: HashMap<String, ThemeDefinition>,
    pub system_theme_detected: SystemThemeType,
    pub theme_cache: HashMap<String, Handle<Image>>,
    pub custom_theme_directory: PathBuf,
    pub theme_loading_tasks: Vec<Task<ThemeLoadResult>>,
}

impl Default for ThemeManager {
    fn default() -> Self {
        Self {
            current_theme: ThemeDefinition::default_dark(),
            available_themes: Self::load_builtin_themes(),
            system_theme_detected: SystemThemeType::Unknown,
            theme_cache: HashMap::new(),
            custom_theme_directory: PathBuf::from("./themes/custom"),
            theme_loading_tasks: Vec::new(),
        }
    }
}

impl ThemeManager {
    fn load_builtin_themes() -> HashMap<String, ThemeDefinition> {
        let mut themes = HashMap::new();
        themes.insert("Light".to_string(), ThemeDefinition::default_light());
        themes.insert("Dark".to_string(), ThemeDefinition::default_dark());
        themes.insert("High Contrast".to_string(), ThemeDefinition::high_contrast());
        themes
    }
}

// Theme definition with comprehensive styling
#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ThemeDefinition {
    pub name: String,
    pub version: String,
    pub author: String,
    pub colors: ThemeColors,
    pub fonts: ThemeFonts,
    pub assets: ThemeAssets,
    pub metadata: ThemeMetadata,
}

#[derive(Debug, Clone, PartialEq, Reflect, Serialize, Deserialize)]
pub struct ThemeColors {
    pub background_primary: Color,
    pub background_secondary: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub accent_primary: Color,
    pub accent_secondary: Color,
    pub border_primary: Color,
    pub border_secondary: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
}
```

### Theme Selection System Sets

```rust
// System sets for theme management
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum ThemeSystemSet {
    MonitorSystem,        // Monitor system theme changes
    LoadThemes,           // Load and validate themes
    UpdateSelection,      // Handle theme selection changes
    ApplyTheme,          // Apply theme to UI components
    GeneratePreview,     // Generate theme previews
    PersistSettings,     // Save theme preferences
}

// Theme management plugin
pub struct ThemeManagementPlugin;

impl Plugin for ThemeManagementPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ThemeManager>()
            .init_resource::<SystemThemeDetector>()
            
            // Events
            .add_event::<ThemeChangedEvent>()
            .add_event::<SystemThemeChangedEvent>()
            .add_event::<CustomThemeLoadedEvent>()
            
            // Component registration
            .register_type::<ThemeSelector>()
            .register_type::<ThemePreviewComponent>()
            .register_type::<SystemThemeMonitor>()
            .register_type::<ThemeDefinition>()
            
            // System sets configuration
            .configure_sets(
                Update,
                (
                    ThemeSystemSet::MonitorSystem,
                    ThemeSystemSet::LoadThemes,
                    ThemeSystemSet::UpdateSelection,
                    ThemeSystemSet::ApplyTheme,
                    ThemeSystemSet::GeneratePreview,
                    ThemeSystemSet::PersistSettings,
                ).chain()
            )
            
            // Theme management systems
            .add_systems(Update, (
                monitor_system_theme.in_set(ThemeSystemSet::MonitorSystem),
                load_custom_themes_async.in_set(ThemeSystemSet::LoadThemes),
                handle_theme_selection.in_set(ThemeSystemSet::UpdateSelection),
                apply_theme_to_components.in_set(ThemeSystemSet::ApplyTheme),
                generate_theme_previews.in_set(ThemeSystemSet::GeneratePreview),
                persist_theme_settings.in_set(ThemeSystemSet::PersistSettings),
            ))
            
            // Startup systems
            .add_systems(Startup, (
                setup_theme_system,
                detect_initial_system_theme,
                load_saved_theme_preference,
            ));
    }
}
```

### System Theme Monitoring

```rust
// System to monitor macOS system theme changes
fn monitor_system_theme(
    mut theme_monitor: Query<&mut SystemThemeMonitor>,
    mut theme_manager: ResMut<ThemeManager>,
    mut theme_selector: Query<&mut ThemeSelector>,
    mut events: EventWriter<SystemThemeChangedEvent>,
    time: Res<Time>,
) {
    for mut monitor in theme_monitor.iter_mut() {
        if !monitor.monitoring_active {
            continue;
        }
        
        // Check if it's time to poll system theme
        if time.elapsed() - monitor.last_check_time > monitor.check_interval {
            monitor.last_check_time = time.elapsed();
            
            // Detect current system theme (platform-specific)
            let detected_theme = detect_system_theme();
            
            if detected_theme != monitor.current_system_theme {
                monitor.current_system_theme = detected_theme.clone();
                theme_manager.system_theme_detected = detected_theme.clone();
                
                // Send system theme change event
                events.send(SystemThemeChangedEvent {
                    new_theme: detected_theme.clone(),
                    timestamp: time.elapsed(),
                });
                
                // Update theme selectors that follow system
                for mut selector in theme_selector.iter_mut() {
                    if selector.follow_system {
                        let new_theme_type = match detected_theme {
                            SystemThemeType::Light => ThemeType::Light,
                            SystemThemeType::Dark => ThemeType::Dark,
                            SystemThemeType::Unknown => ThemeType::Dark, // Fallback
                        };
                        
                        if selector.current_theme != new_theme_type {
                            selector.current_theme = new_theme_type;
                            selector.theme_transition_progress = 0.0; // Start transition
                        }
                    }
                }
            }
        }
    }
}

// Platform-specific system theme detection using objc2
#[cfg(target_os = "macos")]
fn detect_system_theme() -> SystemThemeType {
    use objc2::msg_send;
    use objc2_app_kit::NSApplication;
    use objc2_foundation::NSString;
    
    unsafe {
        let app_class = objc2::class!(NSApplication);
        let app: *const NSApplication = msg_send![app_class, sharedApplication];
        let appearance: *const objc2::runtime::AnyObject = msg_send![app, effectiveAppearance];
        let name: *const NSString = msg_send![appearance, name];
        
        let theme_str = (*name).to_string();
        
        if theme_str.contains("Dark") {
            SystemThemeType::Dark
        } else if theme_str.contains("Light") {
            SystemThemeType::Light
        } else {
            SystemThemeType::Unknown
        }
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_system_theme() -> SystemThemeType {
    SystemThemeType::Unknown
}:{msg_send, sel, sel_impl};\n    \n    unsafe {\n        let pool = NSAutoreleasePool::new(nil);\n        \n        let appearance: id = msg_send![class!(NSApplication), sharedApplication];\n        let effective_appearance: id = msg_send![appearance, effectiveAppearance];\n        let name: id = msg_send![effective_appearance, name];\n        \n        let theme_name = NSString::UTF8String(name);\n        let theme_str = std::ffi::CStr::from_ptr(theme_name).to_string_lossy();\n        \n        pool.drain();\n        \n        if theme_str.contains(\"Dark\") {\n            SystemThemeType::Dark\n        } else if theme_str.contains(\"Light\") {\n            SystemThemeType::Light\n        } else {\n            SystemThemeType::Unknown\n        }\n    }\n}\n\n#[cfg(not(target_os = \"macos\"))]\nfn detect_system_theme() -> SystemThemeType {\n    SystemThemeType::Unknown\n}
```

### Async Theme Loading System

```rust
// System for async theme loading using AsyncComputeTaskPool
fn load_custom_themes_async(\n    mut theme_manager: ResMut<ThemeManager>,\n    mut events: EventWriter<CustomThemeLoadedEvent>,\n) {\n    // Poll existing loading tasks\n    theme_manager.theme_loading_tasks.retain_mut(|task| {\n        if let Some(result) = block_on(future::poll_once(task)) {\n            match result {\n                Ok(theme_load_result) => {\n                    // Add loaded theme to available themes\n                    theme_manager.available_themes.insert(\n                        theme_load_result.theme.name.clone(),\n                        theme_load_result.theme.clone()\n                    );\n                    \n                    events.send(CustomThemeLoadedEvent {\n                        theme_name: theme_load_result.theme.name.clone(),\n                        load_time: theme_load_result.load_time,\n                    });\n                }\n                Err(e) => {\n                    error!(\"Failed to load custom theme: {}\", e);\n                }\n            }\n            false // Remove completed task\n        } else {\n            true // Keep pending task\n        }\n    });\n    \n    // Spawn new loading tasks if needed\n    let custom_dir = theme_manager.custom_theme_directory.clone();\n    if theme_manager.theme_loading_tasks.is_empty() {\n        let task_pool = AsyncComputeTaskPool::get();\n        let task = task_pool.spawn(async move {\n            scan_and_load_custom_themes(custom_dir).await\n        });\n        theme_manager.theme_loading_tasks.push(task);\n    }\n}\n\n// Async function to scan and load custom themes\nasync fn scan_and_load_custom_themes(theme_dir: PathBuf) -> Result<ThemeLoadResult, ThemeError> {\n    let start_time = Instant::now();\n    \n    if !theme_dir.exists() {\n        tokio::fs::create_dir_all(&theme_dir).await\n            .map_err(|e| ThemeError::DirectoryCreation(e.to_string()))?;\n    }\n    \n    let mut dir_entries = tokio::fs::read_dir(&theme_dir).await\n        .map_err(|e| ThemeError::DirectoryAccess(e.to_string()))?;\n    \n    let mut loaded_themes = Vec::new();\n    \n    while let Some(entry) = dir_entries.next_entry().await\n        .map_err(|e| ThemeError::DirectoryAccess(e.to_string()))? {\n        \n        let path = entry.path();\n        if path.extension().map_or(false, |ext| ext == \"json\") {\n            match load_theme_from_file(&path).await {\n                Ok(theme) => loaded_themes.push(theme),\n                Err(e) => warn!(\"Failed to load theme from {:?}: {}\", path, e),\n            }\n        }\n    }\n    \n    // For now, return the first theme or a default\n    let theme = loaded_themes.into_iter().next()\n        .unwrap_or_else(|| ThemeDefinition::default_dark());\n    \n    Ok(ThemeLoadResult {\n        theme,\n        load_time: start_time.elapsed(),\n    })\n}\n\n// Load theme definition from file\nasync fn load_theme_from_file(path: &Path) -> Result<ThemeDefinition, ThemeError> {\n    let content = tokio::fs::read_to_string(path).await\n        .map_err(|e| ThemeError::FileRead(e.to_string()))?;\n    \n    let theme: ThemeDefinition = serde_json::from_str(&content)\n        .map_err(|e| ThemeError::ParseError(e.to_string()))?;\n    \n    // Validate theme definition\n    validate_theme_definition(&theme)?;\n    \n    Ok(theme)\n}\n\n// Validate theme definition integrity\nfn validate_theme_definition(theme: &ThemeDefinition) -> Result<(), ThemeError> {\n    if theme.name.is_empty() {\n        return Err(ThemeError::InvalidDefinition(\"Theme name cannot be empty\".to_string()));\n    }\n    \n    if theme.version.is_empty() {\n        return Err(ThemeError::InvalidDefinition(\"Theme version cannot be empty\".to_string()));\n    }\n    \n    // Validate color values are reasonable\n    let colors = &theme.colors;\n    if colors.background_primary.alpha() < 0.0 || colors.background_primary.alpha() > 1.0 {\n        return Err(ThemeError::InvalidDefinition(\"Invalid alpha values in colors\".to_string()));\n    }\n    \n    Ok(())\n}
```

### Theme Application System with Smooth Transitions

```rust
// System to apply theme changes with smooth transitions
fn apply_theme_to_components(\n    mut theme_selector: Query<&mut ThemeSelector, Changed<ThemeSelector>>,\n    mut theme_manager: ResMut<ThemeManager>,\n    mut background_query: Query<&mut BackgroundColor>,\n    mut text_query: Query<&mut TextColor>,\n    mut border_query: Query<&mut BorderColor>,\n    time: Res<Time>,\n    mut events: EventWriter<ThemeChangedEvent>,\n) {\n    for mut selector in theme_selector.iter_mut() {\n        // Update transition progress\n        if selector.theme_transition_progress < 1.0 {\n            selector.theme_transition_progress += time.delta_seconds() * 3.0; // Transition speed\n            selector.theme_transition_progress = selector.theme_transition_progress.min(1.0);\n            \n            // Get target theme definition\n            let target_theme = match &selector.current_theme {\n                ThemeType::Light => theme_manager.available_themes.get(\"Light\"),\n                ThemeType::Dark => theme_manager.available_themes.get(\"Dark\"),\n                ThemeType::System => {\n                    match theme_manager.system_theme_detected {\n                        SystemThemeType::Light => theme_manager.available_themes.get(\"Light\"),\n                        SystemThemeType::Dark => theme_manager.available_themes.get(\"Dark\"),\n                        SystemThemeType::Unknown => theme_manager.available_themes.get(\"Dark\"),\n                    }\n                }\n                ThemeType::Custom(name) => theme_manager.available_themes.get(name),\n            };\n            \n            if let Some(target_theme) = target_theme {\n                // Smoothly transition to new theme\n                let current_theme = &theme_manager.current_theme;\n                let transition_theme = interpolate_themes(current_theme, target_theme, selector.theme_transition_progress);\n                \n                // Apply interpolated theme to components\n                apply_theme_to_ui_components(\n                    &transition_theme,\n                    &mut background_query,\n                    &mut text_query,\n                    &mut border_query\n                );\n                \n                // Update current theme when transition is complete\n                if selector.theme_transition_progress >= 1.0 {\n                    theme_manager.current_theme = target_theme.clone();\n                    \n                    events.send(ThemeChangedEvent {\n                        previous_theme: current_theme.name.clone(),\n                        new_theme: target_theme.name.clone(),\n                        transition_duration: Duration::from_secs_f32(1.0 / 3.0), // Based on transition speed\n                    });\n                }\n            }\n        }\n    }\n}\n\n// Interpolate between two themes for smooth transitions\nfn interpolate_themes(from: &ThemeDefinition, to: &ThemeDefinition, progress: f32) -> ThemeDefinition {\n    let t = progress.clamp(0.0, 1.0);\n    \n    ThemeDefinition {\n        name: format!(\"{} -> {} ({}%)\", from.name, to.name, (progress * 100.0) as u32),\n        version: to.version.clone(),\n        author: to.author.clone(),\n        colors: ThemeColors {\n            background_primary: lerp_color(from.colors.background_primary, to.colors.background_primary, t),\n            background_secondary: lerp_color(from.colors.background_secondary, to.colors.background_secondary, t),\n            text_primary: lerp_color(from.colors.text_primary, to.colors.text_primary, t),\n            text_secondary: lerp_color(from.colors.text_secondary, to.colors.text_secondary, t),\n            accent_primary: lerp_color(from.colors.accent_primary, to.colors.accent_primary, t),\n            accent_secondary: lerp_color(from.colors.accent_secondary, to.colors.accent_secondary, t),\n            border_primary: lerp_color(from.colors.border_primary, to.colors.border_primary, t),\n            border_secondary: lerp_color(from.colors.border_secondary, to.colors.border_secondary, t),\n            success: lerp_color(from.colors.success, to.colors.success, t),\n            warning: lerp_color(from.colors.warning, to.colors.warning, t),\n            error: lerp_color(from.colors.error, to.colors.error, t),\n        },\n        fonts: to.fonts.clone(), // Fonts don't interpolate\n        assets: to.assets.clone(), // Assets don't interpolate\n        metadata: to.metadata.clone(),\n    }\n}\n\n// Linear interpolation for colors\nfn lerp_color(from: Color, to: Color, t: f32) -> Color {\n    let from_rgba = from.to_srgba();\n    let to_rgba = to.to_srgba();\n    \n    Color::srgba(\n        from_rgba.red + (to_rgba.red - from_rgba.red) * t,\n        from_rgba.green + (to_rgba.green - from_rgba.green) * t,\n        from_rgba.blue + (to_rgba.blue - from_rgba.blue) * t,\n        from_rgba.alpha + (to_rgba.alpha - from_rgba.alpha) * t,\n    )\n}\n\n// Apply theme to UI components\nfn apply_theme_to_ui_components(\n    theme: &ThemeDefinition,\n    background_query: &mut Query<&mut BackgroundColor>,\n    text_query: &mut Query<&mut TextColor>,\n    border_query: &mut Query<&mut BorderColor>,\n) {\n    // Update background colors\n    for mut bg_color in background_query.iter_mut() {\n        *bg_color = BackgroundColor(theme.colors.background_primary);\n    }\n    \n    // Update text colors\n    for mut text_color in text_query.iter_mut() {\n        *text_color = TextColor(theme.colors.text_primary);\n    }\n    \n    // Update border colors\n    for mut border_color in border_query.iter_mut() {\n        *border_color = BorderColor(theme.colors.border_primary);\n    }\n}
```

### Event System for Theme Management

```rust
// Events for theme system communication
#[derive(Event, Debug, Clone)]
pub struct ThemeChangedEvent {\n    pub previous_theme: String,\n    pub new_theme: String,\n    pub transition_duration: Duration,\n}\n\n#[derive(Event, Debug, Clone)]\npub struct SystemThemeChangedEvent {\n    pub new_theme: SystemThemeType,\n    pub timestamp: Duration,\n}\n\n#[derive(Event, Debug, Clone)]\npub struct CustomThemeLoadedEvent {\n    pub theme_name: String,\n    pub load_time: Duration,\n}\n\n// Error types for theme operations\n#[derive(Debug, Clone)]\npub enum ThemeError {\n    DirectoryCreation(String),\n    DirectoryAccess(String),\n    FileRead(String),\n    ParseError(String),\n    InvalidDefinition(String),\n    AssetLoadError(String),\n}\n\nimpl std::fmt::Display for ThemeError {\n    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {\n        match self {\n            ThemeError::DirectoryCreation(msg) => write!(f, \"Directory creation error: {}\", msg),\n            ThemeError::DirectoryAccess(msg) => write!(f, \"Directory access error: {}\", msg),\n            ThemeError::FileRead(msg) => write!(f, \"File read error: {}\", msg),\n            ThemeError::ParseError(msg) => write!(f, \"Parse error: {}\", msg),\n            ThemeError::InvalidDefinition(msg) => write!(f, \"Invalid theme definition: {}\", msg),\n            ThemeError::AssetLoadError(msg) => write!(f, \"Asset load error: {}\", msg),\n        }\n    }\n}\n\nimpl std::error::Error for ThemeError {}\n\n// Result types for async operations\n#[derive(Debug, Clone)]\npub struct ThemeLoadResult {\n    pub theme: ThemeDefinition,\n    pub load_time: Duration,\n}
```

This comprehensive theme selection system provides dynamic theme switching, system appearance integration, custom theme support, and smooth transitions using Bevy's ECS architecture with async loading and proper resource management.