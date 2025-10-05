# About Menu Specification

## Overview
The About Menu represents the application information and meta-navigation interface. This clean, centered layout serves as the primary source of application identity, version information, and external resource access.

## Layout Architecture
- **Base Layout**: Consistent tabbed navigation with "About" tab active
- **Content Area**: Single-pane centered layout (full width utilization)
- **Vertical Hierarchy**: Logo → Title → Version → Copyright → Action Buttons

## Visual Design Elements

### Application Branding
- **Logo**: Geometric red/coral design with angular, layered composition
  - **Style**: Abstract pattern with multiple angular shapes arranged in a diamond-like formation
  - **Colors**: Primary red/coral (#FF5E5E or similar bright red)
  - **Design Elements**: Multiple geometric pieces with varying angles and orientations
  - **Size**: Large, prominent display (approximately 12% VMin)
  - **Position**: Horizontally centered, upper portion of content area

### Typography Hierarchy
- **Application Name**: "Raycast"
  - **Font**: Bold, large display font
  - **Size**: Primary heading scale (approximately 3.5% viewport height)
  - **Color**: High contrast white (#FFFFFF)
  - **Position**: Directly to the right of logo, vertically centered with logo

- **Version Information**: "Version 1.102.3"
  - **Font**: Regular weight, smaller than app name
  - **Color**: Medium gray text (#888888 or similar)
  - **Position**: Below application name, left-aligned with app name

- **Copyright Notice**: 
  - **Line 1**: "© Raycast Technologies Ltd."
  - **Line 2**: "2019-2025. All Rights Reserved."
  - **Font**: Small, regular weight
  - **Color**: Light gray text (#666666 or similar, lower opacity than version)
  - **Position**: Below version information, left-aligned
  - **Layout**: Two separate lines with consistent left alignment

## Visual Design Specifications

### Layout Structure
- **Content Area**: Full-width utilization with centered content alignment
- **Background**: Dark theme consistent with other settings screens (#1a1a1a or similar)
- **Content Container**: Vertically centered within available space
- **Element Spacing**: Consistent vertical spacing between logo, text, and buttons

### Navigation Tab State
- **About Tab**: Active state with darker background highlighting
- **Tab Icon**: Hand/information icon indicating About section
- **Tab Text**: "About" in lighter text color for active state
- **Inactive Tabs**: Standard gray text and background

### Logo and Branding Layout
- **Logo Position**: Left side of horizontal brand container
- **Text Position**: Right side of horizontal brand container, vertically centered
- **Container Alignment**: Centered horizontally within main content area
- **Brand Grouping**: Logo and text treated as single cohesive unit

### Text Color Hierarchy
- **Primary Text (App Name)**: Pure white (#FFFFFF) for maximum contrast
- **Secondary Text (Version)**: Medium gray for clear but subdued visibility
- **Tertiary Text (Copyright)**: Light gray for legal information readability
- **Consistent Theming**: All text follows established dark theme patterns

### Button Layout Specifications
- **Button Container**: Horizontal arrangement at bottom of content area
- **Button Spacing**: Equal spacing between three buttons
- **Button Width**: Consistent width for visual balance
- **Button Alignment**: Centered as a group within content area
- **Button Styling**: Secondary button appearance with rounded corners

## Interactive Elements

### Action Button Group
- **Position**: Bottom of content area, horizontally centered
- **Layout**: Three-button horizontal arrangement with equal spacing
- **Button Styles**: Consistent styling with rounded corners and hover states

#### Acknowledgements Button
- **Text**: "Acknowledgements"
- **Function**: Opens acknowledgements/credits interface
- **Style**: Secondary button styling (subtle background)
- **Purpose**: Display third-party libraries, contributors, and legal notices

#### Visit Website Button  
- **Text**: "Visit Website"
- **Function**: Opens primary Raycast website in default browser
- **Style**: Secondary button styling
- **Target**: https://raycast.com (or equivalent primary URL)

#### Send Feedback Button
- **Text**: "Send Feedback"
- **Function**: Opens feedback submission interface or email client
- **Style**: Secondary button styling
- **Purpose**: Direct user feedback channel to development team

## Functional Requirements

### Version Management System
- **Dynamic Version Display**: Version number populated from build/deployment metadata
- **Build Information**: Support for build numbers, commit hashes, or additional version details
- **Update Detection**: Potential integration with update checking system
- **Beta/Development Indicators**: Support for pre-release version labeling

### External Navigation System
- **URL Handling**: Secure opening of external URLs in default browser
- **Feedback Integration**: Email client integration or web-based feedback forms
- **Acknowledgements System**: Dynamic loading of dependency and contributor information
- **Analytics Integration**: Optional tracking of external link usage

### Legal and Compliance
- **Copyright Management**: Dynamic copyright year calculation
- **Legal Notice System**: Integration with terms of service and privacy policy
- **Open Source Compliance**: Automatic generation of license acknowledgements
- **Trademark Protection**: Proper trademark usage and attribution

## Bevy Implementation Examples

### Centered Layout System
- Reference: `./docs/bevy/examples/ui/flex_layout.rs` - Centered flex container implementation
- Reference: `./docs/bevy/examples/ui/ui.rs` - Container alignment and positioning

### Image Loading and Display
- Reference: `./docs/bevy/examples/asset_loading/asset_loading.rs` - Logo asset management
- Reference: `./docs/bevy/examples/ui/ui_texture_atlas.rs` - High-resolution image display

### Typography Hierarchy
- Reference: `./docs/bevy/examples/ui/text.rs` - Multi-level text styling and sizing
- Reference: `./docs/bevy/examples/ui/text_debug.rs` - Text alignment and positioning

### Button Group Layout
- Reference: `./docs/bevy/examples/ui/button.rs` - Button interaction and styling
- Reference: `./docs/bevy/examples/ui/ui.rs` - Horizontal button group arrangement

### External URL Handling
- Reference: `./docs/bevy/examples/app/return_after_run.rs` - External system integration
- Reference: `./docs/bevy/examples/async_tasks/async_compute.rs` - Asynchronous external operations

### Dynamic Version Display
- Reference: `./docs/bevy/examples/reflection/reflection.rs` - Runtime metadata access
- Reference: `./docs/bevy/examples/ui/text.rs` - Dynamic text content updates

### Copyright and Legal Text
- Reference: `./docs/bevy/examples/time/time.rs` - Dynamic date calculation for copyright years
- Reference: `./docs/bevy/examples/ui/text.rs` - Multi-line text layout and formatting

## State Management Requirements

### Application Metadata
- **Version Tracking**: Runtime access to application version information
- **Build Metadata**: Integration with build system for dynamic information
- **Environment Detection**: Development vs. production environment indicators
- **Update Status**: Optional integration with application update mechanisms

### External Integration State
- **Browser Availability**: Detection of default browser for URL opening
- **Email Client Integration**: Availability and configuration of email client
- **Network Connectivity**: Optional network status for external operations
- **Analytics Consent**: User preference management for usage tracking

## Accessibility Requirements

### Screen Reader Support
- **Semantic Structure**: Proper heading hierarchy and landmark regions
- **Image Alt Text**: Descriptive alt text for logo and visual elements
- **Button Labels**: Clear, descriptive button labels for screen readers
- **Navigation Context**: Clear indication of current tab and navigation state

### Keyboard Navigation
- **Tab Order**: Logical tab order through interactive elements
- **Keyboard Shortcuts**: Optional keyboard shortcuts for common actions
- **Focus Indicators**: Clear visual focus indicators for all interactive elements
- **Escape Handling**: Proper keyboard navigation for modal or overlay elements

### Visual Accessibility
- **Color Contrast**: WCAG AA compliance for all text elements
- **Focus Indicators**: High contrast focus indicators for all interactive elements
- **Scalability**: Proper scaling behavior for increased font sizes
- **High Contrast Mode**: Alternative styling for high contrast system settings

## Animation and Micro-Interactions

### Logo Presentation
- **Entrance Animation**: Subtle fade-in or scale animation on menu load
- **Hover Effects**: Optional subtle animation on logo hover
- **Loading States**: Placeholder or skeleton loading for dynamic content
- **Transition Smoothness**: Smooth transitions between menu tabs

### Button Interactions
- **Hover States**: Subtle color and scale changes on button hover
- **Click Feedback**: Brief animation or state change on button activation
- **Loading States**: Visual feedback during external operation initiation
- **Success/Error States**: Optional feedback for successful or failed operations

### Layout Transitions
- **Tab Switching**: Smooth transitions when navigating to/from About tab
- **Content Loading**: Smooth loading of version and metadata information
- **Responsive Behavior**: Smooth adaptation to different window sizes
- **State Persistence**: Smooth restoration of previous application state

## Error Handling Requirements

### External Operation Failures
- **URL Opening Errors**: Graceful handling of browser opening failures
- **Email Client Errors**: Fallback options when email client unavailable
- **Network Failures**: Appropriate user feedback for connectivity issues
- **Permission Errors**: Clear messaging for system permission requirements

### Data Loading Failures
- **Version Information**: Fallback display when version metadata unavailable
- **Copyright Information**: Static fallback for dynamic copyright calculation
- **Build Metadata**: Graceful degradation when build information unavailable
- **Asset Loading**: Placeholder or fallback for missing logo or images

## Bevy Implementation Details

### About Menu Component Architecture

```rust
use bevy::{prelude::*, window::PrimaryWindow};

// About menu specific components
#[derive(Component, Reflect)]
pub struct AboutMenu;

#[derive(Component, Reflect)]
pub struct AppLogo {
    pub texture_handle: Handle<Image>,
    pub is_loaded: bool,
}

#[derive(Component, Reflect)]
pub struct AppBrandingSection {
    pub app_name: String,
    pub version: String,
    pub build_info: BuildInformation,
}

#[derive(Component, Reflect)]
pub struct CopyrightSection {
    pub company_name: String,
    pub start_year: u16,
    pub current_year: u16,
}

#[derive(Component, Reflect)]
pub struct ActionButton {
    pub button_type: ActionButtonType,
    pub url: Option<String>,
    pub is_loading: bool,
}

#[derive(Component, Reflect, Clone, Copy, PartialEq)]
pub enum ActionButtonType {
    Acknowledgements,
    VisitWebsite,
    SendFeedback,
}

#[derive(Clone, Reflect)]
pub struct BuildInformation {
    pub version_major: u16,
    pub version_minor: u16,
    pub version_patch: u16,
    pub build_number: Option<String>,
    pub commit_hash: Option<String>,
    pub build_date: Option<String>,
    pub is_development: bool,
}

// Animation components for About menu
#[derive(Component, Reflect)]
pub struct FadeInAnimation {
    pub current_alpha: f32,
    pub target_alpha: f32,
    pub animation_speed: f32,
}

#[derive(Component, Reflect)]
pub struct ScaleAnimation {
    pub current_scale: f32,
    pub target_scale: f32,
    pub animation_speed: f32,
}
```

### Resource Management for Application Metadata

```rust
// Global application information resource
#[derive(Resource, Reflect)]
pub struct ApplicationInfo {
    pub name: String,
    pub version: String,
    pub build_info: BuildInformation,
    pub copyright_info: CopyrightInfo,
    pub external_urls: ExternalUrls,
    pub legal_info: LegalInformation,
}

#[derive(Clone, Reflect)]
pub struct CopyrightInfo {
    pub company_name: String,
    pub start_year: u16,
    pub auto_update_year: bool,
    pub additional_notices: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct ExternalUrls {
    pub website: String,
    pub feedback_email: Option<String>,
    pub feedback_form: Option<String>,
    pub support_url: Option<String>,
    pub privacy_policy: Option<String>,
}

#[derive(Clone, Reflect)]
pub struct LegalInformation {
    pub license_text: String,
    pub third_party_licenses: Vec<ThirdPartyLicense>,
    pub trademark_notices: Vec<String>,
}

#[derive(Clone, Reflect)]
pub struct ThirdPartyLicense {
    pub name: String,
    pub version: String,
    pub license_type: String,
    pub license_text: String,
    pub homepage: Option<String>,
}
```

### Event System for External Actions

```rust
// About menu specific events
#[derive(Event, Reflect)]
pub enum AboutMenuEvent {
    AcknowledgementsRequested,
    WebsiteVisitRequested(String),
    FeedbackRequested,
    ExternalUrlOpenRequested(String),
    CopyrightYearUpdated(u16),
    BuildInfoUpdated(BuildInformation),
}

#[derive(Event, Reflect)]
pub struct ExternalOperationResult {
    pub operation: ExternalOperation,
    pub success: bool,
    pub error_message: Option<String>,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum ExternalOperation {
    OpenUrl(String),
    OpenEmailClient(String),
    ShowAcknowledgements,
}

#[derive(Event, Reflect)]
pub struct AssetLoadingEvent {
    pub asset_type: AssetType,
    pub path: String,
    pub loaded: bool,
}

#[derive(Clone, Reflect, PartialEq)]
pub enum AssetType {
    Logo,
    Icon,
    Font,
}
```

### System Architecture with External Integrations

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum AboutMenuSystems {
    Input,
    ExternalOperations,
    StateUpdate,
    Animation,
    Rendering,
}

impl Plugin for AboutMenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Resources
            .init_resource::<ApplicationInfo>()
            .init_resource::<SystemIntegration>()
            
            // Events
            .add_event::<AboutMenuEvent>()
            .add_event::<ExternalOperationResult>()
            .add_event::<AssetLoadingEvent>()
            
            // System ordering
            .configure_sets(Update, (
                AboutMenuSystems::Input,
                AboutMenuSystems::ExternalOperations,
                AboutMenuSystems::StateUpdate,
                AboutMenuSystems::Animation,
                AboutMenuSystems::Rendering,
            ).chain())
            
            // Systems
            .add_systems(Startup, (
                setup_about_menu,
                load_application_metadata,
                initialize_copyright_year,
            ))
            
            .add_systems(Update, (
                handle_button_interactions,
                handle_keyboard_shortcuts,
            ).in_set(AboutMenuSystems::Input))
            
            .add_systems(Update, (
                process_external_url_requests,
                handle_email_client_integration,
                manage_acknowledgements_display,
            ).in_set(AboutMenuSystems::ExternalOperations))
            
            .add_systems(Update, (
                update_about_menu_state,
                sync_application_metadata,
                update_copyright_year,
            ).in_set(AboutMenuSystems::StateUpdate))
            
            .add_systems(Update, (
                animate_logo_entrance,
                animate_button_hover_states,
                animate_content_transitions,
            ).in_set(AboutMenuSystems::Animation))
            
            .add_systems(Update, (
                update_version_display,
                update_button_states,
                handle_asset_loading_states,
            ).in_set(AboutMenuSystems::Rendering));
    }
}

// External system integration resource
#[derive(Resource)]
pub struct SystemIntegration {
    pub default_browser: Option<String>,
    pub email_client_available: bool,
    pub network_status: NetworkStatus,
}

#[derive(Clone, PartialEq)]
pub enum NetworkStatus {
    Connected,
    Disconnected,
    Limited,
    Unknown,
}
```

### Centered Layout Implementation with Constraints

```rust
fn setup_about_menu(
    mut commands: Commands, 
    asset_server: Res<AssetServer>,
    app_info: Res<ApplicationInfo>,
) {
    // Root container with proper flex constraints
    commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            max_width: Val::Px(800.0), // Constrain maximum width
            max_height: Val::Px(900.0), // Constrain maximum height
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0, // CRITICAL: Prevent expansion
            margin: UiRect::horizontal(Val::Auto), // Center horizontally
            padding: UiRect::all(Val::Px(40.0)),
            ..default()
        },
        BackgroundColor(Color::srgb(0.1, 0.1, 0.1)),
        AboutMenu,
    )).with_children(|parent| {
        
        // Content container with constrained growth
        parent.spawn((
            Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(600.0), // Constrain content width
                height: Val::Auto,
                max_height: Val::Px(700.0), // Constrain content height
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                flex_grow: 0.0,
                row_gap: Val::Px(32.0),
                ..default()
            },
        )).with_children(|content_parent| {
            
            // Logo and branding section
            spawn_branding_section(content_parent, &asset_server, &app_info);
            
            // Version and copyright information
            spawn_information_section(content_parent, &asset_server, &app_info);
            
            // Action buttons
            spawn_action_buttons(content_parent, &asset_server);
        });
    });
}

fn spawn_branding_section(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
    app_info: &ApplicationInfo,
) {
    parent.spawn((
        Node {
            width: Val::Auto,
            height: Val::Auto,
            max_width: Val::Px(400.0), // Constrain branding section
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            column_gap: Val::Px(20.0),
            flex_grow: 0.0,
            ..default()
        },
        AppBrandingSection {
            app_name: app_info.name.clone(),
            version: app_info.version.clone(),
            build_info: app_info.build_info.clone(),
        },
    )).with_children(|branding_parent| {
        
        // Application logo with fade-in animation
        branding_parent.spawn((
            ImageNode::new(asset_server.load("icons/app_logo.png")),
            Node {
                width: Val::Px(120.0),
                height: Val::Px(120.0),
                max_width: Val::Px(120.0), // Prevent logo expansion
                max_height: Val::Px(120.0),
                flex_grow: 0.0,
                ..default()
            },
            AppLogo {
                texture_handle: asset_server.load("icons/app_logo.png"),
                is_loaded: false,
            },
            FadeInAnimation {
                current_alpha: 0.0,
                target_alpha: 1.0,
                animation_speed: 3.0,
            },
        ));
        
        // Application name and version
        branding_parent.spawn((
            Node {
                width: Val::Auto,
                height: Val::Auto,
                max_width: Val::Px(250.0), // Constrain text area
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                flex_grow: 0.0,
                ..default()
            },
        )).with_children(|text_parent| {
            
            // App name
            text_parent.spawn((
                Text::new(&app_info.name),
                TextFont {
                    font: asset_server.load("fonts/Inter-Bold.ttf"),
                    font_size: 36.0,
                    ..default()
                },
                TextColor(Color::srgb(1.0, 1.0, 1.0)),
                FadeInAnimation {
                    current_alpha: 0.0,
                    target_alpha: 1.0,
                    animation_speed: 2.5,
                },
            ));
            
            // Version info
            text_parent.spawn((
                Text::new(&format!("Version {}", app_info.version)),
                TextFont {
                    font: asset_server.load("fonts/Inter-Regular.ttf"),
                    font_size: 16.0,
                    ..default()
                },
                TextColor(Color::srgb(0.6, 0.6, 0.6)),
                FadeInAnimation {
                    current_alpha: 0.0,
                    target_alpha: 1.0,
                    animation_speed: 2.0,
                },
            ));
        });
    });
}

fn spawn_action_buttons(
    parent: &mut ChildSpawnerCommands,
    asset_server: &AssetServer,
) {
    parent.spawn((
        Node {
            width: Val::Auto,
            height: Val::Auto,
            max_width: Val::Px(500.0), // Constrain button container
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Center,
            column_gap: Val::Px(16.0),
            flex_grow: 0.0,
            ..default()
        },
    )).with_children(|buttons_parent| {
        
        let buttons = [
            (ActionButtonType::Acknowledgements, "Acknowledgements"),
            (ActionButtonType::VisitWebsite, "Visit Website"),
            (ActionButtonType::SendFeedback, "Send Feedback"),
        ];
        
        for (button_type, label) in buttons {
            buttons_parent.spawn((
                Button,
                Node {
                    width: Val::Px(140.0),
                    height: Val::Px(36.0),
                    max_width: Val::Px(140.0), // Prevent button expansion
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    border: UiRect::all(Val::Px(1.0)),
                    flex_grow: 0.0,
                    ..default()
                },
                BackgroundColor(Color::srgb(0.15, 0.15, 0.15)),
                BorderColor::all(Color::srgb(0.3, 0.3, 0.3)),
                BorderRadius::all(Val::Px(6.0)),
                ActionButton {
                    button_type,
                    url: match button_type {
                        ActionButtonType::VisitWebsite => Some("https://raycast.com".to_string()),
                        _ => None,
                    },
                    is_loading: false,
                },
                ScaleAnimation {
                    current_scale: 1.0,
                    target_scale: 1.0,
                    animation_speed: 8.0,
                },
            )).with_children(|btn_parent| {
                btn_parent.spawn((
                    Text::new(label),
                    TextFont {
                        font: asset_server.load("fonts/Inter-Regular.ttf"),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                ));
            });
        }
    });
}
```

### Animation and External Integration Systems

```rust
// Logo and content fade-in animation
fn animate_logo_entrance(
    time: Res<Time>,
    mut animation_query: Query<(&mut FadeInAnimation, &mut BackgroundColor), With<FadeInAnimation>>,
    mut text_query: Query<(&mut FadeInAnimation, &mut TextColor), (With<FadeInAnimation>, Without<BackgroundColor>)>,
) {
    // Animate background elements with fade
    for (mut fade_anim, mut bg_color) in animation_query.iter_mut() {
        if (fade_anim.current_alpha - fade_anim.target_alpha).abs() > 0.01 {
            fade_anim.current_alpha = fade_anim.current_alpha
                .lerp(fade_anim.target_alpha, fade_anim.animation_speed * time.delta_secs());
                
            // Update alpha while preserving RGB values
            let current_color = bg_color.0;
            bg_color.0 = Color::srgba(
                current_color.red(),
                current_color.green(),
                current_color.blue(),
                fade_anim.current_alpha,
            );
        }
    }
    
    // Animate text elements with fade
    for (mut fade_anim, mut text_color) in text_query.iter_mut() {
        if (fade_anim.current_alpha - fade_anim.target_alpha).abs() > 0.01 {
            fade_anim.current_alpha = fade_anim.current_alpha
                .lerp(fade_anim.target_alpha, fade_anim.animation_speed * time.delta_secs());
                
            // Update text alpha
            let current_color = text_color.0;
            text_color.0 = Color::srgba(
                current_color.red(),
                current_color.green(),
                current_color.blue(),
                fade_anim.current_alpha,
            );
        }
    }
}

// Button hover and interaction animations
fn animate_button_hover_states(
    time: Res<Time>,
    mut interaction_query: Query<
        (&Interaction, &mut ScaleAnimation, &mut BackgroundColor),
        (Changed<Interaction>, With<ActionButton>)
    >,
    mut scale_query: Query<(&mut ScaleAnimation, &mut Transform), Without<Interaction>>,
) {
    // Handle interaction state changes
    for (interaction, mut scale_anim, mut bg_color) in interaction_query.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                scale_anim.target_scale = 1.02;
                bg_color.0 = Color::srgb(0.2, 0.2, 0.2);
            }
            Interaction::Pressed => {
                scale_anim.target_scale = 0.98;
                bg_color.0 = Color::srgb(0.1, 0.1, 0.1);
            }
            Interaction::None => {
                scale_anim.target_scale = 1.0;
                bg_color.0 = Color::srgb(0.15, 0.15, 0.15);
            }
        }
    }
    
    // Apply scale animations
    for (mut scale_anim, mut transform) in scale_query.iter_mut() {
        if (scale_anim.current_scale - scale_anim.target_scale).abs() > 0.001 {
            scale_anim.current_scale = scale_anim.current_scale
                .lerp(scale_anim.target_scale, scale_anim.animation_speed * time.delta_secs());
                
            transform.scale = Vec3::splat(scale_anim.current_scale);
        }
    }
}

// External operation handling using async tasks
fn process_external_url_requests(
    mut about_events: EventReader<AboutMenuEvent>,
    mut external_results: EventWriter<ExternalOperationResult>,
    system_integration: Res<SystemIntegration>,
    task_pool: Res<AsyncComputeTaskPool>,
    mut commands: Commands,
) {
    for event in about_events.read() {
        match event {
            AboutMenuEvent::WebsiteVisitRequested(url) => {
                let url_clone = url.clone();
                let task = task_pool.spawn(async move {
                    // Simulate opening URL in default browser
                    #[cfg(target_os = "macos")]
                    {
                        std::process::Command::new("open")
                            .arg(&url_clone)
                            .output()
                            .await
                    }
                    
                    #[cfg(target_os = "windows")]
                    {
                        std::process::Command::new("cmd")
                            .args(["/C", "start", &url_clone])
                            .output()
                            .await
                    }
                    
                    #[cfg(target_os = "linux")]
                    {
                        std::process::Command::new("xdg-open")
                            .arg(&url_clone)
                            .output()
                            .await
                    }
                });
                
                commands.spawn(AsyncExternalOperationTask {
                    task,
                    operation: ExternalOperation::OpenUrl(url.clone()),
                });
            }
            
            AboutMenuEvent::FeedbackRequested => {
                // Handle feedback request (email client or web form)
                let feedback_url = "mailto:feedback@raycast.com".to_string();
                external_results.write(ExternalOperationResult {
                    operation: ExternalOperation::OpenEmailClient(feedback_url),
                    success: system_integration.email_client_available,
                    error_message: if !system_integration.email_client_available {
                        Some("No email client configured".to_string())
                    } else {
                        None
                    },
                });
            }
            
            _ => {}
        }
    }
}

#[derive(Component)]
struct AsyncExternalOperationTask {
    task: Task<Result<std::process::Output, std::io::Error>>,
    operation: ExternalOperation,
}

// Copyright year auto-update system
fn update_copyright_year(
    time: Res<Time>,
    mut copyright_query: Query<&mut CopyrightSection, Changed<CopyrightSection>>,
    mut app_info: ResMut<ApplicationInfo>,
) {
    let current_year = chrono::Utc::now().year() as u16;
    
    if app_info.copyright_info.auto_update_year && 
       app_info.copyright_info.start_year != current_year {
        
        // Update copyright year in app info
        let old_year = app_info.copyright_info.start_year;
        app_info.copyright_info.start_year = current_year;
        
        // Update all copyright section components
        for mut copyright in copyright_query.iter_mut() {
            copyright.current_year = current_year;
        }
        
        info!("Updated copyright year from {} to {}", old_year, current_year);
    }
}
```

### Testing Strategy for About Menu

```rust
#[cfg(test)]
mod about_menu_tests {
    use super::*;
    
    #[test]
    fn test_about_menu_initialization() {
        let mut app = setup_test_app();
        
        // Initialize with test application info
        let test_app_info = ApplicationInfo {
            name: "Test App".to_string(),
            version: "1.0.0".to_string(),
            build_info: BuildInformation {
                version_major: 1,
                version_minor: 0,
                version_patch: 0,
                build_number: Some("123".to_string()),
                commit_hash: None,
                build_date: None,
                is_development: true,
            },
            copyright_info: CopyrightInfo {
                company_name: "Test Company".to_string(),
                start_year: 2023,
                auto_update_year: true,
                additional_notices: vec![],
            },
            external_urls: ExternalUrls {
                website: "https://test.com".to_string(),
                feedback_email: Some("feedback@test.com".to_string()),
                feedback_form: None,
                support_url: None,
                privacy_policy: None,
            },
            legal_info: LegalInformation {
                license_text: "MIT License".to_string(),
                third_party_licenses: vec![],
                trademark_notices: vec![],
            },
        };
        
        app.world_mut().insert_resource(test_app_info);
        app.update();
        
        // Verify about menu components were spawned
        let about_menu_count = app.world().query::<&AboutMenu>().iter(app.world()).count();
        assert_eq!(about_menu_count, 1);
        
        // Verify branding section was created
        let branding_count = app.world().query::<&AppBrandingSection>().iter(app.world()).count();
        assert_eq!(branding_count, 1);
    }
    
    #[test]
    fn test_external_url_handling() {
        let mut app = setup_test_app();
        
        // Send website visit request
        app.world_mut().resource_mut::<Events<AboutMenuEvent>>()
            .write(AboutMenuEvent::WebsiteVisitRequested("https://test.com".to_string()));
        
        app.update();
        
        // Verify external operation task was created
        let external_tasks = app.world().query::<&AsyncExternalOperationTask>().iter(app.world()).count();
        assert!(external_tasks > 0);
    }
    
    #[test]
    fn test_button_interaction_states() {
        let mut app = setup_test_app();
        
        // Create action button
        let button_entity = app.world_mut().spawn((
            Button,
            ActionButton {
                button_type: ActionButtonType::VisitWebsite,
                url: Some("https://test.com".to_string()),
                is_loading: false,
            },
            ScaleAnimation {
                current_scale: 1.0,
                target_scale: 1.0,
                animation_speed: 8.0,
            },
            Interaction::None,
        )).id();
        
        // Simulate hover interaction
        app.world_mut().get_mut::<Interaction>(button_entity).unwrap().clone_from(&Interaction::Hovered);
        
        app.update();
        
        // Verify scale animation target changed
        let scale_anim = app.world().get::<ScaleAnimation>(button_entity).unwrap();
        assert!(scale_anim.target_scale > 1.0);
    }
}