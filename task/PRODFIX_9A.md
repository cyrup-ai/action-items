# PRODFIX_9A: Implement Linux Accessibility Detection

## OBJECTIVE
Replace placeholder accessibility detector with functional AT-SPI based implementation to support screen readers and accessibility features on Linux.

## PRIORITY
**P2 - MEDIUM (Accessibility Failure)**

## LOCATION
`packages/ecs-ui/src/accessibility/detection.rs`

## CURRENT STATE
Line 194 has a placeholder `FallbackAccessibilityDetector` struct with no implementation. Linux users with disabilities get no accessibility support.

## SUBTASK 1: Add Linux Accessibility Dependencies
Add AT-SPI (Assistive Technology Service Provider Interface) crate.

**Changes needed in** `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
atspi = "0.19"
zbus = "3.14" # AT-SPI uses D-Bus
```

## SUBTASK 2: Implement AT-SPI Connection
Create the FallbackAccessibilityDetector struct with AT-SPI connection.

**Changes needed in** `packages/ecs-ui/src/accessibility/detection.rs` **around line 194:**
```rust
#[cfg(target_os = "linux")]
pub struct FallbackAccessibilityDetector {
    connection: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        // Connect to session D-Bus
        let connection = zbus::Connection::session()?;

        Ok(Self { connection })
    }
}
```

## SUBTASK 3: Implement Screen Reader Detection
Detect if screen reader (Orca, NVDA on Wine, etc.) is active.

**Add method to FallbackAccessibilityDetector:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    pub fn detect_preferences(&self) -> AccessibilityPreferences {
        AccessibilityPreferences {
            screen_reader_active: self.is_screen_reader_active(),
            high_contrast: self.is_high_contrast_active(),
            reduced_motion: self.is_reduced_motion_active(),
            large_text: self.is_large_text_enabled(),
        }
    }

    fn is_screen_reader_active(&self) -> bool {
        // Check if org.a11y.atspi.Registry is available
        match self.check_atspi_registry() {
            Ok(is_active) => is_active,
            Err(e) => {
                warn!("Failed to check screen reader status: {}", e);
                false
            }
        }
    }

    fn check_atspi_registry(&self) -> Result<bool, Box<dyn std::error::Error>> {
        use zbus::proxy;

        // Create proxy for AT-SPI registry
        #[proxy(
            interface = "org.a11y.atspi.Registry",
            default_service = "org.a11y.atspi.Registry",
            default_path = "/org/a11y/atspi/registry"
        )]
        trait Registry {
            fn get_registered_event_listeners(&self) -> zbus::Result<Vec<(String, String)>>;
        }

        let proxy = RegistryProxyBlocking::new(&self.connection)?;

        // Check if any event listeners are registered (indicates active AT)
        match proxy.get_registered_event_listeners() {
            Ok(listeners) => Ok(!listeners.is_empty()),
            Err(_) => Ok(false), // Registry not available = no screen reader
        }
    }
}
```

## SUBTASK 4: Implement High Contrast Detection
Check GNOME/KDE settings for high contrast theme.

**Add method:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_high_contrast_active(&self) -> bool {
        // Try GNOME settings first
        if let Ok(is_high_contrast) = self.check_gnome_high_contrast() {
            return is_high_contrast;
        }

        // Try KDE settings
        if let Ok(is_high_contrast) = self.check_kde_high_contrast() {
            return is_high_contrast;
        }

        false
    }

    fn check_gnome_high_contrast(&self) -> Result<bool, Box<dyn std::error::Error>> {
        use zbus::proxy;

        #[proxy(
            interface = "org.freedesktop.portal.Settings",
            default_service = "org.freedesktop.portal.Desktop",
            default_path = "/org/freedesktop/portal/desktop"
        )]
        trait Settings {
            fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
        }

        let proxy = SettingsProxyBlocking::new(&self.connection)?;

        // Read org.gnome.desktop.a11y.interface high-contrast setting
        match proxy.read("org.gnome.desktop.a11y.interface", "high-contrast") {
            Ok(value) => {
                if let Some(v) = value.downcast_ref::<bool>() {
                    Ok(*v)
                } else {
                    Ok(false)
                }
            },
            Err(_) => Ok(false),
        }
    }

    fn check_kde_high_contrast(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // Check KDE config files
        let config_path = dirs::config_dir()
            .ok_or("Failed to get config directory")?
            .join("kdeglobals");

        if !config_path.exists() {
            return Ok(false);
        }

        let content = std::fs::read_to_string(config_path)?;

        // Look for high contrast color scheme
        Ok(content.contains("High Contrast") || content.contains("HighContrast"))
    }
}
```

## SUBTASK 5: Implement Reduced Motion Detection
Check desktop settings for reduced motion preference.

**Add method:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_reduced_motion_active(&self) -> bool {
        // Try GNOME animation settings
        if let Ok(animations_disabled) = self.check_gnome_animations() {
            return animations_disabled;
        }

        false
    }

    fn check_gnome_animations(&self) -> Result<bool, Box<dyn std::error::Error>> {
        use zbus::proxy;

        #[proxy(
            interface = "org.freedesktop.portal.Settings",
            default_service = "org.freedesktop.portal.Desktop",
            default_path = "/org/freedesktop/portal/desktop"
        )]
        trait Settings {
            fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
        }

        let proxy = SettingsProxyBlocking::new(&self.connection)?;

        // Read org.gnome.desktop.interface enable-animations setting
        match proxy.read("org.gnome.desktop.interface", "enable-animations") {
            Ok(value) => {
                if let Some(v) = value.downcast_ref::<bool>() {
                    Ok(!*v) // Inverted: animations disabled = reduced motion
                } else {
                    Ok(false)
                }
            },
            Err(_) => Ok(false),
        }
    }
}
```

## SUBTASK 6: Implement Large Text Detection
Check font scaling settings.

**Add method:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_large_text_enabled(&self) -> bool {
        if let Ok(scale) = self.check_text_scaling() {
            return scale > 1.0;
        }
        false
    }

    fn check_text_scaling(&self) -> Result<f64, Box<dyn std::error::Error>> {
        use zbus::proxy;

        #[proxy(
            interface = "org.freedesktop.portal.Settings",
            default_service = "org.freedesktop.portal.Desktop",
            default_path = "/org/freedesktop/portal/desktop"
        )]
        trait Settings {
            fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
        }

        let proxy = SettingsProxyBlocking::new(&self.connection)?;

        // Read text scaling factor
        match proxy.read("org.gnome.desktop.interface", "text-scaling-factor") {
            Ok(value) => {
                if let Some(v) = value.downcast_ref::<f64>() {
                    Ok(*v)
                } else {
                    Ok(1.0)
                }
            },
            Err(_) => Ok(1.0),
        }
    }
}
```

## SUBTASK 7: Error Handling and Fallbacks
Add comprehensive error handling for D-Bus connection failures.

**Changes needed:**
- Handle D-Bus connection failures gracefully
- Fall back to safe defaults when detection fails
- Log detection failures at appropriate level (debug, not error)
- Document limitations (requires D-Bus, GNOME/KDE specific)

## DEFINITION OF DONE
- [ ] AT-SPI and zbus dependencies added
- [ ] FallbackAccessibilityDetector struct implemented
- [ ] Screen reader detection functional (Orca, etc.)
- [ ] High contrast detection for GNOME/KDE
- [ ] Reduced motion detection implemented
- [ ] Large text/font scaling detection implemented
- [ ] D-Bus error handling comprehensive
- [ ] Code compiles on Linux without warnings
- [ ] Accessibility features work on Linux

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src
- Linux-specific code only (do not modify Windows or macOS sections)

## RESEARCH NOTES
- AT-SPI: Assistive Technology Service Provider Interface (Linux accessibility)
- D-Bus: Inter-process communication system used by AT-SPI
- Orca: GNOME screen reader
- xdg-desktop-portal: freedesktop.org settings portal
- GNOME settings: org.gnome.desktop.a11y.interface namespace
- KDE settings: kdeglobals config file

## DOCUMENTATION LOCATIONS
- AT-SPI docs: https://www.freedesktop.org/wiki/Accessibility/AT-SPI2/
- zbus docs: https://docs.rs/zbus/latest/zbus/
- freedesktop portal: https://flatpak.github.io/xdg-desktop-portal/
- Existing accessibility code: `packages/ecs-ui/src/accessibility/`
