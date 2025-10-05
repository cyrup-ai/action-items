# Task 4: Version Management and Metadata System Implementation

## Objective
Implement dynamic version information system with build metadata, copyright year calculation, environment detection, and metadata access for the About menu display.

## Implementation Details

### Target Files
- `core/src/metadata.rs:1-120` - Core metadata resource and build information
- `core/src/build_info.rs:1-80` - Build-time metadata extraction and compilation
- `ui/src/ui/systems/version_display.rs:1-100` - Version display update system
- `Cargo.toml:1-50` - Package metadata and version configuration

### Bevy Implementation Patterns

#### Metadata Resource System
**Reference**: `./docs/bevy/examples/ecs/resources.rs:15-40` - ECS resource definition and access patterns
**Reference**: `./docs/bevy/examples/reflection/reflection.rs:60-85` - Runtime metadata access and reflection
```rust
// Core metadata resource for application information
#[derive(Resource, Clone, Debug)]
pub struct AppMetadata {
    pub version: String,
    pub build_date: String,
    pub commit_hash: Option<String>,
    pub build_env: BuildEnvironment,
    pub copyright_years: String,
}

impl Default for AppMetadata {
    fn default() -> Self {
        Self {
            version: env!("CARGO_PKG_VERSION").to_string(),
            build_date: build_info::BUILD_DATE.to_string(),
            commit_hash: build_info::GIT_COMMIT_HASH.map(String::from),
            build_env: BuildEnvironment::from_env(),
            copyright_years: calculate_copyright_years(),
        }
    }
}
```

#### Build Information Integration
**Reference**: `./docs/bevy/examples/app/empty.rs:10-25` - App configuration and resource insertion
**Reference**: `./docs/bevy/examples/time/time.rs:40-65` - Time-based calculations and date handling
```rust
// Build-time information extraction
pub mod build_info {
    use chrono::{DateTime, Utc};

    pub const BUILD_DATE: &str = env!("BUILD_DATE");
    pub const GIT_COMMIT_HASH: Option<&str> = option_env!("GIT_COMMIT_HASH");
    pub const BUILD_PROFILE: &str = env!("BUILD_PROFILE");
    
    pub fn current_year() -> i32 {
        Utc::now().year()
    }
    
    pub fn build_timestamp() -> DateTime<Utc> {
        BUILD_DATE.parse().unwrap_or_else(|_| Utc::now())
    }
}
```

#### Dynamic Copyright Calculation
**Reference**: `./docs/bevy/examples/time/time.rs:15-35` - Date and time manipulation patterns
```rust
// Dynamic copyright year calculation
fn calculate_copyright_years() -> String {
    const START_YEAR: i32 = 2019;
    let current_year = build_info::current_year();
    
    if current_year == START_YEAR {
        START_YEAR.to_string()
    } else {
        format!("{}-{}", START_YEAR, current_year)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildEnvironment {
    Development,
    Production,
    Staging,
}

impl BuildEnvironment {
    pub fn from_env() -> Self {
        match build_info::BUILD_PROFILE {
            "debug" => Self::Development,
            "release" => Self::Production,
            _ => Self::Staging,
        }
    }
}
```

### Version Display Update System

#### Dynamic Text Update System
**Reference**: `./docs/bevy/examples/ui/text.rs:200-230` - Dynamic text content updates
**Reference**: `./docs/bevy/examples/ecs/change_detection.rs:25-50` - Change detection for efficient updates
```rust
// System for updating version display when metadata changes
#[derive(Component)]
pub struct VersionText;

#[derive(Component)]
pub struct CopyrightText;

fn update_version_display_system(
    metadata: Res<AppMetadata>,
    mut version_query: Query<&mut Text, (With<VersionText>, Without<CopyrightText>)>,
    mut copyright_query: Query<&mut Text, (With<CopyrightText>, Without<VersionText>)>,
) {
    if metadata.is_changed() {
        // Update version text
        for mut text in version_query.iter_mut() {
            text.sections[0].value = format!("Version {}", metadata.version);
        }
        
        // Update copyright text
        for mut text in copyright_query.iter_mut() {
            text.sections[0].value = format!("© Action Items Technologies Ltd.\n{} All Rights Reserved.", 
                metadata.copyright_years);
        }
    }
}
```

#### Build Metadata Integration
**Reference**: `./docs/bevy/examples/diagnostics/system_information_diagnostics.rs:30-60` - System information access
```rust
// Extended metadata for development builds
#[derive(Component)]
pub struct BuildInfoText;

fn update_build_info_system(
    metadata: Res<AppMetadata>,
    mut query: Query<&mut Text, With<BuildInfoText>>,
) {
    if metadata.build_env == BuildEnvironment::Development {
        for mut text in query.iter_mut() {
            let info = format!(
                "Build: {} ({})\nCommit: {}",
                metadata.build_date,
                metadata.build_env.as_str(),
                metadata.commit_hash.as_deref().unwrap_or("unknown")
            );
            text.sections[0].value = info;
        }
    }
}
```

### Build Configuration Integration

#### Cargo.toml Metadata
```toml
[package]
name = "action-items"
version = "1.0.0"
authors = ["David Maple <david@cyrup.ai>"]
edition = "2021"
description = "Advanced launcher and productivity tool"

[package.metadata.about]
company = "Action Items Technologies Ltd."
copyright_start = "2019"
website = "https://cyrup.ai"
```

#### Build Script Integration
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:120-150` - Build-time asset processing
```rust
// build.rs for compile-time metadata injection
use std::process::Command;

fn main() {
    // Inject build timestamp
    println!("cargo:rustc-env=BUILD_DATE={}", chrono::Utc::now().to_rfc3339());
    
    // Inject git commit hash if available
    if let Ok(output) = Command::new("git")
        .args(&["rev-parse", "--short", "HEAD"])
        .output() 
    {
        if output.status.success() {
            let git_hash = String::from_utf8_lossy(&output.stdout);
            println!("cargo:rustc-env=GIT_COMMIT_HASH={}", git_hash.trim());
        }
    }
    
    // Inject build profile
    println!("cargo:rustc-env=BUILD_PROFILE={}", 
        std::env::var("PROFILE").unwrap_or_else(|_| "unknown".to_string()));
}
```

### Architecture Notes

#### Component Structure
- **AppMetadata**: Global resource containing all application metadata
- **VersionText**: Component marking version display text elements
- **CopyrightText**: Component marking copyright display text elements
- **BuildInfoText**: Component for development build information display

#### Update Strategy
- **Reactive Updates**: Use Bevy's change detection for efficient text updates
- **Build-time Injection**: Compile-time metadata extraction and embedding
- **Environment Detection**: Runtime environment determination for conditional features
- **Graceful Fallbacks**: Default values when metadata unavailable

#### Integration Points
- Build script integration for compile-time metadata
- Git integration for commit hash extraction
- Time system integration for copyright year calculation
- Text system integration for dynamic content updates

### Quality Standards
- Zero-allocation string updates using pre-allocated buffers
- Proper error handling for metadata extraction failures
- Build reproducibility with consistent metadata generation
- Cross-platform build script compatibility
- Performance optimization for frequent metadata access

### Development Features
- Extended build information display in development mode
- Hot-reload compatibility for metadata changes
- Debug information display for troubleshooting
- Version validation and consistency checking