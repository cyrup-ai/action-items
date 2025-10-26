# PRODFIX_9B: Implement Windows Accessibility Detection

## OBJECTIVE
Replace placeholder accessibility detector with functional UI Automation based implementation to support screen readers and accessibility features on Windows.

## PRIORITY
**P2 - MEDIUM (Accessibility Failure)**

## LOCATION
`packages/ecs-ui/src/accessibility/detection.rs`

## CURRENT STATE
Line 194 has a placeholder `FallbackAccessibilityDetector` struct with no implementation. Windows users with disabilities get no accessibility support.

## SUBTASK 1: Add Windows Accessibility Dependencies
Add Windows UI Automation API crates.

**Changes needed in** `packages/ecs-ui/Cargo.toml`:
```toml
[target.'cfg(windows)'.dependencies]
windows = { version = "0.52", features = [
    "Win32_UI_Accessibility",
    "Win32_Foundation",
    "Win32_System_SystemInformation",
    "Win32_Graphics_Gdi",
] }
```

## SUBTASK 2: Implement Windows Accessibility Detector
Create the FallbackAccessibilityDetector struct with UI Automation.

**Changes needed in** `packages/ecs-ui/src/accessibility/detection.rs` **around line 194:**
```rust
#[cfg(target_os = "windows")]
pub struct FallbackAccessibilityDetector {
    // Windows doesn't need persistent state, all checks are via Win32 APIs
}

#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        Ok(Self {})
    }

    pub fn detect_preferences(&self) -> AccessibilityPreferences {
        AccessibilityPreferences {
            screen_reader_active: self.is_screen_reader_active(),
            high_contrast: self.is_high_contrast_active(),
            reduced_motion: self.is_reduced_motion_active(),
            large_text: self.is_large_text_enabled(),
        }
    }
}
```

## SUBTASK 3: Implement Screen Reader Detection
Use SystemParametersInfo to detect screen reader presence.

**Add method:**
```rust
#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    fn is_screen_reader_active(&self) -> bool {
        use windows::Win32::UI::Accessibility::*;
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::*;

        unsafe {
            // Check if screen reader is running
            let mut screen_reader_active: BOOL = FALSE;
            let result = SystemParametersInfoW(
                SPI_GETSCREENREADER,
                0,
                Some(&mut screen_reader_active as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );

            if result.is_ok() && screen_reader_active.as_bool() {
                return true;
            }

            // Alternative check: Look for common screen reader processes
            self.check_screen_reader_processes()
        }
    }

    fn check_screen_reader_processes(&self) -> bool {
        use std::process::Command;

        // Check for NVDA, JAWS, Narrator processes
        let screen_readers = vec!["nvda.exe", "jfw.exe", "narrator.exe"];

        if let Ok(output) = Command::new("tasklist").output() {
            if let Ok(tasklist) = String::from_utf8(output.stdout) {
                let tasklist_lower = tasklist.to_lowercase();
                return screen_readers.iter()
                    .any(|sr| tasklist_lower.contains(&sr.to_lowercase()));
            }
        }

        false
    }
}
```

## SUBTASK 4: Implement High Contrast Detection
Check Windows high contrast mode setting.

**Add method:**
```rust
#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    fn is_high_contrast_active(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::UI::Accessibility::*;
        use windows::Win32::Foundation::*;

        unsafe {
            let mut high_contrast = HIGHCONTRASTW {
                cbSize: std::mem::size_of::<HIGHCONTRASTW>() as u32,
                dwFlags: HCF_AVAILABLE,
                lpszDefaultScheme: PWSTR::null(),
            };

            let result = SystemParametersInfoW(
                SPI_GETHIGHCONTRAST,
                high_contrast.cbSize,
                Some(&mut high_contrast as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );

            if result.is_ok() {
                return high_contrast.dwFlags.contains(HCF_HIGHCONTRASTON);
            }

            false
        }
    }
}
```

## SUBTASK 5: Implement Reduced Motion Detection
Check Windows animation settings.

**Add method:**
```rust
#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    fn is_reduced_motion_active(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::*;

        unsafe {
            // Check if animations are disabled
            let mut client_area_animation: BOOL = FALSE;
            let result = SystemParametersInfoW(
                SPI_GETCLIENTAREAANIMATION,
                0,
                Some(&mut client_area_animation as *mut _ as *mut _),
                SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
            );

            if result.is_ok() {
                // Inverted: animations disabled = reduced motion
                return !client_area_animation.as_bool();
            }

            false
        }
    }
}
```

## SUBTASK 6: Implement Large Text Detection
Check Windows text scaling and DPI settings.

**Add method:**
```rust
#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    fn is_large_text_enabled(&self) -> bool {
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::Foundation::*;

        unsafe {
            // Get system DPI
            let hdc = GetDC(None);
            if hdc.is_invalid() {
                return false;
            }

            let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
            let _ = ReleaseDC(None, hdc);

            // Standard DPI is 96, anything above 120 indicates scaling
            dpi_x > 120
        }
    }

    fn get_text_scale_factor(&self) -> f64 {
        use windows::Win32::Graphics::Gdi::*;
        use windows::Win32::Foundation::*;

        unsafe {
            let hdc = GetDC(None);
            if hdc.is_invalid() {
                return 1.0;
            }

            let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
            let _ = ReleaseDC(None, hdc);

            // Calculate scale factor relative to 96 DPI (100%)
            dpi_x as f64 / 96.0
        }
    }
}
```

## SUBTASK 7: Add Accessibility Feature Flags
Check additional Windows accessibility features.

**Add helper methods:**
```rust
#[cfg(target_os = "windows")]
impl FallbackAccessibilityDetector {
    fn is_magnifier_active(&self) -> bool {
        use windows::Win32::UI::WindowsAndMessaging::*;
        use windows::Win32::Foundation::*;

        unsafe {
            // Find Magnifier window
            let window_name: Vec<u16> = "Magnifier\0".encode_utf16().collect();
            let hwnd = FindWindowW(
                None,
                PCWSTR(window_name.as_ptr()),
            );

            !hwnd.is_invalid() && IsWindowVisible(hwnd).as_bool()
        }
    }

    fn is_narrator_active(&self) -> bool {
        use std::process::Command;

        if let Ok(output) = Command::new("tasklist").output() {
            if let Ok(tasklist) = String::from_utf8(output.stdout) {
                return tasklist.to_lowercase().contains("narrator.exe");
            }
        }

        false
    }
}
```

## SUBTASK 8: Error Handling and Logging
Add comprehensive error handling for Win32 API calls.

**Changes needed:**
- Handle SystemParametersInfoW failures gracefully
- Log detection failures at debug level
- Return safe defaults when detection fails
- Document Windows version requirements
- Handle permission issues

## DEFINITION OF DONE
- [ ] Windows accessibility dependencies added
- [ ] FallbackAccessibilityDetector implemented
- [ ] Screen reader detection functional (NVDA, JAWS, Narrator)
- [ ] High contrast mode detection working
- [ ] Reduced motion (animation) detection implemented
- [ ] Large text/DPI scaling detection functional
- [ ] Optional magnifier/narrator checks added
- [ ] Win32 API error handling comprehensive
- [ ] Code compiles on Windows without warnings
- [ ] Accessibility features work on Windows

## CONSTRAINTS
- **DO NOT write unit tests** - another team handles testing
- **DO NOT write benchmarks** - another team handles performance
- Focus solely on implementation in ./src
- Windows-specific code only (do not modify Linux or macOS sections)

## RESEARCH NOTES
- SystemParametersInfo: Primary API for accessibility settings
- SPI_GETSCREENREADER: Detect screen reader presence
- SPI_GETHIGHCONTRAST: Get high contrast mode state
- SPI_GETCLIENTAREAANIMATION: Check animation settings
- DPI awareness: Important for text scaling detection
- NVDA: Popular open-source screen reader
- JAWS: Commercial screen reader
- Narrator: Built-in Windows screen reader

## DOCUMENTATION LOCATIONS
- SystemParametersInfo: https://learn.microsoft.com/en-us/windows/win32/api/winuser/nf-winuser-systemparametersinfow
- Windows Accessibility: https://learn.microsoft.com/en-us/windows/win32/winauto/entry-uiauto-win32
- UI Automation: https://learn.microsoft.com/en-us/windows/win32/winauto/uiauto-uiautomationoverview
- Existing accessibility code: `packages/ecs-ui/src/accessibility/`
