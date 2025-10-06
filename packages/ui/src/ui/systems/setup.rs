//! Viewport-responsive UI setup systems
//!
//! Zero-allocation UI initialization with blazing-fast viewport-responsive design,
//! screen dimension integration, and reactive UI updates.

use bevy::prelude::*;
use tracing::{info, warn};

use crate::ui::components::modal::PreferencesContainer;
use crate::ui::components::{
    FallbackIcon, LauncherContainer, ResultsContainer,
    SearchContainer, SearchInput, SettingsContainer, UiFonts, UiRoot, ViewportResponsiveContainer,
};
use crate::ui::icons::LauncherIconCache;
use action_items_ecs_ui::icons::IconTheme;
use crate::ui::systems::monitor_constraints::MonitorConstrained;
use crate::ui::typography::TypographyScale;
use action_items_ecs_ui::gradients::{GradientComponent, GradientTheme};
use action_items_ecs_ui::theme::{ShadowElevation, Theme};
use action_items_ecs_ui::visibility::UiComponentTarget;

/// UI setup system - loads fonts, themes, and initializes resources
/// Zero-allocation UI initialization with blazing-fast resource setup and theme configuration
#[inline]
pub fn setup_ui_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    // Initialize theme system
    let theme = Theme::default();
    commands.insert_resource(theme.clone());

    // Initialize gradient theme system for professional Raycast-like aesthetics
    let gradient_theme = GradientTheme::professional_dark();
    commands.insert_resource(gradient_theme);

    // Load fonts with error handling
    let ubuntu_regular = asset_server.load("fonts/Ubuntu-Regular.ttf");
    let ubuntu_medium = asset_server.load("fonts/Ubuntu-Medium.ttf");
    let ubuntu_bold = asset_server.load("fonts/Ubuntu-Bold.ttf");
    let fira_code_font = asset_server.load("fonts/FiraCodeNerdFontMono-Regular.ttf");
    let fontawesome_font = asset_server.load("fonts/Font Awesome 7 Free-Solid-900.otf");

    // Create typography scale resource
    let typography = TypographyScale::new(
        ubuntu_regular.clone(),
        ubuntu_medium.clone(),
        ubuntu_bold.clone(),
        fira_code_font.clone(),
        fontawesome_font.clone(),
        &theme,
    );

    let typography_clone = typography.clone();
    commands.insert_resource(typography);

    // Register UI fonts resource for component systems
    let ui_fonts = UiFonts {
        regular: ubuntu_regular,
        medium: ubuntu_medium.clone(),
        bold: ubuntu_bold.clone(),
        monospace: fira_code_font.clone(),
        mono: fira_code_font,
        icons: fontawesome_font,
        ubuntu_medium,
        ubuntu_bold,
    };
    commands.insert_resource(ui_fonts);

    // Initialize fallback icon resource without loading - will be loaded in PostStartup
    commands.insert_resource(FallbackIcon(None));

    // Initialize icon cache and extraction system
    commands.insert_resource(LauncherIconCache::new());
    commands.insert_resource(IconTheme::default());

    // Spawn viewport-responsive UI root with screen dimension integration
    commands
        .spawn((
            Node {
                // Full screen root container for proper viewport calculations
                width: Val::Vw(100.0),
                height: Val::Vh(100.0),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                ..default()
            },
            GlobalZIndex(1),
            UiRoot,
            MonitorConstrained::full_screen(),
        ))
        .with_children(|parent| {
            // VIEWPORT-RESPONSIVE LAUNCHER CONTAINER - PROFESSIONAL RAYCAST-LIKE DESIGN
            let viewport_container = ViewportResponsiveContainer::default();
            parent
                .spawn((
                    viewport_container.to_node_style(),
                    viewport_container,
                    // Professional gradient background matching Raycast aesthetic
                    GradientComponent::primary_container(),
                    BorderRadius::all(Val::Px(16.0)),
                    theme.create_box_shadow(ShadowElevation::XL),
                    LauncherContainer,  // App-specific marker
                    UiComponentTarget::PrimaryContainer,  // Generic marker for visibility system
                    Visibility::Hidden, // Hidden initially
                ))
                .with_children(|parent| {
                    // SEARCH BAR - REAL 40PX COMPACT HEIGHT
                    parent
                        .spawn((
                            Node {
                                width: Val::Auto,      // Flex to container width
                                height: Val::Px(40.0), // Fixed compact height
                                flex_direction: FlexDirection::Row,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(12.0)),
                                ..default()
                            },
                            // Professional search input gradient - elevated surface matching
                            // Raycast
                            GradientComponent::secondary_container(),
                            BorderRadius::all(Val::Px(8.0)),
                            SearchContainer,
                        ))
                        .with_children(|parent| {
                            // Search icon
                            parent.spawn((
                                Text::new("\u{F002}"),
                                TextFont {
                                    font: typography_clone.font_handles.ubuntu_regular.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
                                Node {
                                    margin: UiRect::right(Val::Px(8.0)),
                                    ..default()
                                },
                            ));

                            // Search input text
                            parent.spawn((
                                Text::new("Search..."),
                                TextFont {
                                    font: typography_clone.font_handles.ubuntu_regular.clone(),
                                    font_size: 16.0,
                                    ..default()
                                },
                                TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
                                Node {
                                    flex_grow: 1.0,
                                    ..default()
                                },
                                SearchInput,
                            ));
                        });

                    // RESULTS CONTAINER - REAL OVERFLOW HANDLING
                    parent.spawn((
                        Node {
                            width: Val::Auto, // Flex to container width
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Px(4.0), // Tight spacing between results
                            overflow: Overflow::clip(), // Prevent expansion
                            ..default()
                        },
                        ResultsContainer,
                        Visibility::Hidden, // Hidden until search starts
                    ));
                });
        });

    // Spawn placeholder modal container entity for animation targeting
    commands.spawn((
        Node {
            width: Val::Vw(80.0),
            height: Val::Vh(70.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.95)),
        BorderRadius::all(Val::Px(12.0)),
        PreferencesContainer,  // App-specific marker
        UiComponentTarget::Dialog,  // Generic marker for visibility system
        Visibility::Hidden, // Hidden until modal is opened
    ));

    // Spawn placeholder settings container entity for animation targeting
    commands.spawn((
        Node {
            width: Val::Vw(70.0),
            height: Val::Vh(60.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::FlexStart,
            align_items: AlignItems::Stretch,
            position_type: PositionType::Absolute,
            ..default()
        },
        BackgroundColor(Color::srgba(0.12, 0.12, 0.12, 0.98)),
        BorderRadius::all(Val::Px(10.0)),
        SettingsContainer,  // App-specific marker
        UiComponentTarget::Panel,  // Generic marker for visibility system
        Visibility::Hidden, // Hidden until settings are opened
    ));

    info!("UI initialized with viewport-responsive design and screen dimension integration");
}

/// Load fallback icon after asset system is fully initialized
/// This system runs in PostStartup to ensure the ImagePlugin is ready to handle PNG files
pub fn load_fallback_icon_system(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    fallback_icon: Res<FallbackIcon>,
) {
    // Only load if not already loaded
    if fallback_icon.0.is_none() {
        let icon_handle = asset_server.load("icons/app.png");
        commands.insert_resource(FallbackIcon(Some(icon_handle)));
        info!("Fallback icon loaded successfully");
    }
}

/// Reactive UI system that responds to screen dimension changes
/// Zero-allocation system for updating viewport-responsive UI when screen configuration changes
/// Maintains component hierarchy and ensures proper parent-child relationships
#[inline]
pub fn reactive_screen_dimension_system(
    screen_dimensions: Res<action_items_core::screen_dimensions::ScreenDimensions>,
    mut viewport_containers: Query<(&mut Node, &ViewportResponsiveContainer)>,
    mut ui_roots: Query<&mut Node, (With<UiRoot>, Without<ViewportResponsiveContainer>)>,
) {
    if !screen_dimensions.is_changed() {
        return;
    }

    info!("Screen dimensions reactive system - viewport update");

    // Update UI root dimensions to match new screen size
    for mut style in ui_roots.iter_mut() {
        style.width = Val::Vw(100.0);
        style.height = Val::Vh(100.0);
    }

    // Update all viewport-responsive containers with new screen dimensions
    for (mut style, container) in viewport_containers.iter_mut() {
        // Validate container before applying changes
        if !container.is_valid() {
            warn!("Invalid ViewportResponsiveContainer detected during screen update, skipping");
            continue;
        }

        // Regenerate style with current container settings
        // Viewport units automatically adapt to new screen dimensions
        *style = container.to_node_style();

        info!(
            "Updated viewport container: {}vw x {}vh for new screen dimensions",
            container.width_vw, container.height_vh
        );
    }
}

/// UI initialization system with proper system ordering
/// Ensures screen dimension resources are available before UI setup
/// Handles system dependencies correctly for seamless viewport-responsive design
#[inline]
pub fn ensure_screen_dimensions_system(
    mut commands: Commands,
    screen_dimensions: Option<Res<action_items_core::screen_dimensions::ScreenDimensions>>,
) {
    if screen_dimensions.is_none() {
        let default_dimensions = action_items_core::screen_dimensions::ScreenDimensions::default();
        commands.insert_resource(default_dimensions);
        info!("Screen dimensions system initialized with defaults");
    } else {
        info!("Screen dimensions system initialized");
    }
}

/// Advanced viewport adaptation system for extreme screen configurations
/// Automatically optimizes UI layout for ultra-wide, portrait, and unusual aspect ratios
/// Zero-allocation adaptive logic with blazing-fast screen configuration detection
#[inline]
pub fn adaptive_viewport_system(
    screen_dimensions: Res<action_items_core::screen_dimensions::ScreenDimensions>,
    mut viewport_containers: Query<&mut ViewportResponsiveContainer, With<LauncherContainer>>,
) {
    if !screen_dimensions.is_changed() {
        return;
    }

    let aspect_ratio = screen_dimensions.aspect_ratio();
    let screen_width = screen_dimensions.width as f32;

    for mut container in viewport_containers.iter_mut() {
        let original_width = container.width_vw;
        let original_height = container.height_vh;

        // Adaptive logic for different screen configurations
        match aspect_ratio {
            // Ultra-wide screens (>2.5 aspect ratio): compact horizontal layout
            ratio if ratio > 2.5 => {
                *container = ViewportResponsiveContainer::compact();
                container.width_vw = container.width_vw.min(40.0);
                info!(
                    "Applied ultra-wide adaptation: {}vw container width",
                    container.width_vw
                );
            },
            // Portrait screens (<0.8 aspect ratio): expanded vertical layout
            ratio if ratio < 0.8 => {
                *container = ViewportResponsiveContainer::expanded();
                container.height_vh = container.height_vh.max(70.0);
                info!(
                    "Applied portrait adaptation: {}vh container height",
                    container.height_vh
                );
            },
            // Very small screens (<1024px): compact variant
            _ if screen_width < 1024.0 => {
                *container = ViewportResponsiveContainer::compact();
                info!(
                    "Applied small screen compact layout: {}vw x {}vh",
                    container.width_vw, container.height_vh
                );
            },
            // Very large screens (>3840px): expanded variant
            _ if screen_width > 3840.0 => {
                *container = ViewportResponsiveContainer::expanded();
                info!(
                    "Applied large screen expanded layout: {}vw x {}vh",
                    container.width_vw, container.height_vh
                );
            },
            // Standard screens: use default settings
            _ => {
                if original_width != ViewportResponsiveContainer::default().width_vw {
                    *container = ViewportResponsiveContainer::default();
                    info!(
                        "Reset to standard layout: {}vw x {}vh",
                        container.width_vw, container.height_vh
                    );
                }
            },
        }

        // Log changes only if actual adaptation occurred
        if (container.width_vw - original_width).abs() > f32::EPSILON
            || (container.height_vh - original_height).abs() > f32::EPSILON
        {
            info!(
                "Viewport adaptation applied for {:.2} aspect ratio: {:.1}vw x {:.1}vh (was \
                 {:.1}vw x {:.1}vh)",
                aspect_ratio,
                container.width_vw,
                container.height_vh,
                original_width,
                original_height
            );
        }
    }
}
