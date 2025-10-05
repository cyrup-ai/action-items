# Task 0: User Profile Display System Implementation

## Objective
Implement the left panel user profile display system with circular avatar, verification badge, user information hierarchy, and subscription status banner.

## Implementation Details

### Target Files
- `ui/src/ui/components/account/profile_display.rs:1-200` - Main profile display component
- `ui/src/ui/components/account/avatar.rs:1-120` - Circular avatar with verification badge
- `ui/src/ui/components/account/subscription_banner.rs:1-80` - Status banner component
- `core/src/user/profile.rs:1-150` - User profile data management

### Bevy Implementation Patterns

#### Split Panel Layout Structure
**Reference**: `./docs/bevy/examples/ui/flex_layout.rs:85-110` - Flexible panel sizing with percentages
**Reference**: `./docs/bevy/examples/ui/ui.rs:140-170` - Multi-column layout structure
```rust
// Left panel container (40% width)
NodeBundle {
    style: Style {
        flex_direction: FlexDirection::Column,
        justify_content: JustifyContent::FlexStart,
        align_items: AlignItems::Center,
        width: Val::Percent(40.0),
        height: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(24.0)),
        gap: Size::all(Val::Px(16.0)),
        ..default()
    },
    background_color: theme.panel_background.into(),
    ..default()
}
```

#### Circular Profile Avatar System
**Reference**: `./docs/bevy/examples/ui/ui_texture_atlas.rs:45-70` - Circular image clipping and scaling
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:90-120` - Profile image loading with fallbacks
```rust
// Circular avatar container with blue accent ring
NodeBundle {
    style: Style {
        width: Val::Px(120.0),
        height: Val::Px(120.0),
        border: UiRect::all(Val::Px(3.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    border_color: Color::rgb(0.0, 0.48, 1.0).into(), // Blue accent ring
    border_radius: BorderRadius::all(Val::Px(60.0)), // Perfect circle
    ..default()
}

// Profile image with circular clipping
ImageBundle {
    style: Style {
        width: Val::Px(114.0), // Slightly smaller than container
        height: Val::Px(114.0),
        ..default()
    },
    image: profile_image_handle.clone().into(),
    border_radius: BorderRadius::all(Val::Px(57.0)),
    ..default()
}
```

#### Verification Badge Overlay
**Reference**: `./docs/bevy/examples/ui/ui.rs:250-280` - Positioned overlays and badges
```rust
// Verification badge positioned in bottom-right
NodeBundle {
    style: Style {
        position_type: PositionType::Absolute,
        position: UiRect {
            right: Val::Px(8.0),
            bottom: Val::Px(8.0),
            ..default()
        },
        width: Val::Px(24.0),
        height: Val::Px(24.0),
        border: UiRect::all(Val::Px(2.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgb(0.0, 0.48, 1.0).into(),
    border_color: Color::WHITE.into(),
    border_radius: BorderRadius::all(Val::Px(12.0)),
    ..default()
}
```

#### Typography Hierarchy System
**Reference**: `./docs/bevy/examples/ui/text.rs:25-55` - Multi-level text styling and sizing
**Reference**: `./docs/bevy/examples/ui/text.rs:80-110` - Text alignment and color configuration
```rust
// Display name with large bold styling
TextBundle::from_section(
    user_profile.display_name.clone(),
    TextStyle {
        font: font_bold.clone(),
        font_size: 28.0,
        color: Color::WHITE,
    },
).with_style(Style {
    margin: UiRect::bottom(Val::Px(4.0)),
    ..default()
})

// Combined username and email with separator
TextBundle::from_section(
    format!("{} · {}", user_profile.username, user_profile.email),
    TextStyle {
        font: font_regular.clone(),
        font_size: 14.0,
        color: Color::rgba(0.53, 0.53, 0.53, 1.0), // Medium gray
    },
).with_style(Style {
    margin: UiRect::bottom(Val::Px(20.0)),
    ..default()
})
```

#### Subscription Status Banner
**Reference**: `./docs/bevy/examples/ui/ui.rs:300-330` - Status panel styling with rounded corners
**Reference**: `./docs/bevy/examples/ui/text.rs:140-170` - Multi-line text in containers
```rust
// Subscription status banner container
NodeBundle {
    style: Style {
        width: Val::Percent(100.0),
        padding: UiRect::all(Val::Px(16.0)),
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        ..default()
    },
    background_color: Color::rgba(0.15, 0.15, 0.15, 1.0).into(), // Dark gray panel
    border_radius: BorderRadius::all(Val::Px(8.0)),
    ..default()
}

// Dynamic subscription status text
TextBundle::from_section(
    subscription_status.display_message(),
    TextStyle {
        font: font_regular.clone(),
        font_size: 13.0,
        color: Color::rgba(0.85, 0.85, 0.85, 1.0), // Light gray
    },
).with_text_alignment(TextAlignment::Center)
```

### Profile Image Management System

#### Image Upload and Processing
**Reference**: `./docs/bevy/examples/asset_loading/asset_loading.rs:140-180` - Dynamic asset loading and caching
```rust
// Profile image component with upload capabilities
#[derive(Component)]
pub struct ProfileImage {
    pub user_id: String,
    pub image_handle: Handle<Image>,
    pub upload_state: ImageUploadState,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ImageUploadState {
    None,
    Uploading(f32), // Progress percentage
    Completed,
    Failed(String),
}

// Image upload system
fn profile_image_upload_system(
    mut commands: Commands,
    mut query: Query<&mut ProfileImage>,
    asset_server: Res<AssetServer>,
    mut upload_events: EventReader<ProfileImageUploadEvent>,
) {
    for event in upload_events.iter() {
        if let Ok(mut profile_image) = query.get_mut(event.entity) {
            profile_image.upload_state = ImageUploadState::Uploading(0.0);
            
            // Spawn async upload task
            commands.spawn_task(async move {
                upload_profile_image(event.user_id.clone(), event.image_data.clone()).await
            });
        }
    }
}
```

#### Fallback Avatar System
**Reference**: `./docs/bevy/examples/ui/text.rs:200-230` - Generated text content for fallbacks
```rust
// Generate initials fallback when no profile image
fn generate_initials_avatar(display_name: &str) -> String {
    display_name
        .split_whitespace()
        .filter_map(|word| word.chars().next())
        .take(2)
        .collect::<String>()
        .to_uppercase()
}

// Initials avatar component when image unavailable
TextBundle::from_section(
    generate_initials_avatar(&user_profile.display_name),
    TextStyle {
        font: font_bold.clone(),
        font_size: 48.0,
        color: Color::WHITE,
    },
).with_style(Style {
    justify_content: JustifyContent::Center,
    align_items: AlignItems::Center,
    ..default()
})
```

### Architecture Notes

#### Component Structure
- **ProfileDisplayComponent**: Main component for entire profile section
- **UserAvatarComponent**: Circular avatar with verification badge
- **SubscriptionBannerComponent**: Dynamic status banner
- **ProfileImageComponent**: Image management and upload state

#### User Data Integration
- **UserProfile**: Core data structure with display name, username, email
- **SubscriptionStatus**: Dynamic subscription state and messaging
- **AvatarAssets**: Cached avatar images and fallback resources
- **VerificationState**: User verification status and badge display

#### Interactive Elements
**Reference**: `./docs/bevy/examples/input/mouse_input.rs:35-65` - Click detection for image upload
- Avatar click for image upload dialog
- Hover states for interactive elements
- Verification badge hover for additional information

### Quality Standards
- High-resolution image support with proper scaling
- Secure image upload with validation and sanitization
- Efficient avatar caching and loading states
- Responsive layout adaptation for different panel sizes
- WCAG AA compliance for text contrast and readability

### Integration Points
- User authentication system for profile data
- Subscription management system for status banner
- Asset loading system for profile images
- Theme system integration for colors and typography
- Upload service integration for profile image changes