# Advanced Menu 2 Specification

## Overview
Advanced Menu 2 represents additional advanced configuration options focusing on system branding, accessibility features, data management, and developer tools. This interface exposes specialized settings for favicon management, emoji customization, data portability, and window capture functionality.

## Layout Architecture
- **Base Layout**: Advanced tab active in primary navigation
- **Vertical Configuration Sections**: Logically grouped specialized settings
- **Mixed Control Types**: Dropdowns, emoji selectors, button groups, and action buttons
- **Info Integration**: Contextual help icons for complex specialized features

## Configuration Sections

### Advanced Input Features (Continued)

#### Hyper Key Configuration
- **Current Setting**: "–" (disabled/not configured)
- **Control Type**: Dropdown for key assignment
- **Purpose**: Hyper key (super-modifier) configuration for power users
- **Options**:
  - Disabled (–)
  - Caps Lock remapping
  - Right Option key
  - Right Command key
  - Function key assignment
  - Custom key combinations
- **Info Icon**: Explains hyper key functionality and system integration requirements

#### Advanced Text Replacement
- **Setting**: "Replace occurrences of ^⌃⇧⌘ with ⌃"
- **Control Type**: Checkbox toggle
- **Current State**: Enabled (checked)
- **Purpose**: Automatic text replacement for complex modifier notation
- **Functionality**: 
  - Converts verbose modifier combinations to simplified notation
  - Improves readability in documentation and help text
  - Customizable replacement rules for keyboard shortcuts

### System Integration Features

#### Favicon Provider Configuration
- **Setting**: "Favicon Provider"
- **Current Selection**: "Raycast"
- **Control Type**: Dropdown with provider options
- **Purpose**: Source selection for website favicon retrieval
- **Provider Options**:
  - Raycast (internal favicon service)
  - Google Favicon API
  - Custom favicon service
  - Local favicon cache
- **Info Icon**: Details about favicon service performance and privacy implications

### Accessibility and Personalization

#### Emoji Skin Tone Selection
- **Setting**: "Emoji Skin Tone"
- **Control Type**: Visual emoji selector with skin tone options
- **Current Selection**: Medium skin tone (third option highlighted)
- **Available Options**: Six skin tone variations from light to dark
- **Visual Display**: 👋 emoji in different skin tones
- **Purpose**: Personalize emoji appearance across the application interface
- **Accessibility**: Inclusive representation and user preference respect

### Data Management and Portability

#### Import/Export System
- **Section Title**: "Import / Export"
- **Purpose**: Comprehensive data backup and migration capabilities
- **Description**: "Exporting will back-up your settings, quicklinks, snippets, notes, script-command folder paths, aliases, hotkeys, favorites, custom window management commands and other data."

##### Export/Import Controls
- **Import Button**: 
  - **Icon**: Download arrow pointing down
  - **Functionality**: Import previously exported configuration data
  - **File Formats**: Support for JSON, ZIP, or proprietary formats
  - **Validation**: Comprehensive validation of imported data integrity

- **Export Button**:
  - **Icon**: Upload arrow pointing up  
  - **Functionality**: Export current configuration for backup or transfer
  - **Scope**: Complete application state and user data
  - **Format**: Structured export with version compatibility

##### Advanced Export Features
- **Configure Export Schedule Button**: 
  - **Purpose**: Automated backup scheduling configuration
  - **Features**: Recurring export schedules, cloud storage integration
  - **Pro Feature**: Advanced scheduling options for Pro users
  - **Flexibility**: Custom schedule intervals and export destinations

- **Info Icon**: Detailed explanation of export scope and data handling

### Developer and Advanced Tools

#### Window Capture System
- **Setting**: "Window Capture"
- **Purpose**: Screenshot and sharing functionality for developers
- **Description**: "Capture the Raycast window to share it or add a screenshot of your extension to the Store."
- **Use Cases**:
  - Extension development and documentation
  - Bug reporting and support
  - Marketing and promotional materials
  - Development workflow integration

##### Window Capture Controls
- **Record Hotkey Button**:
  - **Functionality**: Assign custom hotkey for window capture
  - **Integration**: System-wide hotkey registration
  - **Conflict Detection**: Automatic detection of hotkey conflicts
  - **Customization**: User-defined capture triggers

## Functional Requirements

### Advanced Input Management
- **Hyper Key Integration**: Deep system integration for hyper key functionality
- **Text Replacement Engine**: Sophisticated pattern matching and replacement system
- **Accessibility Support**: Full accessibility integration for specialized input methods
- **Performance Optimization**: Minimal impact on system performance for input processing

### Favicon Management System
- **Provider Abstraction**: Flexible provider system for favicon retrieval
- **Caching Strategy**: Intelligent caching for performance optimization  
- **Fallback Mechanisms**: Graceful fallback for unavailable favicons
- **Privacy Controls**: User control over external favicon service usage

### Emoji Personalization Framework
- **Skin Tone Persistence**: Reliable storage and application of skin tone preferences
- **Unicode Compliance**: Full Unicode emoji standard compliance
- **Accessibility Integration**: Screen reader and accessibility support for emoji
- **Cultural Sensitivity**: Respectful and inclusive emoji representation

### Data Portability Architecture
- **Comprehensive Export**: Complete application state export capability
- **Versioned Import**: Backward and forward compatibility for data migration
- **Selective Export**: Granular control over exported data categories
- **Scheduled Automation**: Automated backup and export scheduling

## Bevy Implementation Examples

### Hyper Key Configuration
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Advanced keyboard event capture
- Reference: `./docs/bevy/examples/ui/ui.rs` - Dropdown configuration interface

### Emoji Skin Tone Selector
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - Visual emoji selection grid
- Reference: `./docs/bevy/examples/input/mouse_input.rs` - Emoji selection interaction

### Import/Export Interface
- Reference: `./docs/bevy/examples/ui/button.rs` - Import/export button styling
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Background export operations

### Text Replacement System
- Reference: `./docs/bevy/examples/ui/text_input.rs` - Text replacement configuration
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Real-time text processing

### Window Capture System
- Reference: `./docs/bevy/examples/window/screenshot.rs` - Window capture functionality
- Reference: `./docs/bevy/examples/input/keyboard_input.rs` - Hotkey assignment for capture

### Favicon Provider Management
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Dynamic favicon loading
- Reference: `./docs/bevy/examples/ui/ui.rs` - Provider selection interface

### Scheduled Export Configuration
- Reference: `./docs/bevy/examples/time/time.rs` - Export scheduling system
- Reference: `./docs/bevy/examples/ui/ui.rs` - Schedule configuration interface

## State Management Requirements

### Input Method State Tracking
- **Hyper Key Registration**: System-level hyper key state monitoring
- **Text Replacement Rules**: Dynamic text replacement rule management
- **Input Performance**: Real-time input processing performance monitoring
- **Accessibility State**: Integration with system accessibility services

### Personalization State Management
- **Emoji Preferences**: Persistent storage of emoji and skin tone preferences
- **Favicon Cache**: Intelligent favicon cache management and invalidation
- **UI Customization**: Real-time application of personalization preferences
- **Cross-Device Sync**: Optional synchronization of personalization settings

### Data Export State Coordination
- **Export Progress**: Real-time export operation progress tracking
- **Schedule Management**: Export schedule state and execution tracking
- **Import Validation**: Comprehensive import data validation and error handling
- **Backup Integrity**: Continuous validation of backup data integrity

## Security and Privacy Framework

### Data Export Security
- **Export Encryption**: Optional encryption for exported data
- **Access Control**: User authentication for sensitive export operations
- **Data Minimization**: User control over exported data scope and sensitivity
- **Audit Logging**: Comprehensive logging of export and import operations

### Privacy Protection
- **Favicon Privacy**: Control over external favicon service data sharing
- **Personal Data Protection**: Careful handling of personal customization data
- **Analytics Opt-out**: User control over usage analytics and data collection
- **Third-Party Integration**: Transparent disclosure of third-party service usage

### System Integration Security
- **Hyper Key Security**: Secure system integration for hyper key functionality
- **Window Capture Permissions**: Proper system permission handling for screen capture
- **Input Method Security**: Secure handling of advanced input method integration
- **Privilege Minimization**: Minimal system privileges for advanced features

## Performance Optimization Requirements

### Advanced Input Performance
- **Low Latency Processing**: Minimal latency for text replacement and input processing
- **Resource Efficiency**: Efficient resource usage for continuous input monitoring
- **System Integration**: Optimized system integration without performance impact
- **Background Processing**: Efficient background processing of input customizations

### Data Management Performance
- **Export Optimization**: Efficient export processing for large data sets
- **Import Performance**: Fast import processing with progress feedback
- **Cache Management**: Intelligent cache management for favicons and assets
- **Scheduled Operations**: Efficient background processing for scheduled exports

### UI Responsiveness
- **Real-time Feedback**: Immediate visual feedback for all configuration changes
- **Smooth Interactions**: Fluid interactions for emoji selection and configuration
- **Progressive Loading**: Incremental loading of configuration options
- **Non-blocking Operations**: Non-blocking UI for long-running operations

## Accessibility and Usability

### Advanced Accessibility Features
- **Input Method Accessibility**: Full accessibility support for advanced input methods
- **Emoji Accessibility**: Screen reader support for emoji customization
- **Export Accessibility**: Accessible export and import workflow
- **Documentation Integration**: Comprehensive help and documentation integration

### User Experience Enhancement
- **Progressive Disclosure**: Smart revelation of advanced options based on usage
- **Contextual Help**: Context-sensitive help and guidance
- **Error Prevention**: Proactive error prevention and user guidance
- **Recovery Options**: Comprehensive recovery options for failed operations

### Internationalization Support
- **Emoji Localization**: Cultural and regional emoji customization
- **Text Replacement Localization**: Localized text replacement rules
- **Export Format Localization**: Localized export formats and documentation
- **Accessibility Localization**: Localized accessibility features and descriptions

## Error Handling and Recovery

### Advanced Feature Error Handling
- **Hyper Key Failures**: Graceful handling of hyper key registration failures
- **Text Replacement Errors**: Safe handling of text replacement rule conflicts
- **Emoji System Errors**: Recovery from emoji system integration issues
- **Window Capture Failures**: Clear error handling for capture permission or system issues

### Data Management Error Recovery
- **Export Failures**: Comprehensive error handling and recovery for failed exports
- **Import Validation**: Detailed validation with clear error messaging for imports
- **Schedule Failures**: Automatic recovery and user notification for failed scheduled exports
- **Data Corruption**: Detection and recovery from corrupted export/import data

### User Experience Recovery
- **Configuration Reset**: Safe reset options for problematic advanced configurations
- **Incremental Recovery**: Step-by-step recovery from complex configuration issues
- **Diagnostic Tools**: Built-in diagnostic tools for troubleshooting advanced features
- **Expert Support**: Clear escalation paths for complex technical issues

## Bevy Implementation Details

### Component Architecture

#### Specialized Advanced Components
```rust
use bevy::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Component, Reflect)]
pub struct EmojiSkinToneSelector {
    pub selected_tone: SkinTone,
    pub hover_tone: Option<SkinTone>,
    pub tones: Vec<EmojiToneOption>,
}

#[derive(Component, Reflect)]
pub struct FaviconProviderConfig {
    pub current_provider: FaviconProvider,
    pub available_providers: Vec<FaviconProviderOption>,
    pub test_url: Option<String>,
    pub testing: bool,
}

#[derive(Component, Reflect)]
pub struct ImportExportPanel {
    pub export_progress: Option<ExportProgress>,
    pub import_progress: Option<ImportProgress>,
    pub last_export: Option<std::time::SystemTime>,
    pub last_import: Option<std::time::SystemTime>,
    pub scheduled_exports: Vec<ScheduledExport>,
}

#[derive(Component, Reflect)]
pub struct WindowCaptureConfig {
    pub hotkey: Option<KeyCombination>,
    pub copy_to_clipboard: bool,
    pub show_in_finder: bool,
    pub capture_format: CaptureFormat,
    pub quality: f32,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum SkinTone {
    Light,
    MediumLight,
    Medium,
    MediumDark,
    Dark,
    Default,
}

#[derive(Reflect, Clone)]
pub struct EmojiToneOption {
    pub tone: SkinTone,
    pub emoji_example: String,
    pub unicode_modifier: String,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum FaviconProvider {
    Raycast,
    Google,
    Custom(String),
}

#[derive(Reflect, Clone)]
pub struct FaviconProviderOption {
    pub provider: FaviconProvider,
    pub name: String,
    pub description: String,
    pub privacy_level: PrivacyLevel,
}

#[derive(Reflect, Clone, Copy, PartialEq)]
pub enum PrivacyLevel {
    High,    // Local processing
    Medium,  // First-party service
    Low,     // Third-party service
}
```

#### Export/Import System Components
```rust
#[derive(Component, Reflect)]
pub struct ExportProgress {
    pub current_step: String,
    pub progress: f32,
    pub total_items: u32,
    pub processed_items: u32,
    pub start_time: std::time::SystemTime,
}

#[derive(Component, Reflect)]
pub struct ImportProgress {
    pub current_step: String,
    pub progress: f32,
    pub validation_results: Vec<ValidationResult>,
    pub conflicts: Vec<ImportConflict>,
    pub user_choices: HashMap<String, ConflictResolution>,
}

#[derive(Reflect, Clone)]
pub struct ImportConflict {
    pub item_type: String,
    pub item_name: String,
    pub conflict_reason: String,
    pub resolution_options: Vec<ConflictResolution>,
}

#[derive(Reflect, Clone, PartialEq)]
pub enum ConflictResolution {
    OverwriteExisting,
    KeepExisting,
    Rename(String),
    Skip,
}

#[derive(Reflect, Clone)]
pub struct ScheduledExport {
    pub id: String,
    pub name: String,
    pub schedule: ExportSchedule,
    pub destination: ExportDestination,
    pub last_run: Option<std::time::SystemTime>,
    pub next_run: std::time::SystemTime,
    pub enabled: bool,
}

#[derive(Reflect, Clone)]
pub enum ExportSchedule {
    Daily,
    Weekly,
    Monthly,
    Custom(String), // Cron expression
}

#[derive(Reflect, Clone)]
pub enum ExportDestination {
    LocalFile(String),
    CloudStorage { service: String, path: String },
    Email(String),
}
```

### Resource Management for Advanced Features
```rust
#[derive(Resource, Reflect)]
pub struct AdvancedSettings2State {
    pub hyper_key: Option<KeyCode>,
    pub text_replacement_enabled: bool,
    pub replacement_rules: HashMap<String, String>,
    pub favicon_provider: FaviconProvider,
    pub emoji_skin_tone: SkinTone,
    pub window_capture_config: WindowCaptureConfig,
    pub export_settings: ExportSettings,
}

#[derive(Reflect)]
pub struct ExportSettings {
    pub include_settings: bool,
    pub include_quicklinks: bool,
    pub include_snippets: bool,
    pub include_notes: bool,
    pub include_script_paths: bool,
    pub include_aliases: bool,
    pub include_hotkeys: bool,
    pub include_favorites: bool,
    pub include_window_commands: bool,
    pub compress_export: bool,
    pub encrypt_sensitive_data: bool,
}

#[derive(Resource, Default, Reflect)]
pub struct EmojiPersonalization {
    pub skin_tone_preferences: HashMap<String, SkinTone>,
    pub frequently_used: Vec<String>,
    pub custom_shortcuts: HashMap<String, String>,
}
```

### Event System for Advanced Features 2
```rust
#[derive(Event, Reflect)]
pub enum AdvancedConfig2Event {
    SkinToneChanged(SkinTone),
    SkinToneHovered(Option<SkinTone>),
    FaviconProviderChanged(FaviconProvider),
    FaviconProviderTest(String),
    ExportRequested,
    ImportRequested,
    ExportScheduleConfigured(ScheduledExport),
    WindowCaptureHotkeyRecord,
    WindowCaptureConfigChanged(WindowCaptureConfig),
    TextReplacementRuleAdded { from: String, to: String },
    TextReplacementRuleRemoved(String),
}

#[derive(Event, Reflect)]
pub enum DataPortabilityEvent {
    ExportStarted { destination: String, settings: ExportSettings },
    ExportProgress { step: String, progress: f32 },
    ExportCompleted { file_path: String, size: u64 },
    ExportFailed { error: String },
    ImportStarted { source: String },
    ImportValidated { conflicts: Vec<ImportConflict> },
    ImportConflictResolved { item: String, resolution: ConflictResolution },
    ImportCompleted { imported_items: u32 },
    ImportFailed { error: String },
}
```

### Emoji Skin Tone Selector System
```rust
fn emoji_skin_tone_system(
    mut skin_tone_selectors: Query<&mut EmojiSkinToneSelector>,
    mut button_query: Query<(&Interaction, &EmojiToneButton), Changed<Interaction>>,
    mut emoji_personalization: ResMut<EmojiPersonalization>,
    mut events: EventWriter<AdvancedConfig2Event>,
) {
    // Handle emoji button interactions
    for (interaction, tone_button) in button_query.iter() {
        match *interaction {
            Interaction::Pressed => {
                for mut selector in skin_tone_selectors.iter_mut() {
                    selector.selected_tone = tone_button.tone;
                    selector.hover_tone = None;
                }
                
                // Update global emoji preferences
                emoji_personalization.skin_tone_preferences.insert(
                    "default".to_string(),
                    tone_button.tone
                );
                
                events.send(AdvancedConfig2Event::SkinToneChanged(tone_button.tone));
            },
            Interaction::Hovered => {
                for mut selector in skin_tone_selectors.iter_mut() {
                    selector.hover_tone = Some(tone_button.tone);
                }
                events.send(AdvancedConfig2Event::SkinToneHovered(Some(tone_button.tone)));
            },
            Interaction::None => {
                for mut selector in skin_tone_selectors.iter_mut() {
                    if selector.hover_tone == Some(tone_button.tone) {
                        selector.hover_tone = None;
                    }
                }
            },
        }
    }
}

#[derive(Component)]
pub struct EmojiToneButton {
    pub tone: SkinTone,
}

fn spawn_emoji_skin_tone_selector(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(24.0)),
        max_width: Val::Px(400.0), // Constrain width
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Emoji Skin Tone"),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));
        
        // Emoji selector row
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(64.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::SpaceEvenly,
                border_radius: BorderRadius::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Px(8.0)),
                overflow: Overflow::clip(), // Prevent expansion
                flex_grow: 0.0,
                ..default()
            },
            BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            EmojiSkinToneSelector {
                selected_tone: SkinTone::Medium,
                hover_tone: None,
                tones: vec![
                    EmojiToneOption {
                        tone: SkinTone::Light,
                        emoji_example: "👋🏻".to_string(),
                        unicode_modifier: "🏻".to_string(),
                    },
                    EmojiToneOption {
                        tone: SkinTone::MediumLight,
                        emoji_example: "👋🏼".to_string(),
                        unicode_modifier: "🏼".to_string(),
                    },
                    EmojiToneOption {
                        tone: SkinTone::Medium,
                        emoji_example: "👋🏽".to_string(),
                        unicode_modifier: "🏽".to_string(),
                    },
                    EmojiToneOption {
                        tone: SkinTone::MediumDark,
                        emoji_example: "👋🏾".to_string(),
                        unicode_modifier: "🏾".to_string(),
                    },
                    EmojiToneOption {
                        tone: SkinTone::Dark,
                        emoji_example: "👋🏿".to_string(),
                        unicode_modifier: "🏿".to_string(),
                    },
                ],
            },
        )).with_children(|parent| {
            // Spawn individual emoji buttons
            for tone_option in &[
                ("👋🏻", SkinTone::Light),
                ("👋🏼", SkinTone::MediumLight),
                ("👋🏽", SkinTone::Medium),
                ("👋🏾", SkinTone::MediumDark),
                ("👋🏿", SkinTone::Dark),
            ] {
                parent.spawn((
                    Node {
                        width: Val::Px(48.0),
                        height: Val::Px(48.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        border_radius: BorderRadius::all(Val::Px(6.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                    Button,
                    Interaction::default(),
                    EmojiToneButton { tone: tone_option.1 },
                )).with_children(|parent| {
                    parent.spawn((
                        Text::new(tone_option.0),
                        TextFont { font_size: 24.0, ..default() },
                    ));
                });
            }
        });
    });
}
```

### Import/Export System Implementation
```rust
fn import_export_system(
    mut import_export_panels: Query<&mut ImportExportPanel>,
    mut button_query: Query<(&Interaction, &ImportExportButton), Changed<Interaction>>,
    mut events: EventWriter<DataPortabilityEvent>,
    settings_state: Res<AdvancedSettings2State>,
) {
    for (interaction, button) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match button.action {
                ImportExportAction::Export => {
                    for mut panel in import_export_panels.iter_mut() {
                        panel.export_progress = Some(ExportProgress {
                            current_step: "Preparing export...".to_string(),
                            progress: 0.0,
                            total_items: calculate_export_items(&settings_state.export_settings),
                            processed_items: 0,
                            start_time: std::time::SystemTime::now(),
                        });
                    }
                    
                    events.send(DataPortabilityEvent::ExportStarted {
                        destination: "local_file".to_string(),
                        settings: settings_state.export_settings.clone(),
                    });
                },
                ImportExportAction::Import => {
                    for mut panel in import_export_panels.iter_mut() {
                        panel.import_progress = Some(ImportProgress {
                            current_step: "Validating import data...".to_string(),
                            progress: 0.0,
                            validation_results: vec![],
                            conflicts: vec![],
                            user_choices: HashMap::new(),
                        });
                    }
                    
                    events.send(DataPortabilityEvent::ImportStarted {
                        source: "file_picker".to_string(),
                    });
                },
                ImportExportAction::ConfigureSchedule => {
                    // Open schedule configuration modal
                    spawn_export_schedule_modal();
                },
            }
        }
    }
}

#[derive(Component)]
pub struct ImportExportButton {
    pub action: ImportExportAction,
}

#[derive(Clone, Copy, PartialEq)]
pub enum ImportExportAction {
    Export,
    Import,
    ConfigureSchedule,
}

fn calculate_export_items(settings: &ExportSettings) -> u32 {
    let mut count = 0;
    if settings.include_settings { count += 50; }
    if settings.include_quicklinks { count += 100; }
    if settings.include_snippets { count += 200; }
    // ... calculate based on actual data
    count
}

fn spawn_import_export_panel(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Column,
        margin: UiRect::bottom(Val::Px(32.0)),
        max_width: Val::Px(600.0), // Constrain width
        ..default()
    }).with_children(|parent| {
        // Section title
        parent.spawn((
            Text::new("Import / Export"),
            TextFont { font_size: 18.0, ..default() },
            TextColor(Color::WHITE),
            Node {
                margin: UiRect::bottom(Val::Px(12.0)),
                ..default()
            },
        ));
        
        // Description
        parent.spawn((
            Text::new("Exporting will back-up your settings, quicklinks, snippets, notes, script-command folder paths, aliases, hotkeys, favorites, custom window management commands and other data."),
            TextFont { font_size: 13.0, ..default() },
            TextColor(Color::srgb(0.7, 0.7, 0.7)),
            Node {
                margin: UiRect::bottom(Val::Px(20.0)),
                max_width: Val::Px(500.0), // Prevent text overflow
                ..default()
            },
        ));
        
        // Button row
        parent.spawn(Node {
            width: Val::Percent(100.0),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(12.0),
            flex_wrap: FlexWrap::Wrap,
            ..default()
        }).with_children(|parent| {
            // Import button
            spawn_action_button(parent, "Import", ImportExportAction::Import, "↓");
            
            // Export button
            spawn_action_button(parent, "Export", ImportExportAction::Export, "↑");
            
            // Configure schedule button
            spawn_action_button(parent, "Configure Export Schedule", ImportExportAction::ConfigureSchedule, "⏰");
        });
        
        // Progress indicator (when active)
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Px(4.0),
                margin: UiRect::top(Val::Px(16.0)),
                border_radius: BorderRadius::all(Val::Px(2.0)),
                overflow: Overflow::clip(),
                ..default()
            },
            BackgroundColor(Color::srgb(0.2, 0.2, 0.2)),
        )).with_children(|parent| {
            parent.spawn((
                Node {
                    width: Val::Percent(0.0), // Will be animated based on progress
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::srgb(0.2, 0.8, 0.2)),
            ));
        });
    });
}

fn spawn_action_button(
    parent: &mut ChildBuilder,
    text: &str,
    action: ImportExportAction,
    icon: &str,
) {
    parent.spawn((
        Node {
            padding: UiRect::all(Val::Px(12.0)),
            border: UiRect::all(Val::Px(1.0)),
            border_radius: BorderRadius::all(Val::Px(6.0)),
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(8.0),
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
        BorderColor(Color::srgb(0.3, 0.3, 0.3)),
        Button,
        Interaction::default(),
        ImportExportButton { action },
    )).with_children(|parent| {
        // Icon
        parent.spawn((
            Text::new(icon),
            TextFont { font_size: 16.0, ..default() },
            TextColor(Color::srgb(0.8, 0.8, 0.8)),
        ));
        
        // Text
        parent.spawn((
            Text::new(text),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
    });
}
```

### Window Capture System
```rust
fn window_capture_system(
    mut capture_configs: Query<&mut WindowCaptureConfig>,
    mut button_query: Query<(&Interaction, &WindowCaptureButton), Changed<Interaction>>,
    mut checkbox_query: Query<(&Interaction, &mut WindowCaptureCheckbox), Changed<Interaction>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut hotkey_recording: Local<bool>,
    mut events: EventWriter<AdvancedConfig2Event>,
) {
    // Handle button interactions
    for (interaction, button) in button_query.iter() {
        if *interaction == Interaction::Pressed {
            match button.action {
                WindowCaptureAction::RecordHotkey => {
                    *hotkey_recording = true;
                    events.send(AdvancedConfig2Event::WindowCaptureHotkeyRecord);
                },
                WindowCaptureAction::CaptureNow => {
                    // Trigger immediate window capture
                    perform_window_capture();
                },
            }
        }
    }
    
    // Handle checkbox toggles
    for (interaction, mut checkbox) in checkbox_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            checkbox.enabled = !checkbox.enabled;
            
            for mut config in capture_configs.iter_mut() {
                match checkbox.setting {
                    CaptureCheckboxSetting::CopyToClipboard => {
                        config.copy_to_clipboard = checkbox.enabled;
                    },
                    CaptureCheckboxSetting::ShowInFinder => {
                        config.show_in_finder = checkbox.enabled;
                    },
                }
                
                events.send(AdvancedConfig2Event::WindowCaptureConfigChanged(config.clone()));
            }
        }
    }
    
    // Handle hotkey recording
    if *hotkey_recording {
        if keyboard_input.just_pressed(KeyCode::Escape) {
            *hotkey_recording = false;
        } else {
            // Capture key combination
            let mut modifiers = Vec::new();
            let mut key = None;
            
            if keyboard_input.pressed(KeyCode::ControlLeft) || keyboard_input.pressed(KeyCode::ControlRight) {
                modifiers.push(Modifier::Control);
            }
            if keyboard_input.pressed(KeyCode::SuperLeft) || keyboard_input.pressed(KeyCode::SuperRight) {
                modifiers.push(Modifier::Command);
            }
            
            for keycode in keyboard_input.get_just_pressed() {
                if !matches!(keycode, KeyCode::ControlLeft | KeyCode::ControlRight | KeyCode::SuperLeft | KeyCode::SuperRight) {
                    key = Some(*keycode);
                    break;
                }
            }
            
            if let Some(key_code) = key {
                let combination = KeyCombination { modifiers, key: key_code };
                
                for mut config in capture_configs.iter_mut() {
                    config.hotkey = Some(combination);
                }
                
                *hotkey_recording = false;
            }
        }
    }
}

#[derive(Component)]
pub struct WindowCaptureButton {
    pub action: WindowCaptureAction,
}

#[derive(Component)]
pub struct WindowCaptureCheckbox {
    pub setting: CaptureCheckboxSetting,
    pub enabled: bool,
}

#[derive(Clone, Copy, PartialEq)]
pub enum WindowCaptureAction {
    RecordHotkey,
    CaptureNow,
}

#[derive(Clone, Copy, PartialEq)]
pub enum CaptureCheckboxSetting {
    CopyToClipboard,
    ShowInFinder,
}

fn perform_window_capture() {
    // Implementation would use platform-specific screen capture API
    info!("Performing window capture");
}
```

### Favicon Provider System
```rust
fn favicon_provider_system(
    mut favicon_configs: Query<&mut FaviconProviderConfig>,
    mut dropdown_query: Query<(&Interaction, &FaviconProviderDropdown), Changed<Interaction>>,
    mut events: EventWriter<AdvancedConfig2Event>,
) {
    for (interaction, dropdown) in dropdown_query.iter() {
        if *interaction == Interaction::Pressed {
            for mut config in favicon_configs.iter_mut() {
                config.current_provider = dropdown.provider.clone();
                
                // Start provider test
                config.testing = true;
                config.test_url = Some("https://github.com".to_string());
                
                events.send(AdvancedConfig2Event::FaviconProviderChanged(dropdown.provider.clone()));
                events.send(AdvancedConfig2Event::FaviconProviderTest("https://github.com".to_string()));
            }
        }
    }
}

#[derive(Component)]
pub struct FaviconProviderDropdown {
    pub provider: FaviconProvider,
}

fn spawn_favicon_provider_config(parent: &mut ChildBuilder) {
    parent.spawn(Node {
        width: Val::Percent(100.0),
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        justify_content: JustifyContent::SpaceBetween,
        margin: UiRect::bottom(Val::Px(16.0)),
        max_width: Val::Px(500.0), // Constrain width
        ..default()
    }).with_children(|parent| {
        // Label
        parent.spawn((
            Text::new("Favicon Provider"),
            TextFont { font_size: 14.0, ..default() },
            TextColor(Color::WHITE),
        ));
        
        // Dropdown
        parent.spawn((
            Node {
                width: Val::Px(150.0),
                height: Val::Px(36.0),
                padding: UiRect::all(Val::Px(8.0)),
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(4.0)),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                flex_grow: 0.0, // Prevent expansion
                ..default()
            },
            BackgroundColor(Color::srgb(0.12, 0.12, 0.12)),
            BorderColor(Color::srgb(0.3, 0.3, 0.3)),
            Button,
            Interaction::default(),
            FaviconProviderDropdown {
                provider: FaviconProvider::Raycast,
            },
        )).with_children(|parent| {
            parent.spawn((
                Text::new("Raycast"),
                TextFont { font_size: 13.0, ..default() },
                TextColor(Color::WHITE),
            ));
        });
    });
}
```

### SystemSet Organization for Advanced Features 2
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AdvancedConfig2Systems {
    Input,
    DataPortability,
    WindowCapture,
    Personalization,
    Validation,
    UI,
}

impl Plugin for AdvancedConfig2Plugin {
    fn build(&self, app: &mut App) {
        app
            .register_type::<EmojiSkinToneSelector>()
            .register_type::<FaviconProviderConfig>()
            .register_type::<ImportExportPanel>()
            .register_type::<WindowCaptureConfig>()
            .register_type::<AdvancedSettings2State>()
            
            .init_resource::<AdvancedSettings2State>()
            .init_resource::<EmojiPersonalization>()
            
            .add_event::<AdvancedConfig2Event>()
            .add_event::<DataPortabilityEvent>()
            
            .configure_sets(Update, (
                AdvancedConfig2Systems::Input,
                AdvancedConfig2Systems::Personalization,
                AdvancedConfig2Systems::DataPortability,
                AdvancedConfig2Systems::WindowCapture,
                AdvancedConfig2Systems::Validation,
                AdvancedConfig2Systems::UI,
            ).chain())
            
            .add_systems(Update, (
                // Input handling
                (
                    keyboard_input_system,
                    mouse_interaction_system,
                ).in_set(AdvancedConfig2Systems::Input),
                
                // Personalization features
                (
                    emoji_skin_tone_system,
                    favicon_provider_system,
                ).in_set(AdvancedConfig2Systems::Personalization),
                
                // Data portability operations
                (
                    import_export_system,
                    export_progress_system,
                    import_validation_system,
                ).in_set(AdvancedConfig2Systems::DataPortability),
                
                // Window capture functionality
                (
                    window_capture_system,
                    capture_hotkey_system,
                ).in_set(AdvancedConfig2Systems::WindowCapture),
                
                // Validation and testing
                (
                    favicon_provider_validation_system,
                    export_validation_system,
                ).in_set(AdvancedConfig2Systems::Validation),
                
                // UI updates with Changed<T> optimization
                (
                    emoji_selector_ui_system,
                    import_export_ui_system,
                    progress_bar_animation_system,
                ).in_set(AdvancedConfig2Systems::UI),
            ));
    }
}
```

This comprehensive Bevy implementation provides:

1. **Visual emoji selector** with proper skin tone selection and hover states
2. **Import/Export system** with progress tracking and conflict resolution
3. **Window capture configuration** with hotkey recording and checkbox controls
4. **Favicon provider management** with real-time testing and validation
5. **Flex-based layouts** with proper constraints to prevent expansion
6. **Component-driven architecture** with full reflection support for debugging
7. **Event-driven patterns** for all configuration changes and user interactions
8. **Query optimization** using `Changed<T>` filters for efficient UI updates