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
                text_scale_factor: 1.0,
                magnifier_active: false,
                narrator_active: false,
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
        pub text_scale_factor: f32,
        pub magnifier_active: bool,
        pub narrator_active: bool,
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
    use windows::Win32::UI::Accessibility::{IUIAutomation, IUIAutomationElement};
#[cfg(target_os = "windows")]
    use windows::Win32::System::Com::{CoCreateInstance, CLSID_UIAutomation, CLSCTX_INPROC_SERVER};
#[cfg(target_os = "windows")]
    use windows::core::Result;

#[cfg(target_os = "windows")]
use windows::Win32::UI::WindowsAndMessaging::{SystemParametersInfoW, FindWindowW, SPI_GETHIGHCONTRAST, ANIMATIONINFO, SPI_GETANIMATION, SPI_GETLOGICALDPIOVERRIDE, SPI_GETCLIENTAREAANIMATION, SPI_GETMENUANIMATION, SPI_GETCOMBOBOXANIMATION, SPI_GETLISTBOXSMOOTHSCROLLING};
#[cfg(target_os = "windows")]
use windows::Win32::Graphics::Gdi::{GetDC, ReleaseDC, GetDeviceCaps, LOGPIXELSX};

#[cfg(target_os = "windows")]
use windows::Win32::UI::Accessibility::{HIGHCONTRASTW, HCF_HIGHCONTRASTON};

#[cfg(target_os = "windows")]
use windows::Win32::System::ProcessStatus::{EnumProcesses, OpenProcess, GetModuleBaseNameW, CloseHandle};
#[cfg(target_os = "windows")]
use windows::Win32::System::NtDll::{RtlGetVersion, OSVERSIONINFOW};

#[cfg(target_os = "windows")]
use windows::Win32::Foundation::{PROCESS_QUERY_INFORMATION, GetLastError};

#[cfg(target_os = "windows")]
pub fn get_os_version_major() -> u32 {
    use windows::Win32::System::NtDll::{RtlGetVersion, OSVERSIONINFOW};
    unsafe {
        let mut version_info: OSVERSIONINFOW = std::mem::zeroed();
        version_info.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOW>() as u32;
        let status = RtlGetVersion(&mut version_info);
        if status == 0 {
            version_info.dwMajorVersion
        } else {
            tracing::warn!("RtlGetVersion failed with status: {}", status);
            10u32 // default to Windows 10+ for safety
        }
    }
}

#[cfg(target_os = "windows")]
fn initialize_uia() -> windows::core::Result<windows::Win32::UI::Accessibility::IUIAutomation> {
    if get_os_version_major() < 10 {
        tracing::warn!("Windows <10 detected; skipping UIA");
        return Err(windows::core::Error::new(windows::core::HRESULT(0x80004001)));
    }

    use windows::Win32::System::Com::{CoCreateInstance, CLSID_UIAutomation};
    use windows::Win32::UI::Accessibility::{IUIAutomation, IUIAutomationElement};

    let client: IUIAutomation = unsafe { CoCreateInstance(&CLSID_UIAutomation, None, windows::core::CLSCTX_INPROC_SERVER)? };
    let _root = client.GetRootElement()?;
    Ok(client)
}

    #[cfg(target_os = "linux")]
    use zbus::blocking::proxy::Proxy;
    #[cfg(target_os = "linux")]
    use zbus::{Connection, zvariant::OwnedValue};
    #[cfg(target_os = "linux")]
    use dirs;
    #[cfg(target_os = "linux")]
    use std::fs;
    #[cfg(target_os = "linux")]
    use tracing;

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
    #[zbus::proxy(
        interface = "org.freedesktop.portal.Settings",
        default_service = "org.freedesktop.portal.Desktop",
        default_path = "/org/freedesktop/portal/desktop"
    )]
    trait Settings {
        fn read(&self, namespace: &str, key: &str) -> zbus::Result<zbus::zvariant::OwnedValue>;
    }



    /// Fallback accessibility detector for non-macOS platforms
    #[derive(Debug)]
pub struct FallbackAccessibilityDetector {
        #[cfg(target_os = "linux")]
        connection: Connection,
        #[cfg(target_os = "windows")]
        uia_client: Option<IUIAutomation>,
    }

    impl FallbackAccessibilityDetector {
        pub fn new() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
            #[cfg(target_os = "linux")]
            {
                let connection = Connection::session()?;
                Ok(Self { connection })
            }

            #[cfg(target_os = "windows")]
            {
            let major = get_os_version_major();
            tracing::debug!("Windows major version: {}", major);

            let uia_client = match initialize_uia() {
                Ok(client) => {
                    tracing::info!("UIA initialized successfully");
                    Some(client)
                }
                Err(e) => {
                    tracing::warn!("UIA initialization failed: {:?} (code: 0x{:x})", e, e.code().0);
                    None
                }
            };
            Ok(Self { uia_client })
        }
        }

        #[cfg(target_os = "windows")]
        fn enumerate_running_processes() -> std::collections::HashSet<String> {
            use std::collections::HashSet;
            use std::ptr;

            let mut pids: [u32; 1024] = [0; 1024];
            let mut cb_needed = 0u32;
            let mut names = HashSet::new();

            unsafe {
                if EnumProcesses(pids.as_mut_ptr(), (std::mem::size_of::<u32>() * 1024) as u32, &mut cb_needed) == 0 {
                    tracing::debug!("Native process enum failed, falling back to tasklist");
                    // Fallback to tasklist for all names if needed, but for simplicity, return empty and use specific fallback
                    return names;
                }
                let num_procs = cb_needed as usize / std::mem::size_of::<u32>();
                for &pid in &pids[0..num_procs] {
                    if pid == 0 { continue; }
                    let h_process = OpenProcess(PROCESS_QUERY_INFORMATION.0 as u32, false, pid);
                    if h_process.is_invalid() {
                        tracing::debug!("OpenProcess failed for PID {}: {}", pid, GetLastError());
                        continue;
                    }
                    let mut name_buf: [u16; 260] = [0; 260];
                    let name_len = GetModuleBaseNameW(h_process, ptr::null_mut(), name_buf.as_mut_ptr(), 260);
                    unsafe { CloseHandle(h_process); }
                    if name_len > 0 {
                        let name = String::from_utf16_lossy(&name_buf[0..name_len as usize]).to_lowercase();
                        names.insert(name);
                    } else {
                        tracing::debug!("GetModuleBaseNameW failed for PID {}: {}", pid, GetLastError());
                    }
                }
            }
            names
        }

        #[cfg(target_os = "windows")]
        fn fallback_tasklist(&self) -> bool {
            // Check for common Windows screen readers
            let screen_readers = ["nvda", "jaws", "narrator", "sapisvr"];

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
        pub fn check_screen_reader_processes(&self) -> bool {
            let processes = Self::enumerate_running_processes();
            if !processes.is_empty() {
                let screen_readers = ["nvda.exe", "jaws.exe", "narrator.exe", "sapisvr.exe"];
                for reader in screen_readers {
                    if processes.contains(&reader.to_lowercase()) {
                        return true;
                    }
                }
            }
            self.fallback_tasklist()
        }

        #[cfg(target_os = "windows")]
        pub fn is_screen_reader_active(&self) -> bool {
            self.check_screen_reader_processes() || self.fallback_tasklist()
        }

        #[cfg(target_os = "windows")]
        pub fn is_magnifier_active(&self) -> bool {
            unsafe { !FindWindowW(windows::core::w!("Magnifier"), None).is_null() }
        }

        #[cfg(target_os = "windows")]
        pub fn is_narrator_active(&self) -> bool {
            self.check_for_process("narrator.exe")
        }

        #[cfg(target_os = "windows")]
        fn check_for_process(&self, target_name: &str) -> bool {
            let processes = Self::enumerate_running_processes();
            let target_lower = target_name.to_lowercase();
            if processes.contains(&target_lower) {
                true
            } else {
                // Fallback for specific process if enum failed
                if let Ok(output) = std::process::Command::new("tasklist")
                    .args(&["/FI", &format!("IMAGENAME eq {}", target_name)])
                    .output()
                {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains(target_name)
                } else {
                    false
                }
            }
        }

        #[cfg(target_os = "windows")]
        pub fn get_text_scale_factor(&self) -> f32 {
            unsafe {
                let hdc = GetDC(None); // Screen DC
                if hdc.is_invalid() {
                    tracing::debug!("GetDC failed for DPI detection");
                    return 1.0;
                }
                let dpi_x = GetDeviceCaps(hdc, LOGPIXELSX);
                let _ = ReleaseDC(None, hdc);
                let scale = (dpi_x as f32 / 96.0).max(1.0);
                tracing::debug!("Detected DPI scale: {}", scale);
                scale
            }
        }

        #[cfg(target_os = "windows")]
        pub fn is_high_contrast_enabled(&self) -> bool {
            let win32_hc = unsafe {
                let mut hc_info: HIGHCONTRASTW = std::mem::zeroed();
                hc_info.cbSize = std::mem::size_of::<HIGHCONTRASTW>() as u32;

                let result = SystemParametersInfoW(
                    SPI_GETHIGHCONTRAST,
                    hc_info.cbSize,
                    &mut hc_info as *mut _ as *mut std::ffi::c_void,
                    0,
                );

                if result == 0 {
                    tracing::debug!("SPI_GETHIGHCONTRAST failed: {}", GetLastError());
                    false
                } else if (hc_info.dwFlags & HCF_HIGHCONTRASTON) != 0 {
                    true
                } else {
                    false
                }
            };

            let uia_hc = if let Some(client) = &self.uia_client {
                if let Ok(root) = unsafe { client.GetRootElement() } {
                    if let Ok(visual_effects) = unsafe { root.CurrentVisualEffects() } {
                        visual_effects == 1 // 1 indicates high contrast
                    } else {
                        false
                    }
                } else {
                    false
                }
            } else {
                false
            };

            win32_hc || uia_hc
        }

        #[cfg(target_os = "linux")]
        pub fn is_screen_reader_active(&self) -> bool {
            if let Ok(is_active) = self.check_atspi_registry() {
                return is_active;
            }
            // Fallback to shell command
            if let Ok(output) = std::process::Command::new("pgrep").arg("orca").output() {
                if output.status.success() && !output.stdout.is_empty() {
                    return true;
                }
            }
            // Could add more like espeak, but for now orca as primary
            false
        }

        #[cfg(target_os = "linux")]
        fn check_atspi_registry(&self) -> Result<bool, zbus::Error> {
            let proxy = RegistryProxy::new(&self.connection)?;
            let events = proxy.get_registered_events()?;
            Ok(!events.is_empty())
        }

        #[cfg(target_os = "linux")]
        pub fn is_high_contrast_enabled(&self) -> bool {
            self.check_gnome_high_contrast().unwrap_or(false) || self.check_kde_high_contrast().unwrap_or(false)
        }

        #[cfg(target_os = "linux")]
        fn check_gnome_high_contrast(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            match SettingsProxy::new(&self.connection) {
                Ok(proxy) => {
                    match proxy.read("org.gnome.desktop.interface", "gtk-theme") {
                        Ok(value) => {
                            if let Ok(theme) = value.try_into::<String>() {
                                return Ok(theme.starts_with("HighContrast"));
                            }
                            // Fallback to gsettings if value parse fails
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                if let Ok(theme) = output_str.trim().trim_matches('"').parse::<String>() {
                                    return Ok(theme.starts_with("HighContrast"));
                                }
                            }
                            Ok(false)
                        }
                        Err(_) => {
                            // Fallback to shell gsettings
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                if let Ok(theme) = output_str.trim().trim_matches('"').parse::<String>() {
                                    Ok(theme.starts_with("HighContrast"))
                                } else {
                                    Ok(false)
                                }
                            } else {
                                Ok(false)
                            }
                        }
                    }
                }
                Err(_) => {
                    // Fallback to shell gsettings
                    if let Ok(output) = std::process::Command::new("gsettings")
                        .args(&["get", "org.gnome.desktop.interface", "gtk-theme"])
                        .output()
                    {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if let Ok(theme) = output_str.trim().trim_matches('"').parse::<String>() {
                            Ok(theme.starts_with("HighContrast"))
                        } else {
                            Ok(false)
                        }
                    } else {
                        Ok(false)
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        fn check_kde_high_contrast(&self) -> Result<bool, Box<dyn std::error::Error>> {
            let config_path = dirs::config_dir().ok_or("No config dir")?
                .join("kdeglobals");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                Ok(content.contains("[Colors:Window]") && (content.contains("High Contrast") || content.contains("HighContrast")))
            } else {
                Ok(false)
            }
        }

        #[cfg(all(unix, not(any(target_os = "macos", target_os = "linux"))))]
        pub fn is_high_contrast_enabled(&self) -> bool {
            false
        }

        pub fn get_accessibility_state(&self) -> AccessibilityState {
            AccessibilityState {
                screen_reader_active: self.is_screen_reader_active(),
                high_contrast: self.is_high_contrast_enabled(),
                reduced_motion: self.is_reduced_motion_enabled(),
                large_text: self.is_large_text_enabled(),
                text_scale_factor: self.get_text_scale_factor(),
                magnifier_active: self.is_magnifier_active(),
                narrator_active: self.is_narrator_active(),
            }
        }

        fn is_reduced_motion_enabled(&self) -> bool {
            if cfg!(target_os = "windows") {
                let spis = [
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETCLIENTAREAANIMATION,
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETMENUANIMATION,
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETCOMBOBOXANIMATION,
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETLISTBOXSMOOTHSCROLLING,
                    windows::Win32::UI::WindowsAndMessaging::SPI_GETANIMATION,
                ];
                let mut reduced = false;
                for &spi in &spis {
                    let mut enabled = 0u32;
                    let result = unsafe {
                        SystemParametersInfoW(spi, 0, &mut enabled as *mut _ as *mut std::ffi::c_void, 0) != 0
                    };
                    if !result {
                        tracing::debug!("SPI {:?} failed: {}", spi, GetLastError());
                        continue;
                    }
                    if enabled == 0 {
                        reduced = true;
                        break;
                    }
                }
                reduced
            } else if cfg!(target_os = "linux") {
                self.check_gnome_reduced_motion().unwrap_or(false) || self.check_kde_reduced_motion().unwrap_or(false)
            } else {
                false
            }
        }

        #[cfg(target_os = "linux")]
        fn check_gnome_reduced_motion(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
            match SettingsProxy::new(&self.connection) {
                Ok(proxy) => {
                    match proxy.read("org.gnome.desktop.interface", "enable-animations") {
                        Ok(value) => {
                            if let Ok(enabled) = value.try_into::<bool>() {
                                return Ok(!enabled);
                            }
                            // Fallback
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "enable-animations"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                if output_str.trim() == "false" {
                                    return Ok(true);
                                }
                            }
                            Ok(false)
                        }
                        Err(_) => {
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "enable-animations"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                Ok(output_str.trim() == "false")
                            } else {
                                Ok(false)
                            }
                        }
                    }
                }
                Err(_) => {
                    if let Ok(output) = std::process::Command::new("gsettings")
                        .args(&["get", "org.gnome.desktop.interface", "enable-animations"])
                        .output()
                    {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        Ok(output_str.trim() == "false")
                    } else {
                        Ok(false)
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        fn check_kde_reduced_motion(&self) -> Result<bool, Box<dyn std::error::Error>> {
            let config_path = dirs::config_dir().ok_or("No config dir")?
                .join("kdeglobals");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                Ok(content.contains("Animations=0") || content.contains("EnableAnimation=false"))
            } else {
                Ok(false)
            }
        }

        fn is_large_text_enabled(&self) -> bool {
            if cfg!(target_os = "windows") {
                let scale = self.get_text_scale_factor();
                scale > 1.25
            } else if cfg!(target_os = "linux") {
                let scale = self.check_text_scaling().unwrap_or(1.0);
                (scale > 1.0) || self.check_kde_large_text().unwrap_or(false)
            } else {
                false
            }
        }

        #[cfg(target_os = "linux")]
        fn check_text_scaling(&self) -> Result<f64, Box<dyn std::error::Error + Send + Sync>> {
            match SettingsProxy::new(&self.connection) {
                Ok(proxy) => {
                    match proxy.read("org.gnome.desktop.interface", "text-scaling-factor") {
                        Ok(value) => {
                            if let Ok(scale) = value.try_into::<f64>() {
                                return Ok(scale);
                            }
                            // Fallback
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "text-scaling-factor"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                if let Ok(scale_factor) = output_str.trim().parse::<f64>() {
                                    return Ok(scale_factor);
                                }
                            }
                            Ok(1.0)
                        }
                        Err(_) => {
                            if let Ok(output) = std::process::Command::new("gsettings")
                                .args(&["get", "org.gnome.desktop.interface", "text-scaling-factor"])
                                .output()
                            {
                                let output_str = String::from_utf8_lossy(&output.stdout);
                                if let Ok(scale_factor) = output_str.trim().parse::<f64>() {
                                    Ok(scale_factor)
                                } else {
                                    Ok(1.0)
                                }
                            } else {
                                Ok(1.0)
                            }
                        }
                    }
                }
                Err(_) => {
                    if let Ok(output) = std::process::Command::new("gsettings")
                        .args(&["get", "org.gnome.desktop.interface", "text-scaling-factor"])
                        .output()
                    {
                        let output_str = String::from_utf8_lossy(&output.stdout);
                        if let Ok(scale_factor) = output_str.trim().parse::<f64>() {
                            Ok(scale_factor)
                        } else {
                            Ok(1.0)
                        }
                    } else {
                        Ok(1.0)
                    }
                }
            }
        }

        #[cfg(target_os = "linux")]
        fn check_kde_large_text(&self) -> Result<bool, Box<dyn std::error::Error>> {
            let config_path = dirs::config_dir().ok_or("No config dir")?
                .join("kdeglobals");
            if config_path.exists() {
                let content = std::fs::read_to_string(&config_path)?;
                // Check font size in [General] or Fonts section
                if let Some(font_line) = content.lines().find(|l| l.contains("font=")) {
                    if font_line.contains(",14") || font_line.contains(",16") || font_line.contains(",18") {
                        return Ok(true);
                    }
                }
            }
            Ok(false)
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
        pub text_scale_factor: f32,
        pub magnifier_active: bool,
        pub narrator_active: bool,
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
                text_scale_factor: 1.0,
                magnifier_active: false,
                narrator_active: false,
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
                text_scale_factor: state.text_scale_factor,
                magnifier_active: state.magnifier_active,
                narrator_active: state.narrator_active,
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
    pub text_scale_factor: f32,
    pub magnifier_active: bool,
    pub narrator_active: bool,
}

/// System to detect and respond to accessibility preferences with real platform APIs
/// Uses timer-based polling to avoid excessive OS API calls (polls every 3 seconds instead of every frame)
pub fn detect_accessibility_preferences(
    mut accessibility_manager: ResMut<AccessibilityManager>,
    detector: Option<Res<AccessibilityDetector>>,
    mut poll_timer: Local<Option<Timer>>,
    time: Res<Time>,
) {
    // Initialize timer for periodic polling (every 3 seconds)
    if poll_timer.is_none() {
        *poll_timer = Some(Timer::from_seconds(3.0, TimerMode::Repeating));
    }
    
    // Only poll when timer fires, not every frame
    if let Some(ref mut timer) = poll_timer.as_mut() {
        timer.tick(time.delta());
        if !timer.just_finished() {
            return; // Skip this frame
        }
    }
    
    if let Some(detector) = detector {
        #[allow(clippy::disallowed_methods)]
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            detector.get_accessibility_state()
        })) {
            Ok(state) => {
                let old_state = AccessibilityState {
                    screen_reader_active: accessibility_manager.screen_reader_active,
                    high_contrast: accessibility_manager.high_contrast,
                    reduced_motion: accessibility_manager.reduced_motion,
                    large_text: accessibility_manager.large_text,
                    text_scale_factor: accessibility_manager.text_scale_factor,
                    magnifier_active: accessibility_manager.magnifier_active,
                    narrator_active: accessibility_manager.narrator_active,
                };

                // Update accessibility manager with real system state
                accessibility_manager.screen_reader_active = state.screen_reader_active;
                accessibility_manager.high_contrast = state.high_contrast;
                accessibility_manager.reduced_motion = state.reduced_motion;
                accessibility_manager.large_text = state.large_text;
                accessibility_manager.text_scale_factor = state.text_scale_factor;
                accessibility_manager.magnifier_active = state.magnifier_active;
                accessibility_manager.narrator_active = state.narrator_active;

                let state_changed = old_state.screen_reader_active != state.screen_reader_active
                    || old_state.high_contrast != state.high_contrast
                    || old_state.reduced_motion != state.reduced_motion
                    || old_state.large_text != state.large_text
                    || (old_state.text_scale_factor - state.text_scale_factor).abs() > 0.01
                    || old_state.magnifier_active != state.magnifier_active
                    || old_state.narrator_active != state.narrator_active;

                if state_changed {
                    tracing::info!(
                        "Accessibility state changed: screen_reader={}, high_contrast={}, \
                         reduced_motion={}, large_text={}, scale={}, magnifier={}, narrator={}",
                        state.screen_reader_active,
                        state.high_contrast,
                        state.reduced_motion,
                        state.large_text,
                        state.text_scale_factor,
                        state.magnifier_active,
                        state.narrator_active
                    );
                }

                // Always update metrics (lightweight operation)
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
                if state_changed && state.screen_reader_active && accessibility_manager.announcements.is_empty() {
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
        // Only log warning when timer fires, not every frame
        tracing::warn!(
            "AccessibilityDetector resource not found - using default accessibility settings"
        );
        // Use conservative defaults when detector is not available
        accessibility_manager.screen_reader_active = false;
        accessibility_manager.high_contrast = false;
        accessibility_manager.reduced_motion = false;
    }
}
