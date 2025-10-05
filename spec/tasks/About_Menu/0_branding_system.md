# Task 0: Application Branding System Implementation

## Objective
Implement the centered application branding system displaying logo, title, version, and copyright information with proper typography hierarchy and spacing.

## Implementation Details

### Target Files
- `ui/src/ui/components/about_menu.rs:45-120` - Main branding component implementation
- `ui/src/ui/components/mod.rs:15` - Export AboutBranding component
- `ui/src/ui/theme.rs:280-320` - About menu typography and color definitions
- `core/src/metadata.rs:1-40` - Version and metadata access system

### Bevy Implementation Patterns

#### Centered Layout Container
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:45-65` - JustifyContent::Center and AlignItems::Center
```rust
// Centered flex container for main branding
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        width: Val::Percent(100.0),
        height: Val::Percent(80.0),
        ..default()
    },
    ..default()
}
```

#### Logo Image Display  
**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:25-45` - High-resolution image display with proper scaling
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:70-90` - Asset loading and caching
```rust
// Logo image component with proper sizing
ImageBundle {
    style: Style {
        width: Val::Px(120.0),
        height: Val::Px(120.0),
        margin: UiRect::bottom(Val::Px(20.0)),
        ..default()
    },
    image: logo_handle.clone().into(),
    ..default()
}
```

#### Typography Hierarchy System
**Reference**: `./docs/bevy/examples/ui/text.rs:15-40` - Multi-level text styling with different font sizes
**Reference**: `./docs/bevy/examples/ui/text.rs:65-85` - Text color and alignment configuration
```rust
// Application name with large bold styling
TextBundle::from_section(
    "Action Items",
    TextStyle {
        font: font_bold.clone(),
        font_size: 36.0,
        color: Color::WHITE,
    },
).with_style(Style {
    margin: UiRect::bottom(Val::Px(8.0)),
    ..default()
})

// Version text with medium gray styling  
TextBundle::from_section(
    format!("Version {}", metadata.version),
    TextStyle {
        font: font_regular.clone(),
        font_size: 16.0,
        color: Color::rgba(0.53, 0.53, 0.53, 1.0), // #888888
    },
)
```

#### Multi-line Copyright Text
**Reference**: `./docs/bevy/examples/ui/text.rs:100-125` - Multi-section text with different styles
**Reference**: `./docs/bevy/examples/time/time.rs:30-50` - Dynamic date calculation for copyright years
```rust
// Multi-line copyright with consistent styling
TextBundle::from_sections([
    TextSection::new(
        "© Action Items Technologies Ltd.\n",
        TextStyle {
            font: font_regular.clone(),
            font_size: 14.0,
            color: Color::rgba(0.4, 0.4, 0.4, 1.0), // #666666
        },
    ),
    TextSection::new(
        format!("2019-{}. All Rights Reserved.", current_year),
        TextStyle {
            font: font_regular.clone(),
            font_size: 14.0,
            color: Color::rgba(0.4, 0.4, 0.4, 1.0),
        },
    ),
])
```

### Architecture Notes

#### Component Structure
- **AboutBrandingComponent**: Main ECS component for branding state
- **MetadataResource**: ECS resource containing version and build information  
- **BrandingAssets**: Resource containing logo and font handles
- **about_branding_system**: System for updating dynamic text content

#### Vertical Spacing System
**Reference**: `./docs/bevy/examples/ui/ui.rs:120-145` - Consistent margin and padding patterns
- Logo: 20px bottom margin
- App name: 8px bottom margin  
- Version: 8px bottom margin
- Copyright: Flush with version, left-aligned

#### Typography Integration
**Reference**: `./docs/bevy/examples/ui/text_debug.rs:40-65` - Text alignment and positioning validation
- **Primary**: 36px bold white for application name
- **Secondary**: 16px regular medium gray for version
- **Tertiary**: 14px regular light gray for copyright
- **Consistent left-alignment** for text hierarchy

### Quality Standards
- Zero allocation text updates using cached strings
- Asset preloading with loading state management
- Proper error handling for missing assets or metadata
- WCAG AA color contrast compliance for all text
- Responsive scaling for different window sizes

### Integration Points
- Metadata system integration for version display
- Asset loading system for logo display  
- Theme system integration for colors and typography
- Animation system hooks for entrance effects

## Bevy Implementation Details

### Component Architecture for About Branding
```rust
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
pub struct AboutBrandingPanel {
    pub logo_entity: Option<Entity>,
    pub title_entity: Option<Entity>,
    pub version_entity: Option<Entity>,
    pub copyright_entity: Option<Entity>,
}

#[derive(Component, Reflect)]
pub struct BrandingAssets {
    pub logo_handle: Handle<Image>,
    pub bold_font: Handle<Font>,
    pub regular_font: Handle<Font>,
}
```

### System Architecture for Branding Display
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum BrandingSystemSet {
    AssetLoading,
    LayoutUpdate,
    UIRefresh,
}

impl Plugin for AboutBrandingPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(Update, (
            BrandingSystemSet::AssetLoading,
            BrandingSystemSet::LayoutUpdate,
            BrandingSystemSet::UIRefresh,
        ).chain())
        .add_systems(Update, (
            load_branding_assets.in_set(BrandingSystemSet::AssetLoading),
            update_branding_layout.in_set(BrandingSystemSet::LayoutUpdate),
            refresh_branding_ui.in_set(BrandingSystemSet::UIRefresh),
        ));
    }
}
```

### Flex-Based UI Layout for Centered Branding
```rust
fn spawn_about_branding_ui(mut commands: Commands, assets: Res<BrandingAssets>) {
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            padding: UiRect::all(Val::Px(32.0)),
            max_width: Val::Px(600.0),
            flex_grow: 0.0, // Prevent expansion
            ..default()
        },
        AboutBrandingPanel::default(),
    ))
    .with_children(|parent| {
        // Logo
        parent.spawn((
            ImageBundle {
                style: Style {
                    width: Val::Px(120.0),
                    height: Val::Px(120.0),
                    margin: UiRect::bottom(Val::Px(20.0)),
                    ..default()
                },
                image: UiImage::new(assets.logo_handle.clone()),
                ..default()
            },
        ));
        
        // App title
        parent.spawn((
            Text::from_section(
                "Action Items",
                TextStyle {
                    font: assets.bold_font.clone(),
                    font_size: 36.0,
                    color: Color::WHITE,
                }
            ),
            Node {
                margin: UiRect::bottom(Val::Px(8.0)),
                ..default()
            },
        ));
    });
}
```

### Testing Strategy for Branding System
```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_branding_component_creation() {
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, AboutBrandingPlugin));
        
        let branding_panel = AboutBrandingPanel::default();
        let entity = app.world_mut().spawn(branding_panel).id();
        
        app.update();
        
        assert!(app.world().get::<AboutBrandingPanel>(entity).is_some());
    }
}