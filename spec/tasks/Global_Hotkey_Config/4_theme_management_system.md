# Global Hotkey Config - Theme Management System

## Task: Implement Dynamic Theme Selection and System Integration

### File: `ui/src/systems/theme_management.rs` (new file)

Create comprehensive theme management system with live switching, system appearance integration, and custom theme support.

### Implementation Requirements

#### Dynamic Theme Switching System
- File: `ui/src/systems/theme_management.rs` (new file, line 1-134)
- Implement `theme_switching_system` with zero-allocation theme changes
- Bevy Example Reference: [`asset/asset_loading.rs`](../../../docs/bevy/examples/asset/asset_loading.rs) - Asset management patterns for theme resources
- Live theme switching without application restart using Bevy's asset system
- Integration with existing UI theme components and color systems

#### System Appearance Monitoring
- File: `ui/src/platform/system_appearance.rs` (new file, line 1-89)  
- macOS Dark Mode detection and automatic theme switching
- Windows/Linux system theme change notifications
- Real-time system appearance change event processing
- Integration with "Follow system appearance" preference setting

#### Theme Resource Management
```rust
pub fn load_theme_resources(
    mut theme_config: ResMut<ThemeConfig>,
    asset_server: Res<AssetServer>,
    mut theme_assets: ResMut<ThemeAssets>,
) {
    // Implementation following asset_loading.rs patterns
}
```
- Bevy Example Reference: [`asset/hot_asset_reloading.rs`](../../../docs/bevy/examples/asset/hot_asset_reloading.rs) - Hot reloading for live theme updates

#### Custom Theme Support System  
- File: `ui/src/systems/custom_themes.rs` (new file, line 1-67)
- Custom theme file validation and loading
- Theme Studio integration for advanced customization
- Community theme import/export capabilities
- Fallback to default theme on custom theme failures

#### Theme UI Components Integration
- File: `ui/src/components/theme_selector.rs` (new file, line 1-78)
- Theme dropdown component with preview capabilities
- "Follow system appearance" checkbox integration
- "Open Theme Studio" button component
- Real-time preview updates during theme selection

### Architecture Notes
- Event-driven theme switching with ThemeChangeEvent
- Asset-based theme resource management with hot reloading
- Zero-allocation theme switching using Bevy's change detection
- Integration with existing UI component color systems
- Atomic theme updates with rollback on failures

### Integration Points
- `ui/src/ui/theme.rs` - Existing theme system integration (lines 23-89)
- `ui/src/ui/components.rs` - UI component color updates (lines 156-234) 
- `core/src/` - Theme preference persistence integration
- System APIs: NSAppearance (macOS), Registry (Windows), dconf (Linux)

### Event System Integration
```rust
#[derive(Event)]
pub enum ThemeEvent {
    ThemeSelected(ThemeType),
    SystemAppearanceChanged(SystemAppearance),
    CustomThemeLoaded(PathBuf),
    ThemeStudioRequested,
    ThemeLoadFailed(String),
}
```

#### Theme Asset Management
- File: `ui/src/resources/theme_assets.rs` (new file, line 1-45)
- Efficient theme asset caching and management
- Preloading of commonly used themes
- Garbage collection of unused theme resources

### Bevy Example References
- **Asset Management**: [`asset/asset_loading.rs`](../../../docs/bevy/examples/asset/asset_loading.rs) - Dynamic resource loading
- **Hot Reloading**: [`asset/hot_asset_reloading.rs`](../../../docs/bevy/examples/asset/hot_asset_reloading.rs) - Live theme updates
- **UI Updates**: [`ui/ui.rs`](../../../docs/bevy/examples/ui/ui.rs) - UI component state management
- **Event System**: [`ecs/event.rs`](../../../docs/bevy/examples/ecs/event.rs) - Theme event patterns

DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA. Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.