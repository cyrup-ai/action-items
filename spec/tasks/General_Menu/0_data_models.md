# General Menu - Data Models and State Management

## Task: Implement Core Data Structures for General Settings

### File: `ui/src/settings/general/mod.rs` (new file)

Create comprehensive data models for general application settings with zero-allocation patterns and blazing-fast state management.

### Implementation Requirements

#### Core Settings Structure
```rust
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub struct GeneralSettings {
    pub startup: StartupSettings,
    pub hotkey: HotkeySettings,
    pub menu_bar: MenuBarSettings,
    pub text_size: TextSizeSettings,
    pub theme: ThemeSettings,
    pub window_mode: WindowModeSettings,
    pub favorites: FavoritesSettings,
}
```

#### Startup Configuration
- File: `ui/src/settings/general/startup.rs` (new file, line 1-45)
- Implement `StartupSettings` struct with launch_at_login boolean
- Integration with macOS login items system
- Automatic startup validation and error handling

#### Hotkey Management Data
- File: `ui/src/settings/general/hotkey.rs` (new file, line 1-89)
- Implement `HotkeySettings` with global key combination storage
- Conflict detection data structures
- Recording state management for hotkey capture

#### Theme Configuration Data
- File: `ui/src/settings/general/theme.rs` (new file, line 1-67)
- Implement `ThemeSettings` enum for Dark/Light/System themes
- Custom theme support data structures
- Theme asset management and caching

#### Window Mode Data
- File: `ui/src/settings/general/window_mode.rs` (new file, line 1-34)
- Implement `WindowModeSettings` enum for Default/Compact modes
- Favorites visibility configuration
- Mode-specific UI state management

### Architecture Notes
- Use Bevy's `Reflect` trait for all settings structures
- Implement `Resource` trait for global settings access
- Zero-allocation serialization with `serde`
- Atomic state updates with change detection

### Integration Points
- `core/src/` - Core settings persistence system (line 156-189)
- `app/src/preferences/` - Existing preference management integration
- System APIs for login items and theme detection

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Bevy Implementation Details

### Component Architecture

```rust
// Component definitions with Reflect support for inspector debugging
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct GeneralSettingsPanel {
    pub visible: bool,
    pub selected_tab: GeneralTab,
    pub dirty: bool, // Track unsaved changes
}

#[derive(Component, Reflect, Debug, Clone, PartialEq)]
#[reflect(Component)]
pub enum GeneralTab {
    Startup,
    Hotkey,
    MenuBar,
    TextSize,
    Theme,
    WindowMode,
    Favorites,
}

// Settings components for individual subsections
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct StartupSettingsComponent {
    pub launch_at_login: bool,
    pub validation_state: ValidationState,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct HotkeySettingsComponent {
    pub current_hotkey: Option<KeyCombination>,
    pub is_recording: bool,
    pub conflict_detected: bool,
    pub recording_state: HotkeyRecordingState,
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ThemeSettingsComponent {
    pub current_theme: ThemeMode,
    pub system_theme_detected: ThemeMode,
    pub theme_transition_progress: f32, // For smooth theme transitions
}

#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct WindowModeSettingsComponent {
    pub current_mode: WindowMode,
    pub favorites_visible: bool,
    pub transition_progress: f32, // For window resize animation
}
```

### Resource Management

```rust
// Global settings resource with change detection
#[derive(Resource, Reflect, Debug, Clone)]
#[reflect(Resource)]
pub struct GeneralSettingsResource {
    pub settings: GeneralSettings,
    pub previous_settings: GeneralSettings, // For rollback functionality
    pub auto_save_timer: Timer,
    pub validation_cache: HashMap<String, ValidationResult>,
}

impl Default for GeneralSettingsResource {
    fn default() -> Self {
        Self {
            settings: GeneralSettings::default(),
            previous_settings: GeneralSettings::default(),
            auto_save_timer: Timer::from_seconds(2.0, TimerMode::Repeating),
            validation_cache: HashMap::new(),
        }
    }
}

// Settings persistence resource
#[derive(Resource, Reflect)]
#[reflect(Resource)]
pub struct SettingsPersistence {
    pub save_path: PathBuf,
    pub pending_save: bool,
    pub last_save_time: Instant,
    pub save_task: Option<Task<Result<(), SettingsError>>>,
}
```

### Event System

```rust
// Settings change events for reactive updates
#[derive(Event, Reflect, Debug, Clone)]
pub enum GeneralSettingsEvent {
    TabChanged(GeneralTab),
    SettingChanged {
        setting_path: String,
        old_value: SettingValue,
        new_value: SettingValue,
    },
    SaveRequested,
    SaveCompleted(Result<(), SettingsError>),
    ResetToDefaults,
    ValidationFailed {
        setting_path: String,
        error: ValidationError,
    },
}

#[derive(Event, Reflect, Debug)]
pub struct HotkeyRecordingEvent {
    pub state: HotkeyRecordingState,
    pub captured_keys: Vec<KeyCode>,
}

#[derive(Event, Reflect, Debug)]
pub struct ThemeChangeEvent {
    pub from_theme: ThemeMode,
    pub to_theme: ThemeMode,
}
```

### System Organization with SystemSets

```rust
// System sets for proper ordering
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GeneralSettingsSystemSet {
    Input,          // Handle user input
    Validation,     // Validate setting changes
    StateUpdate,    // Update internal state
    Persistence,    // Save to disk
    UIUpdate,       // Update UI elements
}

// System registration with proper ordering
impl Plugin for GeneralSettingsPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<GeneralSettingsResource>()
            .init_resource::<SettingsPersistence>()
            
            // Events
            .add_event::<GeneralSettingsEvent>()
            .add_event::<HotkeyRecordingEvent>()
            .add_event::<ThemeChangeEvent>()
            
            // Component registration for Reflect
            .register_type::<GeneralSettingsPanel>()
            .register_type::<StartupSettingsComponent>()
            .register_type::<HotkeySettingsComponent>()
            .register_type::<ThemeSettingsComponent>()
            .register_type::<WindowModeSettingsComponent>()
            
            // System sets configuration
            .configure_sets(
                Update,
                (
                    GeneralSettingsSystemSet::Input,
                    GeneralSettingsSystemSet::Validation,
                    GeneralSettingsSystemSet::StateUpdate,
                    GeneralSettingsSystemSet::Persistence,
                    GeneralSettingsSystemSet::UIUpdate,
                ).chain()
            )
            
            // Systems
            .add_systems(Update, (
                handle_settings_input.in_set(GeneralSettingsSystemSet::Input),
                validate_settings_changes.in_set(GeneralSettingsSystemSet::Validation),
                update_settings_state.in_set(GeneralSettingsSystemSet::StateUpdate),
                persist_settings_async.in_set(GeneralSettingsSystemSet::Persistence),
                update_settings_ui.in_set(GeneralSettingsSystemSet::UIUpdate),
            ));
    }
}
```

### Async Settings Persistence

```rust
// System for async settings persistence using AsyncComputeTaskPool
fn persist_settings_async(
    mut settings: ResMut<GeneralSettingsResource>,
    mut persistence: ResMut<SettingsPersistence>,
    mut events: EventWriter<GeneralSettingsEvent>,
    time: Res<Time>,
) {
    // Update auto-save timer
    settings.auto_save_timer.tick(time.delta());
    
    // Check if we need to save
    let should_save = settings.auto_save_timer.just_finished() 
        && persistence.pending_save;
    
    if should_save {
        let settings_data = settings.settings.clone();
        let save_path = persistence.save_path.clone();
        
        // Spawn async save task
        let task_pool = AsyncComputeTaskPool::get();
        let task = task_pool.spawn(async move {
            let serialized = serde_json::to_string_pretty(&settings_data)?;
            tokio::fs::write(save_path, serialized).await?;
            Ok(())
        });
        
        persistence.save_task = Some(task);
        persistence.pending_save = false;
    }
    
    // Poll existing save task
    if let Some(mut task) = persistence.save_task.take() {
        if let Some(result) = block_on(future::poll_once(&mut task)) {
            events.send(GeneralSettingsEvent::SaveCompleted(result));
            persistence.last_save_time = Instant::now();
        } else {
            persistence.save_task = Some(task);
        }
    }
}
```

### Query Optimization with Change Detection

```rust
// Optimized system using Changed<T> for performance
fn update_settings_ui(
    // Only query changed settings components
    changed_panels: Query<&GeneralSettingsPanel, Changed<GeneralSettingsPanel>>,
    changed_startup: Query<&StartupSettingsComponent, Changed<StartupSettingsComponent>>,
    changed_hotkey: Query<&HotkeySettingsComponent, Changed<HotkeySettingsComponent>>,
    changed_theme: Query<&ThemeSettingsComponent, Changed<ThemeSettingsComponent>>,
    
    // UI node queries for updates
    mut ui_nodes: Query<(&mut BackgroundColor, &mut BorderColor), With<SettingsUINode>>,
    mut text_nodes: Query<&mut Text, With<SettingsText>>,
    
    settings_resource: Res<GeneralSettingsResource>,
) {
    // Only update UI when components have actually changed
    if !changed_panels.is_empty() || !changed_startup.is_empty() 
        || !changed_hotkey.is_empty() || !changed_theme.is_empty() {
        
        // Update UI elements based on changed components
        for (mut bg_color, mut border_color) in ui_nodes.iter_mut() {
            // Update colors, animations, etc.
        }
        
        for mut text in text_nodes.iter_mut() {
            // Update text content based on settings
        }
    }
}
```

### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::ecs::system::SystemState;
    
    #[test]
    fn test_settings_component_serialization() {
        let settings = GeneralSettings::default();
        let serialized = serde_json::to_string(&settings).unwrap();
        let deserialized: GeneralSettings = serde_json::from_str(&serialized).unwrap();
        assert_eq!(settings, deserialized);
    }
    
    #[test]
    fn test_settings_validation() {
        let mut app = App::new();
        app.add_plugins(GeneralSettingsPlugin);
        
        let world = app.world_mut();
        let mut system_state = SystemState::<ResMut<GeneralSettingsResource>>::new(world);
        let mut settings = system_state.get_mut(world);
        
        // Test validation logic
        settings.settings.hotkey.global_hotkey = Some(KeyCombination::new());
        // Validation assertions...
    }
    
    #test]
    fn test_async_persistence() {
        let mut app = App::new();
        app.add_plugins(GeneralSettingsPlugin);
        
        // Test async save/load functionality
        // Mock file system operations
        // Verify data integrity
    }
}
```

This implementation provides a complete Bevy-native data model architecture with ECS components, resources, events, and systems that follow modern Bevy patterns for optimal performance and maintainability.