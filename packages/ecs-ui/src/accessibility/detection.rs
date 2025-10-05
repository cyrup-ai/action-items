use bevy::prelude::*;

use super::manager::AccessibilityManager;

#[cfg(target_os = "macos")]
mod macos {
    use objc2::msg_send;
    use objc2::rc::Retained;
    use objc2_app_kit::NSWorkspace;
    use objc2_foundation::{NSString, NSUserDefaults};

    /// macOS accessibility preferences detector
    pub struct MacOSAccessibilityDetector {
        user_defaults: Retained<NSUserDefaults>,
        workspace: Retained<NSWorkspace>,
    }

    // SAFETY: NSUserDefaults and NSWorkspace are thread-safe singletons in macOS
    // that can be safely accessed from multiple threads
    unsafe impl Send for MacOSAccessibilityDetector {}
    unsafe impl Sync for MacOSAccessibilityDetector {}

    impl MacOSAccessibilityDetector {
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            let user_defaults = NSUserDefaults::standardUserDefaults();
            let workspace = NSWorkspace::sharedWorkspace();

            Ok(Self {
                user_defaults,
                workspace,
            })
        }

        /// Check if VoiceOver (screen reader) is enabled
        pub fn is_screen_reader_active(&self) -> bool {
            // Check VoiceOver preference
            let voiceover_key = NSString::from_str("voiceOverOnOffKey");
            let voiceover_enabled: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*voiceover_key]
            };

            if voiceover_enabled {
                return true;
            }

            // Also check accessibility API enabled state
            let accessibility_enabled_key =
                NSString::from_str("accessibilityDisplayShouldShowWindowUI");
            let accessibility_ui: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*accessibility_enabled_key]
            };

            accessibility_ui || voiceover_enabled
        }

        /// Check if high contrast mode is enabled
        pub fn is_high_contrast_enabled(&self) -> bool {
            // Check increase contrast setting
            let contrast_key = NSString::from_str("increaseContrast");
            let increase_contrast: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*contrast_key]
            };

            // Also check differentiate without color (high contrast alternative)
            let differentiate_key = NSString::from_str("differentiateWithoutColor");
            let differentiate: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*differentiate_key]
            };

            increase_contrast || differentiate
        }

        /// Check if reduced motion is enabled
        pub fn is_reduced_motion_enabled(&self) -> bool {
            let motion_key = NSString::from_str("reduceMotion");
            let reduced_motion: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*motion_key]
            };

            // Also check reduce transparency which often correlates with reduced motion
            // preference
            let transparency_key = NSString::from_str("reduceTransparency");
            let reduced_transparency: bool = unsafe {
                msg_send![&self.user_defaults, boolForKey: &*transparency_key]
            };

            reduced_motion || reduced_transparency
        }

        /// Check if large text/accessibility font scaling is enabled
        pub fn is_large_text_enabled(&self) -> bool {
            // Check for dynamic type preference (text size scaling)
            let text_size_key = NSString::from_str("AppleTextScalingFactor");
            let text_scaling: f64 = unsafe {
                msg_send![&self.user_defaults, doubleForKey: &*text_size_key]
            };

            // Consider large text if scaling > 1.0 (default)
            text_scaling > 1.0
        }

        /// Check if accessibility apps are running using NSWorkspace
        pub fn is_accessibility_app_running(&self) -> bool {
            let running_apps = self.workspace.runningApplications();

            // Check for common accessibility applications
            let accessibility_bundle_ids = [
                "com.apple.VoiceOver4",
                "com.apple.VoiceOverUtility",
                "com.apple.SwitchControl",
                "com.apple.Zoom",
                "com.apple.UniversalAccessControl",
            ];

            for app in running_apps.iter() {
                if let Some(bundle_id) = app.bundleIdentifier() {
                    let bundle_str = bundle_id.to_string();
                    if accessibility_bundle_ids.iter().any(|&id| bundle_str == id) {
                        return true;
                    }
                }
            }

            false
        }

        /// Check if the current application is active using NSWorkspace
        pub fn is_current_app_active(&self) -> bool {
            let front_app = self.workspace.frontmostApplication();
            if let Some(front_bundle_id) = front_app.and_then(|app| app.bundleIdentifier()) {
                // Compare with our bundle identifier if available
                let main_bundle = objc2_foundation::NSBundle::mainBundle();
                if let Some(our_bundle_id) = main_bundle.bundleIdentifier() {
                    return front_bundle_id.to_string() == our_bundle_id.to_string();
                }
            }
            false
        }

        /// Get information about the frontmost application for accessibility context
        #[allow(dead_code)]
        pub fn get_frontmost_app_info(&self) -> Option<(String, bool)> {
            let front_app = self.workspace.frontmostApplication()?;
            let bundle_id = front_app.bundleIdentifier()?.to_string();
            let is_active = front_app.isActive();
            Some((bundle_id, is_active))
        }

        /// Get comprehensive accessibility state
        pub fn get_accessibility_state(&self) -> AccessibilityState {
            AccessibilityState {
                screen_reader_active: self.is_screen_reader_active(),
                high_contrast: self.is_high_contrast_enabled(),
                reduced_motion: self.is_reduced_motion_enabled(),
                large_text: self.is_large_text_enabled(),
                accessibility_app_running: self.is_accessibility_app_running(),
                current_app_active: self.is_current_app_active(),
            }
        }
    }

    /// Complete accessibility state information
    pub struct AccessibilityState {
        pub screen_reader_active: bool,
        pub high_contrast: bool,
        pub reduced_motion: bool,
        pub large_text: bool,
        #[allow(dead_code)]
        pub accessibility_app_running: bool,
        #[allow(dead_code)]
        pub current_app_active: bool,
    }
}

#[cfg(not(target_os = "macos"))]
mod fallback {
    #[cfg(target_os = "windows")]
    use std::mem;
    #[cfg(all(unix, not(target_os = "macos")))]
    use std::process::Command;

    #[cfg(target_os = "windows")]
    use windows_sys::Win32::UI::{
        Accessibility::{HCF_HIGHCONTRASTON, HIGHCONTRASTW},
        WindowsAndMessaging::{
            ANIMATIONINFO, SPI_GETANIMATION, SPI_GETHIGHCONTRAST, SPI_GETLOGICALDPIOVERRIDE,
            SYSTEM_PARAMETERS_INFO_ACTION, SystemParametersInfoW,
        },
    };

    /// Fallback accessibility detector for non-macOS platforms
    pub struct FallbackAccessibilityDetector {
        #[cfg(all(unix, not(target_os = "macos")))]
        _display_connection: Option<()>, // Placeholder for X11 display connection
    }

    impl FallbackAccessibilityDetector {
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            #[cfg(all(unix, not(target_os = "macos")))]
            {
                Ok(Self {
                    _display_connection: None, // Would initialize X11/Wayland connection
                })
            }

            #[cfg(not(all(unix, not(target_os = "macos"))))]
            Ok(Self {})
        }

        #[cfg(target_os = "windows")]
        pub fn is_screen_reader_active(&self) -> bool {
            // Check for common Windows screen readers
            let screen_readers = ["nvda", "jaws", "narrator"];

            for reader in &screen_readers {
                if let Ok(output) = Command::new("tasklist")
                    .args(&["/FI", &format!("IMAGENAME eq {}.exe", reader)])
                    .output()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if output_str.contains(&format!("{}.exe", reader)) {
                        return true;
                    }
                }
            }

            false
        }

        #[cfg(target_os = "windows")]
        pub fn is_high_contrast_enabled(&self) -> bool {
            // Check Windows high contrast mode using real SystemParametersInfoW API
            unsafe {
                let mut hc_info: HIGHCONTRASTW = std::mem::zeroed();
                hc_info.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;

                let result = SystemParametersInfoW(
                    SPI_GETHIGHCONTRAST,
                    hc_info.cbSize,
                    &mut hc_info as *mut _ as *mut std::ffi::c_void,
                    0,
                );

                result != 0 && (hc_info.dwFlags & HCF_HIGHCONTRASTON) != 0
            }
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        pub fn is_screen_reader_active(&self) -> bool {
            // Check for common Linux screen readers
            let screen_readers = ["orca", "speakup", "speechd-up"];

            for reader in &screen_readers {
                if let Ok(output) = Command::new("pgrep").arg(reader).output() {
                    if output.status.success() && !output.stdout.is_empty() {
                        return true;
                    }
                }
            }

            // Check AT-SPI environment variable
            if std::env::var("GNOME_ACCESSIBILITY").unwrap_or_default() == "1" {
                return true;
            }

            false
        }

        #[cfg(all(unix, not(target_os = "macos")))]
        pub fn is_high_contrast_enabled(&self) -> bool {
            // Check GNOME high contrast setting
            if let Ok(output) = Command::new("gsettings")
                .args(&["get", "org.gnome.desktop.interface", "high-contrast"])
                .output()
            {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.trim() == "true" {
                    return true;
                }
            }

            // Check KDE high contrast
            if std::env::var("KDE_HIGH_CONTRAST").unwrap_or_default() == "1" {
                return true;
            }

            // Check Qt high contrast theme
            if let Some(theme) = std::env::var("QT_STYLE_OVERRIDE").ok() {
                if theme.to_lowercase().contains("contrast") {
                    return true;
                }
            }

            false
        }

        pub fn get_accessibility_state(&self) -> AccessibilityState {
            AccessibilityState {
                screen_reader_active: self.is_screen_reader_active(),
                high_contrast: self.is_high_contrast_enabled(),
                reduced_motion: self.is_reduced_motion_enabled(),
                large_text: self.is_large_text_enabled(),
            }
        }

        fn is_reduced_motion_enabled(&self) -> bool {
            #[cfg(target_os = "windows")]
            {
                // Check Windows animation preferences using real SystemParametersInfoW API
                unsafe {
                    let mut anim_info: ANIMATIONINFO = std::mem::zeroed();
                    anim_info.cbSize = std::mem::size_of::<ANIMATIONINFO>() as u32;

                    let result = SystemParametersInfoW(
                        SPI_GETANIMATION,
                        anim_info.cbSize,
                        &mut anim_info as *mut _ as *mut std::ffi::c_void,
                        0,
                    );

                    // Reduced motion = animations disabled
                    result == 0 || anim_info.iMinAnimate == 0
                }
            }

            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Check GNOME reduced animations
                if let Ok(output) = Command::new("gsettings")
                    .args(&["get", "org.gnome.desktop.interface", "enable-animations"])
                    .output()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    return output_str.trim() == "false";
                }

                // Check for motion reduction preference in environment
                if std::env::var("REDUCE_MOTION").unwrap_or_default() == "1" {
                    return true;
                }

                false
            }

            #[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
            false
        }

        fn is_large_text_enabled(&self) -> bool {
            #[cfg(target_os = "windows")]
            {
                // Check Windows text scaling using DPI detection
                unsafe {
                    let mut dpi_x = 0u32;
                    let result = SystemParametersInfoW(
                        SPI_GETLOGICALDPIOVERRIDE,
                        0,
                        &mut dpi_x as *mut _ as *mut std::ffi::c_void,
                        0,
                    );

                    // Large text if DPI scaling > standard 96 DPI
                    result != 0 && dpi_x > 96
                }
            }

            #[cfg(all(unix, not(target_os = "macos")))]
            {
                // Check GNOME text scaling
                if let Ok(output) = Command::new("gsettings")
                    .args(&["get", "org.gnome.desktop.interface", "text-scaling-factor"])
                    .output()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    if let Ok(scale_factor) = output_str.trim().parse::<f64>() {
                        return scale_factor > 1.0;
                    }
                }

                // Check KDE text scaling
                if let Ok(output) = Command::new("kreadconfig5")
                    .args(&["--group", "General", "--key", "font"])
                    .output()
                {
                    // Basic check for font size in KDE config
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    // Look for font size indicators in the font string
                    if output_str.contains(",14")
                        || output_str.contains(",16")
                        || output_str.contains(",18")
                    {
                        return true;
                    }
                }

                false
            }

            #[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
            false
        }

        #[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
        pub fn is_screen_reader_active(&self) -> bool {
            false
        }

        #[cfg(not(any(target_os = "windows", all(unix, not(target_os = "macos")))))]
        pub fn is_high_contrast_enabled(&self) -> bool {
            false
        }
    }

    pub struct AccessibilityState {
        pub screen_reader_active: bool,
        pub high_contrast: bool,
        pub reduced_motion: bool,
        pub large_text: bool,
    }
}

// Resource for storing platform-specific detector
#[derive(Resource)]
pub struct AccessibilityDetector {
    #[cfg(target_os = "macos")]
    inner: macos::MacOSAccessibilityDetector,
    #[cfg(not(target_os = "macos"))]
    inner: fallback::FallbackAccessibilityDetector,
}

impl AccessibilityDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        #[cfg(target_os = "macos")]
        let inner = macos::MacOSAccessibilityDetector::new()?;
        #[cfg(not(target_os = "macos"))]
        let inner = fallback::FallbackAccessibilityDetector::new()?;

        Ok(Self { inner })
    }

    pub fn get_accessibility_state(&self) -> AccessibilityState {
        #[cfg(target_os = "macos")]
        {
            let state = self.inner.get_accessibility_state();
            AccessibilityState {
                screen_reader_active: state.screen_reader_active,
                high_contrast: state.high_contrast,
                reduced_motion: state.reduced_motion,
                large_text: state.large_text,
            }
        }
        #[cfg(not(target_os = "macos"))]
        {
            let state = self.inner.get_accessibility_state();
            AccessibilityState {
                screen_reader_active: state.screen_reader_active,
                high_contrast: state.high_contrast,
                reduced_motion: state.reduced_motion,
                large_text: state.large_text,
            }
        }
    }
}

/// Unified accessibility state across platforms
pub struct AccessibilityState {
    pub screen_reader_active: bool,
    pub high_contrast: bool,
    pub reduced_motion: bool,
    pub large_text: bool,
}

/// System to detect and respond to accessibility preferences with real platform APIs
pub fn detect_accessibility_preferences(
    mut accessibility_manager: ResMut<AccessibilityManager>,
    detector: Option<Res<AccessibilityDetector>>,
) {
    if let Some(detector) = detector {
        #[allow(clippy::disallowed_methods)]
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            detector.get_accessibility_state()
        })) {
            Ok(state) => {
                // Update accessibility manager with real system state
                accessibility_manager.screen_reader_active = state.screen_reader_active;
                accessibility_manager.high_contrast = state.high_contrast;
                accessibility_manager.reduced_motion = state.reduced_motion;

                // Log accessibility state changes for debugging
                tracing::debug!(
                    "Accessibility state updated: screen_reader={}, high_contrast={}, \
                     reduced_motion={}, large_text={}",
                    state.screen_reader_active,
                    state.high_contrast,
                    state.reduced_motion,
                    state.large_text
                );

                // Export accessibility metrics
                metrics::gauge!("accessibility_screen_reader_active")
                    .set(if state.screen_reader_active { 1.0 } else { 0.0 });
                metrics::gauge!("accessibility_high_contrast").set(if state.high_contrast {
                    1.0
                } else {
                    0.0
                });
                metrics::gauge!("accessibility_reduced_motion").set(if state.reduced_motion {
                    1.0
                } else {
                    0.0
                });
                metrics::gauge!("accessibility_large_text").set(if state.large_text {
                    1.0
                } else {
                    0.0
                });

                // Add announcements for screen reader users if state changed
                if state.screen_reader_active && accessibility_manager.announcements.is_empty() {
                    accessibility_manager.announcements.push(
                        "Action Items application loaded with accessibility support enabled"
                            .to_string(),
                    );
                }
            },
            Err(e) => {
                tracing::error!("Failed to detect accessibility preferences: {:?}", e);
                // Fall back to safe defaults
                accessibility_manager.screen_reader_active = false;
                accessibility_manager.high_contrast = false;
                accessibility_manager.reduced_motion = false;
            },
        }
    } else {
        tracing::warn!(
            "AccessibilityDetector resource not found - using default accessibility settings"
        );
        // Use conservative defaults when detector is not available
        accessibility_manager.screen_reader_active = false;
        accessibility_manager.high_contrast = false;
        accessibility_manager.reduced_motion = false;
    }
}
