use bevy::prelude::*;
use super::types::IconSize;

/// Generic color scheme for icon theming
///
/// Provides a consistent color palette for icon rendering
/// that adapts to light/dark themes.
#[derive(Debug, Clone)]
pub struct ThemeColors {
    /// Primary accent color for highlights and focus states
    pub accent: Color,
    /// Background color for icon containers
    pub background: Color,
    /// Foreground/text color for icon content
    pub foreground: Color,
    /// Muted/secondary color for disabled or inactive states
    pub muted: Color,
}

/// Icon theme resource with system theme detection
///
/// Automatically detects system dark/light mode preferences on macOS, Windows, and Linux.
/// Provides appropriate color schemes and default icon sizing.
///
/// # Platform Detection
/// - **macOS**: Uses `defaults read -g AppleInterfaceStyle`
/// - **Windows**: Queries registry `HKCU\SOFTWARE\Microsoft\Windows\CurrentVersion\Themes\Personalize\AppsUseLightTheme`
/// - **Linux**: Checks `GTK_THEME` environment variable, falls back to `gsettings`
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::icons::IconTheme;
///
/// fn setup(mut commands: Commands) {
///     commands.insert_resource(IconTheme::default());
/// }
/// ```
#[derive(Resource, Debug, Clone)]
pub struct IconTheme {
    /// Whether system is using dark theme
    pub is_dark_theme: bool,
    /// Default icon size for rendering
    pub icon_size: IconSize,
    /// Color scheme for current theme
    pub theme_colors: ThemeColors,
}

impl Default for IconTheme {
    fn default() -> Self {
        Self {
            is_dark_theme: Self::detect_system_theme(),
            icon_size: IconSize::Medium,
            theme_colors: Self::get_system_colors(),
        }
    }
}

impl IconTheme {
    /// Detect system dark/light theme preference (cross-platform)
    ///
    /// # Platform Behavior
    /// - macOS: Reads AppleInterfaceStyle global default
    /// - Windows: Checks AppsUseLightTheme registry value (0x0 = dark)
    /// - Linux: Checks GTK_THEME env var, falls back to gsettings
    /// - Other platforms: Returns false (light mode)
    pub fn detect_system_theme() -> bool {
        #[cfg(target_os = "macos")]
        {
            std::process::Command::new("defaults")
                .args(["read", "-g", "AppleInterfaceStyle"])
                .output()
                .map(|output| String::from_utf8_lossy(&output.stdout).contains("Dark"))
                .unwrap_or(false)
        }
        #[cfg(target_os = "windows")]
        {
            std::process::Command::new("reg")
                .args([
                    "query",
                    "HKCU\\SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize",
                    "/v",
                    "AppsUseLightTheme",
                ])
                .output()
                .map(|output| {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains("0x0")
                })
                .unwrap_or(false)
        }
        #[cfg(target_os = "linux")]
        {
            std::env::var("GTK_THEME")
                .map(|theme| theme.to_lowercase().contains("dark"))
                .or_else(|_| {
                    std::process::Command::new("gsettings")
                        .args(["get", "org.gnome.desktop.interface", "gtk-theme"])
                        .output()
                        .map(|output| {
                            String::from_utf8_lossy(&output.stdout)
                                .to_lowercase()
                                .contains("dark")
                        })
                })
                .unwrap_or(false)
        }
        #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
        {
            false
        }
    }

    /// Get system-appropriate theme colors
    ///
    /// Returns dark or light theme colors based on system detection.
    pub fn get_system_colors() -> ThemeColors {
        if Self::detect_system_theme() {
            // Dark theme colors
            ThemeColors {
                accent: Color::srgba(0.39, 0.59, 1.0, 1.0),      // Blue accent
                background: Color::srgba(0.1, 0.1, 0.1, 1.0),     // Dark gray
                foreground: Color::srgba(1.0, 1.0, 1.0, 1.0),     // White
                muted: Color::srgba(0.6, 0.6, 0.6, 1.0),          // Light gray
            }
        } else {
            // Light theme colors
            ThemeColors {
                accent: Color::srgba(0.0, 0.48, 1.0, 1.0),        // Darker blue
                background: Color::srgba(0.95, 0.95, 0.95, 1.0),  // Light gray
                foreground: Color::srgba(0.0, 0.0, 0.0, 1.0),     // Black
                muted: Color::srgba(0.4, 0.4, 0.4, 1.0),          // Dark gray
            }
        }
    }
}
