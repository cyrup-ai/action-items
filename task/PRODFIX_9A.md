# PRODFIX_9A: Implement Linux Accessibility Detection

## OBJECTIVE
Replace the current shell-command based fallback accessibility detector with a functional AT-SPI and D-Bus based implementation to provide robust support for screen readers (e.g., Orca) and accessibility features on Linux desktops like GNOME and KDE.

## PRIORITY
**P2 - MEDIUM (Accessibility Failure)**

## LOCATION
`packages/ecs-ui/src/accessibility/detection.rs`

## CURRENT STATE
The Linux implementation in the `fallback` module (lines ~300-450) uses external commands like `pgrep`, `gsettings`, and `kreadconfig5`, which can fail in minimal environments, containers, or non-standard setups. Line 194 refers to the overall fallback struct, but the Linux-specific logic needs replacement with native D-Bus communication for reliability.

## RESEARCH NOTES
- **AT-SPI Detection**: The AT-SPI Registry D-Bus interface (`org.a11y.atspi.Registry` at `/org/a11y/atspi/registry`) can be queried to check for registered event listeners, indicating active assistive technology clients like screen readers. If the registry responds and has registered events, a screen reader is likely active. Source: [AT-SPI2 Core Spec](../tmp/at-spi2-core/atk/README.md)
- **zbus API**: Use `zbus::blocking::proxy` for synchronous D-Bus calls suitable for Bevy's main thread. Define custom proxy traits with `#[zbus::proxy]`. The `get_registered_events()` method returns event types; non-empty list indicates activity. Update to zbus 3.15 for latest blocking support. Source: [zbus Repository](../tmp/zbus/zbus/README.md), [zbus Docs](https://docs.rs/zbus/3.15.2/zbus/)
- **Settings Portal**: Use `org.freedesktop.portal.Settings` to read GNOME gsettings without spawning processes. Correct keys:
  - Screen reader: Not directly via portal; use AT-SPI for detection.
  - High contrast: Read `org.gnome.desktop.interface gtk-theme`; check if value starts with `"HighContrast"`. There is no boolean `high-contrast` key.
  - Reduced motion: Read `org.gnome.desktop.interface enable-animations`; `!value` indicates reduced motion.
  - Large text: Read `org.gnome.desktop.interface text-scaling-factor`; `> 1.0` indicates large text.
  Source: [xdg-desktop-portal](../tmp/xdg-desktop-portal/doc/portals/settings.md)
- **KDE Fallbacks**: For non-GNOME (e.g., KDE), parse config files using `dirs` crate. Add `dirs = "5"` dependency for cross-platform config paths.
- **Dependencies**: `atspi = "0.19"` for potential future AT-SPI interactions, though basic detection uses only `zbus`. No need for `atspi` if sticking to registry check, but include for completeness. Source: [rust-atspi](https://crates.io/crates/atspi)
- **Error Handling**: D-Bus calls can fail if no session bus or portal unavailable; fallback to `false` and log at debug level. Ensure non-blocking to avoid UI stalls.
- **Limitations**: Detection is best-effort; works on desktop Linux (GNOME/KDE with D-Bus). Server/headless environments will fallback gracefully. No Wayland/X11 distinction needed as D-Bus is protocol-agnostic.

## ULTRATHINK: Implementation Plan
### Step 1: Dependency Management
- Add Linux-specific dependencies to `Cargo.toml` to avoid bloating other platforms.
- Ensure `dirs` for KDE config paths.

### Step 2: Struct and Connection
- Define `FallbackAccessibilityDetector` under `#[cfg(target_os = "linux")]`.
- Initialize `zbus::Connection::session()?` for D-Bus access.
- Pattern: Wrap in `Result` for init failures.

### Step 3: Screen Reader Detection Pattern
- Define proxy trait for AT-SPI Registry.
- Attempt proxy creation and `get_registered_events()`.
- If successful and events non-empty, return `true`.
- Core Pattern:
  ```rust
  #[zbus::proxy(
      interface = "org.a11y.atspi.Registry",
      default_service = "org.a11y.atspi.Registry",
      default_path = "/org/a11y/atspi/registry"
  )]
  trait Registry {
      fn get_registered_events(&self) -> zbus::Result<Vec<String>>;
  }
  let proxy = RegistryProxy::new(&self.connection)?;
  let events = proxy.get_registered_events()?;
  !events.is_empty()
  ```
- Fallback: If proxy fails, assume inactive.

### Step 4: Settings Detection via Portal
- Define Settings proxy for `org.freedesktop.portal.Settings`.
- For each preference:
  - High contrast: `let theme = proxy.read("org.gnome.desktop.interface", "gtk-theme")?; if let Some(theme_str) = theme.downcast_ref::<&str>() { theme_str.starts_with("HighContrast") }`
  - Reduced motion: `let anims = proxy.read("org.gnome.desktop.interface", "enable-animations")?; if let Some(enabled) = anims.downcast_ref::<bool>() { !enabled }`
  - Large text: `let scale = proxy.read("org.gnome.desktop.interface", "text-scaling-factor")?; if let Some(s) = scale.downcast_ref::<f64>() { *s > 1.0 }`
- Pattern: Use `zbus::zvariant::OwnedValue` handling with downcast and defaults.
- KDE Fallback: Read `~/.config/kdeglobals`, parse INI-like for [Colors:Window] or ColorScheme containing "HighContrast".

### Step 5: Integration and Polling
- In `detect_accessibility_preferences` system, use the new detector.
- Poll every 3s as current; add change detection to minimize logs.
- Ensure thread-safety: zbus::Connection is Send+Sync.

### Step 6: Comprehensive Error Handling
- Wrap all D-Bus calls in `match` with `warn!` on error, default to `false`.
- No panics; use `?` in methods, propagate to system.

### Definition of Done Criteria
- Code compiles on Linux target without errors.
- D-Bus calls succeed on GNOME desktop; returns expected values (test manually with gsettings).
- Fallbacks work on KDE and non-D-Bus envs.
- No regressions on macOS/Windows.

## SUBTASK 1: Add Linux Accessibility Dependencies
Add to `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
zbus = "3.15"
atspi = "0.19"  # Optional, for future AT-SPI features
dirs = "5"      # For KDE config paths
```

## SUBTASK 2: Implement AT-SPI Connection and Struct
Replace the Linux parts in `fallback` mod around line 300+.

**Changes needed in** `packages/ecs-ui/src/accessibility/detection.rs`:
```rust
#[cfg(target_os = "linux")]
use zbus::blocking::proxy::{Proxy, ProxyDefault, ProxyError};
#[cfg(target_os = "linux")]
use zbus::zvariant::{OwnedValue, Type};
#[cfg(target_os = "linux")]
use dirs::config_dir;

#[cfg(target_os = "linux")]
pub struct FallbackAccessibilityDetector {
    connection: zbus::Connection,
}

#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let connection = zbus::Connection::session()?;
        Ok(Self { connection })
    }
}
```

## SUBTASK 3: Implement Screen Reader Detection
**Add to FallbackAccessibilityDetector:**
```rust
#[cfg(target_os = "linux")]
#[zbus::proxy(
    interface = "org.a11y.atspi.Registry",
    default_service = "org.a11y.atspi.Registry",
    default_path = "/org/a11y/atspi/registry"
)]
trait Registry {
    fn get_registered_events(&self) -> zbus::Result<Vec<String>>;
}

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
        match self.check_atspi_registry() {
            Ok(is_active) => is_active,
            Err(e) => {
                tracing::debug!("AT-SPI check failed: {}", e);
                false
            }
        }
    }

    fn check_atspi_registry(&self) -> Result<bool, zbus::Error> {
        let proxy = RegistryProxy::new(&self.connection)?;
        let events = proxy.get_registered_events()?;
        Ok(!events.is_empty())
    }
}
```
Note: Use `zbus::blocking::proxy::Proxy` for the proxy creation; the derive handles the trait.

## SUBTASK 4: Implement High Contrast Detection
Corrected to check GTK theme name.

**Add method:**
```rust
#[cfg(target_os = "linux")]
#[zbus::proxy(
    interface = "org.freedesktop.portal.Settings",
    default_service = "org.freedesktop.portal.Desktop",
    default_path = "/org/freedesktop/portal/desktop"
)]
trait Settings {
    fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
}

#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_high_contrast_active(&self) -> bool {
        if let Ok(is_high_contrast) = self.check_gnome_high_contrast() {
            return is_high_contrast;
        }
        self.check_kde_high_contrast().unwrap_or(false)
    }

    fn check_gnome_high_contrast(&self) -> Result<bool, zbus::Error> {
        let proxy = SettingsProxy::new(&self.connection)?;
        let value = proxy.read("org.gnome.desktop.interface", "gtk-theme")?;
        if let Ok(theme) = value.downcast::<String>() {
            Ok(theme.starts_with("HighContrast"))
        } else {
            Ok(false)
        }
    }

    fn check_kde_high_contrast(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let config_path = config_dir().ok_or("No config dir")?
            .join("kdeglobals");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            Ok(content.contains("[Colors:Window]") && (content.contains("High Contrast") || content.contains("HighContrast")))
        } else {
            Ok(false)
        }
    }
}
```

## SUBTASK 5: Implement Reduced Motion Detection
**Add method:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_reduced_motion_active(&self) -> bool {
        if let Ok(reduced) = self.check_gnome_reduced_motion() {
            return reduced;
        }
        self.check_kde_reduced_motion().unwrap_or(false)
    }

    fn check_gnome_reduced_motion(&self) -> Result<bool, zbus::Error> {
        let proxy = SettingsProxy::new(&self.connection)?;
        let value = proxy.read("org.gnome.desktop.interface", "enable-animations")?;
        if let Ok(enabled) = value.downcast::<bool>() {
            Ok(!enabled)
        } else {
            Ok(false)
        }
    }

    fn check_kde_reduced_motion(&self) -> Result<bool, Box<dyn std::error::Error>> {
        // KDE animations config
        let config_path = config_dir().ok_or("No config dir")?
            .join("kdeglobals");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            // Check if animations disabled
            Ok(content.contains("Animations=0") || content.contains("EnableAnimation=false"))
        } else {
            Ok(false)
        }
    }
}
```

## SUBTASK 6: Implement Large Text Detection
**Add method:**
```rust
#[cfg(target_os = "linux")]
impl FallbackAccessibilityDetector {
    fn is_large_text_enabled(&self) -> bool {
        if let Ok(scale) = self.check_text_scaling() {
            return scale > 1.0;
        }
        self.check_kde_large_text().unwrap_or(false)
    }

    fn check_text_scaling(&self) -> Result<f64, zbus::Error> {
        let proxy = SettingsProxy::new(&self.connection)?;
        let value = proxy.read("org.gnome.desktop.interface", "text-scaling-factor")?;
        if let Ok(scale) = value.downcast::<f64>() {
            Ok(*scale)
        } else {
            Ok(1.0)
        }
    }

    fn check_kde_large_text(&self) -> Result<bool, Box<dyn std::error::Error>> {
        let config_path = config_dir().ok_or("No config dir")?
            .join("kdeglobals");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            // Check font size in [General] or Fonts section
            if let Some(font_line) = content.lines().find(|l| l.contains("font=")) {
                if font_line.contains(",14") || font_line.contains(",16") || font_line.contains(",18") {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }
}
```

## SUBTASK 7: Update AccessibilityState and Integration
- Ensure `AccessibilityState` and `detect_preferences` use the new methods.
- In the `fallback` mod, replace all Linux `is_*` methods with calls to the new detector.
- Update `get_accessibility_state` to use `self.detect_preferences()`.
- Add necessary imports: `use zbus::blocking::proxy::{Proxy, ProxyDefault}; use zbus::zvariant::Value;`

## SUBTASK 8: Error Handling and Fallbacks
- All D-Bus methods return `Result`, catch errors with `match` or `?`, log with `tracing::debug!`.
- If portal/AT-SPI unavailable, fallback to current shell-command logic as secondary check.
- In `detect_accessibility_preferences` system, wrap detector calls in `catch_unwind` as existing.

## DEFINITION OF DONE
- [ ] Linux dependencies added and Cargo.toml updates
- [ ] FallbackAccessibilityDetector uses D-Bus connection
- [ ] AT-SPI registry check for screen reader
- [ ] Portal-based GNOME settings detection for high contrast (theme check), reduced motion, large text
- [ ] KDE file-based fallbacks for all features
- [ ] Error handling prevents crashes; logs at debug level
- [ ] Code compiles and runs on Linux without warnings
- [ ] Accessibility state updates correctly on GNOME desktop (manual verification)
- [ ] No changes to macOS/Windows code

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- **DO NOT add extensive documentation** - inline comments only
- Focus solely on implementation in `./src/accessibility/detection.rs` and Cargo.toml
- Linux-specific code only (#[cfg(target_os = "linux")]); preserve other platforms
- Keep polling interval at 3s to avoid performance impact