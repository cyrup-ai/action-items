# About_Menu Task 4: Dynamic Version Management System

## Task Overview
Implement dynamic version and metadata display system for the About menu, providing real-time application information and build metadata.

## Implementation Requirements

### Core Components
```rust
// Version management system
#[derive(Resource, Reflect, Debug)]
pub struct VersionManagementResource {
    pub app_version: String,
    pub build_hash: String,
    pub build_date: DateTime<Utc>,
    pub rust_version: String,
    pub bevy_version: String,
    pub platform_info: PlatformInfo,
}

#[derive(Component, Reflect, Debug)]
pub struct VersionDisplayComponent {
    pub version_text: Entity,
    pub build_info_text: Entity,
    pub platform_text: Entity,
    pub update_interval: Duration,
}

#[derive(Reflect, Debug)]
pub struct PlatformInfo {
    pub os_name: String,
    pub os_version: String,
    pub arch: String,
    pub total_memory: u64,
}
```

### Build Information System
```rust
// Build-time metadata capture
#[derive(Resource, Reflect)]
pub struct BuildMetadataResource {
    pub version: &'static str,
    pub git_hash: &'static str,
    pub build_timestamp: &'static str,
    pub cargo_features: Vec<&'static str>,
}

impl Default for BuildMetadataResource {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION"),
            git_hash: option_env!("GIT_HASH").unwrap_or("unknown"),
            build_timestamp: env!("BUILD_TIMESTAMP"),
            cargo_features: vec![
                #[cfg(feature = "dev")]
                "dev",
                #[cfg(feature = "pro")]
                "pro",
            ],
        }
    }
}
```

### Dynamic Update System
```rust
pub fn update_version_display_system(
    mut version_query: Query<&mut Text, With<VersionDisplayComponent>>,
    version_resource: Res<VersionManagementResource>,
    time: Res<Time>,
) {
    // System with change detection for minimal updates
    if version_resource.is_changed() {
        for mut text in &mut version_query {
            // Update version display with zero allocations
        }
    }
}
```

## Bevy Examples Integration

### Related Examples from docs/bevy/examples:
- `ui/text.rs` - Text rendering patterns
- `utils/tracing_config.rs` - System information gathering
- `time/time.rs` - Time-based updates

### Implementation Pattern
```rust
// Based on ui/text.rs for dynamic text updates
fn version_text_system(
    mut text_query: Query<&mut Text, With<VersionText>>,
    version_res: Res<VersionManagementResource>,
) {
    for mut text in &mut text_query {
        if version_res.is_changed() {
            text.sections[0].value = format!(
                "Version {}\nBuild {}\n{}",
                version_res.app_version,
                &version_res.build_hash[..8],
                version_res.platform_info.os_name
            );
        }
    }
}
```

## System Information Collection
- Real-time memory usage monitoring
- Platform detection and display
- Build configuration information
- Performance metrics integration

## Performance Constraints
- **ZERO ALLOCATIONS** during version display updates
- Cached string formatting for version information
- Efficient system information collection
- Minimal UI update frequency

## Success Criteria
- Dynamic version information displays correctly
- Build metadata shows accurate information
- No unwrap()/expect() calls in production code
- Zero-allocation string updates
- Responsive UI with minimal performance impact

## DO NOT MOCK, FABRICATE, FAKE or SIMULATE ANY OPERATION or DATA
Make ONLY THE MINIMAL, SURGICAL CHANGES required. Do not modify or rewrite any portion of the app outside scope.

## Testing Requirements
- Unit tests for version string formatting
- Integration tests for build metadata collection
- Performance tests for display update efficiency
- System information accuracy validation

## Bevy Implementation Details

### Component Architecture for Version Management
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct VersionDisplayPanel {
    pub show_detailed_info: bool,
    pub last_update: SystemTime,
    pub update_interval: Duration,
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum VersionSystemSet {
    MetadataCollection,
    DisplayUpdate,
}

impl Plugin for VersionManagementPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            VersionSystemSet::MetadataCollection,
            VersionSystemSet::DisplayUpdate,
        ).chain())
        .add_systems(Update, (
            collect_system_metadata.in_set(VersionSystemSet::MetadataCollection),
            update_version_display.in_set(VersionSystemSet::DisplayUpdate),
        ));
    }
}
```

### Dynamic Version Display with Change Detection
```rust
fn update_version_display(
    version_resource: Res<VersionManagementResource>,
    mut version_query: Query<&mut Text, (With<VersionDisplayComponent>, Changed<VersionManagementResource>)>,
) {
    if version_resource.is_changed() {
        for mut text in &mut version_query {
            text.sections[0].value = format!(
                "Version {}\nBuild {}\n{}",
                version_resource.app_version,
                &version_resource.build_hash[..8],
                version_resource.platform_info.os_name
            );
        }
    }
}
```

### Testing Strategy for Version Management
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_version_metadata_collection() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, VersionManagementPlugin));
        
        app.insert_resource(VersionManagementResource {
            app_version: "1.0.0".to_string(),
            build_hash: "abcd1234".to_string(),
            build_date: Utc::now(),
            rust_version: "1.70.0".to_string(),
            bevy_version: "0.12.0".to_string(),
            platform_info: PlatformInfo::default(),
        });
        
        app.update();
        
        let version_resource = app.world().resource::<VersionManagementResource>();
        assert_eq!(version_resource.app_version, "1.0.0");
    }
}