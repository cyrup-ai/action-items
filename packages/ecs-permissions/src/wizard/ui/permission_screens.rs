//! Permission Screen Infrastructure
//!
//! Provides foundational infrastructure for permission-specific UI screens with platform detection
//! and content management systems. This builds on existing permission card components to provide
//! platform-specific instruction screens for all 40 permission types across macOS, Windows, and Linux.

use crate::types::PermissionType;

/// Platform enumeration for runtime platform detection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
}

/// Detect the current platform at runtime
pub fn detect_current_platform() -> Platform {
    #[cfg(target_os = "macos")]
    return Platform::MacOS;
    #[cfg(target_os = "windows")]
    return Platform::Windows;
    #[cfg(target_os = "linux")]
    return Platform::Linux;
}

/// Trait for consistent permission screen interface across all 40 permission types
pub trait PermissionScreen {
    /// Get the permission type this screen represents
    fn permission_type(&self) -> PermissionType;
    
    /// Get platform-specific instructions for granting this permission
    fn platform_instructions(&self, platform: Platform) -> &'static str;
    
    /// Get the Unicode icon character for this permission
    fn icon_unicode(&self) -> char;
    
    /// Get the button text for requesting this permission
    fn button_text(&self) -> &'static str;
    
    /// Check if privilege escalation is required for this permission on the given platform
    fn is_privilege_escalation_required(&self, platform: Platform) -> bool;
}

/// Screen content components for permission screen elements
#[derive(Debug, Clone)]
pub struct PermissionScreenContent {
    pub permission_type: PermissionType,
    pub title: String,
    pub description: String,
    pub platform_instructions: String,
    pub icon: String,
    pub button_text: String,
    pub requires_elevation: bool,
}

impl PermissionScreenContent {
    /// Create a new builder for permission screen content
    pub fn builder(permission_type: PermissionType) -> PermissionScreenContentBuilder {
        PermissionScreenContentBuilder::new(permission_type)
    }
}

/// Builder pattern for creating permission screen content
#[derive(Debug)]
pub struct PermissionScreenContentBuilder {
    permission_type: PermissionType,
    title: Option<String>,
    description: Option<String>,
    platform_instructions: Option<String>,
    icon: Option<String>,
    button_text: Option<String>,
    requires_elevation: Option<bool>,
}

impl PermissionScreenContentBuilder {
    /// Create a new builder for the given permission type
    pub fn new(permission_type: PermissionType) -> Self {
        Self {
            permission_type,
            title: None,
            description: None,
            platform_instructions: None,
            icon: None,
            button_text: None,
            requires_elevation: None,
        }
    }
    
    /// Set the title for the permission screen
    pub fn title<S: Into<String>>(mut self, title: S) -> Self {
        self.title = Some(title.into());
        self
    }
    
    /// Set the description for the permission screen
    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
    
    /// Set the platform-specific instructions
    pub fn platform_instructions<S: Into<String>>(mut self, instructions: S) -> Self {
        self.platform_instructions = Some(instructions.into());
        self
    }
    
    /// Set the Unicode icon for the permission
    pub fn icon<S: Into<String>>(mut self, icon: S) -> Self {
        self.icon = Some(icon.into());
        self
    }
    
    /// Set the button text for requesting the permission
    pub fn button_text<S: Into<String>>(mut self, text: S) -> Self {
        self.button_text = Some(text.into());
        self
    }
    
    /// Set whether privilege escalation is required
    pub fn requires_elevation(mut self, requires: bool) -> Self {
        self.requires_elevation = Some(requires);
        self
    }
    
    /// Build the permission screen content
    pub fn build(self) -> PermissionScreenContent {
        PermissionScreenContent {
            permission_type: self.permission_type,
            title: self.title.unwrap_or_else(|| format!("{}", self.permission_type)),
            description: self.description.unwrap_or_else(|| "System permission".to_string()),
            platform_instructions: self.platform_instructions.unwrap_or_else(|| "Please grant this permission in system settings".to_string()),
            icon: self.icon.unwrap_or_else(|| "🔒".to_string()),
            button_text: self.button_text.unwrap_or_else(|| format!("Grant {} Access", self.permission_type)),
            requires_elevation: self.requires_elevation.unwrap_or(false),
        }
    }
}


// =============================================================================
// DATA ACCESS PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create a calendar permission screen with platform-specific content
pub fn create_calendar_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Calendars, then check the box next to this application",
        Platform::Windows => "Open Settings > Privacy > Calendar, then toggle 'Allow apps to access your calendar' and enable this application",
        Platform::Linux => "Calendar access is managed by Evolution Data Server. The system will handle permissions through D-Bus services",
    };

    PermissionScreenContent::builder(PermissionType::Calendar)
        .title("Calendar Access Required")
        .description("This application needs access to your calendar for scheduling and event management features")
        .platform_instructions(instructions)
        .icon('📅')
        .button_text("Grant Calendar Access")
        .requires_elevation(false)
        .build()
}

/// Create a reminders permission screen with platform-specific content
pub fn create_reminders_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Reminders, then check the box next to this application",
        Platform::Windows => "Reminders access is managed by Windows Notification API. The system will prompt you when needed",
        Platform::Linux => "Reminders are managed by task management services. Access is handled through D-Bus productivity services",
    };

    PermissionScreenContent::builder(PermissionType::Reminders)
        .title("Reminders Access Required")
        .description("This application needs access to your reminders for task management and notification features")
        .platform_instructions(instructions)
        .icon('⏰')
        .button_text("Grant Reminders Access")
        .requires_elevation(false)
        .build()
}

/// Create a contacts permission screen with platform-specific content
pub fn create_contacts_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Contacts, then check the box next to this application",
        Platform::Windows => "Open Settings > Privacy > Contacts, then toggle 'Allow apps to access your contacts' and enable this application",
        Platform::Linux => "Contacts are managed by Evolution Data Server. Access is handled through D-Bus productivity services",
    };

    PermissionScreenContent::builder(PermissionType::Contacts)
        .title("Contacts Access Required")
        .description("This application needs access to your contacts for address book and contact management features")
        .platform_instructions(instructions)
        .icon('👥')
        .button_text("Grant Contacts Access")
        .requires_elevation(false)
        .build()
}

/// Create an address book permission screen with platform-specific content
pub fn create_address_book_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Contacts, then check the box next to this application",
        Platform::Windows => "Address book access is managed by Windows Contacts API. The system will handle permissions automatically",
        Platform::Linux => "Address book is managed by system contact services. Access is handled through D-Bus productivity services",
    };

    PermissionScreenContent::builder(PermissionType::AddressBook)
        .title("Address Book Access Required")
        .description("This application needs access to your address book for legacy contact management features")
        .platform_instructions(instructions)
        .icon('📇')
        .button_text("Grant Address Book Access")
        .requires_elevation(false)
        .build()
}

/// Create a photos permission screen with platform-specific content
pub fn create_photos_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Photos, then check the box next to this application",
        Platform::Windows => "Open Settings > Privacy > Pictures, then toggle 'Allow apps to access your pictures library' and enable this application",
        Platform::Linux => "Photo library access is managed by file permissions. Ensure the Pictures directory is accessible",
    };

    PermissionScreenContent::builder(PermissionType::Photos)
        .title("Photos Access Required")
        .description("This application needs access to your photo library for image management and viewing features")
        .platform_instructions(instructions)
        .icon('📸')
        .button_text("Grant Photos Access")
        .requires_elevation(false)
        .build()
}

/// Create a photos add permission screen with platform-specific content
pub fn create_photos_add_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Open System Preferences > Security & Privacy > Privacy > Photos, then check the box next to this application for add permissions",
        Platform::Windows => "Photo library modification is managed by Windows Photos API. Ensure the app can write to Pictures library",
        Platform::Linux => "Photo library modification is managed by file permissions. Ensure write access to the Pictures directory",
    };

    PermissionScreenContent::builder(PermissionType::PhotosAdd)
        .title("Photos Add Access Required")
        .description("This application needs permission to add photos to your photo library")
        .platform_instructions(instructions)
        .icon('📷')
        .button_text("Grant Photo Add Access")
        .requires_elevation(false)
        .build()
}


// =============================================================================
// CORE SYSTEM PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create a camera permission screen with platform-specific content
pub fn create_camera_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Camera",
        Platform::Windows => "Settings > Privacy > Camera",
        Platform::Linux => "Camera access managed by application permissions",
    };

    PermissionScreenContent::builder(PermissionType::Camera)
        .title("Camera Access Required")
        .description("This application needs access to your camera for video capture and image processing features")
        .platform_instructions(instructions)
        .icon('📷')
        .button_text("Grant Camera Access")
        .requires_elevation(false)
        .build()
}

/// Create a microphone permission screen with platform-specific content
pub fn create_microphone_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Microphone",
        Platform::Windows => "Settings > Privacy > Microphone",
        Platform::Linux => "Microphone access managed by PulseAudio permissions",
    };

    PermissionScreenContent::builder(PermissionType::Microphone)
        .title("Microphone Access Required")
        .description("This application needs access to your microphone for audio recording and voice input features")
        .platform_instructions(instructions)
        .icon('🎤')
        .button_text("Grant Microphone Access")
        .requires_elevation(false)
        .build()
}

/// Create a location permission screen with platform-specific content
pub fn create_location_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Location Services",
        Platform::Windows => "Settings > Privacy > Location",
        Platform::Linux => "Location services managed by GeoClue service",
    };

    PermissionScreenContent::builder(PermissionType::Location)
        .title("Location Access Required")
        .description("This application needs access to your location for location-based features and services")
        .platform_instructions(instructions)
        .icon('📍')
        .button_text("Grant Location Access")
        .requires_elevation(false)
        .build()
}

/// Create a bluetooth permission screen with platform-specific content
pub fn create_bluetooth_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Bluetooth",
        Platform::Windows => "Settings > Privacy > Other devices",
        Platform::Linux => "Bluetooth access managed by BlueZ service",
    };

    PermissionScreenContent::builder(PermissionType::Bluetooth)
        .title("Bluetooth Access Required")
        .description("This application needs access to Bluetooth for device connectivity and wireless communication")
        .platform_instructions(instructions)
        .icon('📶')
        .button_text("Grant Bluetooth Access")
        .requires_elevation(false)
        .build()
}

/// Create a wifi permission screen with platform-specific content
pub fn create_wifi_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Network > WiFi",
        Platform::Windows => "Settings > Network & Internet > WiFi",
        Platform::Linux => "WiFi managed by NetworkManager service",
    };

    PermissionScreenContent::builder(PermissionType::WiFi)
        .title("WiFi Access Required")
        .description("This application needs access to WiFi information for network connectivity features")
        .platform_instructions(instructions)
        .icon('📶')
        .button_text("Grant WiFi Access")
        .requires_elevation(false)
        .build()
}

/// Create a screen capture permission screen with platform-specific content
pub fn create_screen_capture_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Screen Recording",
        Platform::Windows => "Screen capture permissions managed by Windows Display API",
        Platform::Linux => "Screen capture managed by X11/Wayland compositor",
    };

    PermissionScreenContent::builder(PermissionType::ScreenCapture)
        .title("Screen Capture Access Required")
        .description("This application needs permission to capture screen content for recording and sharing features")
        .platform_instructions(instructions)
        .icon('📺')
        .button_text("Grant Screen Capture Access")
        .requires_elevation(false)
        .build()
}

/// Create an accessibility permission screen with platform-specific content
pub fn create_accessibility_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Accessibility",
        Platform::Windows => "Accessibility features managed by Windows Accessibility API",
        Platform::Linux => "Accessibility managed by AT-SPI service",
    };

    PermissionScreenContent::builder(PermissionType::Accessibility)
        .title("Accessibility Access Required")
        .description("This application needs accessibility permissions for assistive features and system interaction")
        .platform_instructions(instructions)
        .icon('♿')
        .button_text("Grant Accessibility Access")
        .requires_elevation(true)
        .build()
}



// =============================================================================
// FILE SYSTEM & STORAGE PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create a full disk access permission screen with platform-specific content
pub fn create_full_disk_access_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Full Disk Access",
        Platform::Windows => "Administrator privileges required for full disk access",
        Platform::Linux => "Root or sudo access required for full disk access",
    };

    PermissionScreenContent::builder(PermissionType::FullDiskAccess)
        .title("Full Disk Access Required")
        .description("This application needs full disk access for comprehensive file system operations")
        .platform_instructions(instructions)
        .icon('💾')
        .button_text("Grant Full Disk Access")
        .requires_elevation(true)
        .build()
}

/// Create a desktop folder permission screen with platform-specific content
pub fn create_desktop_folder_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Files and Folders > Desktop Folder",
        Platform::Windows => "Desktop folder access managed by Windows File API",
        Platform::Linux => "Desktop folder access managed by file permissions",
    };

    PermissionScreenContent::builder(PermissionType::DesktopFolder)
        .title("Desktop Folder Access Required")
        .description("This application needs access to your desktop folder for file management features")
        .platform_instructions(instructions)
        .icon("🖥️")
        .button_text("Grant Desktop Access")
        .requires_elevation(false)
        .build()
}

/// Create a documents folder permission screen with platform-specific content
pub fn create_documents_folder_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Files and Folders > Documents Folder",
        Platform::Windows => "Documents folder access managed by Windows File API",
        Platform::Linux => "Documents folder access managed by file permissions",
    };

    PermissionScreenContent::builder(PermissionType::DocumentsFolder)
        .title("Documents Folder Access Required")
        .description("This application needs access to your documents folder for file management features")
        .platform_instructions(instructions)
        .icon('📄')
        .button_text("Grant Documents Access")
        .requires_elevation(false)
        .build()
}

/// Create a downloads folder permission screen with platform-specific content
pub fn create_downloads_folder_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Files and Folders > Downloads Folder",
        Platform::Windows => "Downloads folder access managed by Windows File API",
        Platform::Linux => "Downloads folder access managed by file permissions",
    };

    PermissionScreenContent::builder(PermissionType::DownloadsFolder)
        .title("Downloads Folder Access Required")
        .description("This application needs access to your downloads folder for file management features")
        .platform_instructions(instructions)
        .icon("⬇️")
        .button_text("Grant Downloads Access")
        .requires_elevation(false)
        .build()
}

/// Create a network volumes permission screen with platform-specific content
pub fn create_network_volumes_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Files and Folders > Network Volumes",
        Platform::Windows => "Network drive access managed by Windows Network API",
        Platform::Linux => "Network volumes managed by mount permissions",
    };

    PermissionScreenContent::builder(PermissionType::NetworkVolumes)
        .title("Network Volumes Access Required")
        .description("This application needs access to network volumes for network file operations")
        .platform_instructions(instructions)
        .icon('🌐')
        .button_text("Grant Network Access")
        .requires_elevation(false)
        .build()
}

/// Create a removable volumes permission screen with platform-specific content
pub fn create_removable_volumes_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Files and Folders > Removable Volumes",
        Platform::Windows => "Removable drive access managed by Windows Storage API",
        Platform::Linux => "Removable volumes managed by udisks service",
    };

    PermissionScreenContent::builder(PermissionType::RemovableVolumes)
        .title("Removable Volumes Access Required")
        .description("This application needs access to removable volumes for external storage operations")
        .platform_instructions(instructions)
        .icon('💿')
        .button_text("Grant Removable Access")
        .requires_elevation(false)
        .build()
}

/// Create an admin files permission screen with platform-specific content
pub fn create_admin_files_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Administrator privileges required for system files",
        Platform::Windows => "Administrator privileges required for system files",
        Platform::Linux => "Root access required for system files",
    };

    PermissionScreenContent::builder(PermissionType::AdminFiles)
        .title("Admin Files Access Required")
        .description("This application needs administrator privileges to access system files")
        .platform_instructions(instructions)
        .icon('🔐')
        .button_text("Grant Admin Access")
        .requires_elevation(true)
        .build()
}
// =============================================================================
// INPUT & ACCESSIBILITY PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create an input monitoring permission screen with platform-specific content
pub fn create_input_monitoring_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Input Monitoring",
        Platform::Windows => "Input monitoring managed by Windows Input API",
        Platform::Linux => "Input monitoring managed by X11/Wayland input system",
    };

    PermissionScreenContent::builder(PermissionType::InputMonitoring)
        .title("Input Monitoring Access Required")
        .description("This application needs access to monitor input events for automation and accessibility features")
        .platform_instructions(instructions)
        .icon("⌨️")
        .button_text("Grant Input Monitoring")
        .requires_elevation(true)
        .build()
}

/// Create an accessibility mouse permission screen with platform-specific content
pub fn create_accessibility_mouse_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Accessibility (Mouse Control)",
        Platform::Windows => "Mouse accessibility features managed by Windows Accessibility API",
        Platform::Linux => "Mouse accessibility managed by AT-SPI mouse control service",
    };

    PermissionScreenContent::builder(PermissionType::AccessibilityMouse)
        .title("Mouse Control Access Required")
        .description("This application needs access to control mouse events for automation and accessibility features")
        .platform_instructions(instructions)
        .icon("🖱️")
        .button_text("Grant Mouse Control")
        .requires_elevation(true)
        .build()
}

/// Create a post event permission screen with platform-specific content
pub fn create_post_event_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Accessibility (Event Posting)",
        Platform::Windows => "Event posting managed by Windows Event API",
        Platform::Linux => "Event posting managed by X11/Wayland event system",
    };

    PermissionScreenContent::builder(PermissionType::PostEvent)
        .title("Event Posting Access Required")
        .description("This application needs permission to post system events for automation features")
        .platform_instructions(instructions)
        .icon('📤')
        .button_text("Grant Event Posting")
        .requires_elevation(true)
        .build()
}

/// Create a speech recognition permission screen with platform-specific content
pub fn create_speech_recognition_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Speech Recognition",
        Platform::Windows => "Settings > Privacy > Speech",
        Platform::Linux => "Speech recognition managed by Speech Dispatcher service",
    };

    PermissionScreenContent::builder(PermissionType::SpeechRecognition)
        .title("Speech Recognition Access Required")
        .description("This application needs access to speech recognition for voice control features")
        .platform_instructions(instructions)
        .icon("🗣️")
        .button_text("Grant Speech Recognition")
        .requires_elevation(false)
        .build()
}

/// Create a willful write permission screen with platform-specific content
pub fn create_willful_write_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "File system write permissions managed by sandboxing",
        Platform::Windows => "File write permissions managed by Windows File API",
        Platform::Linux => "File write permissions managed by filesystem ACLs",
    };

    PermissionScreenContent::builder(PermissionType::WillfulWrite)
        .title("File Write Access Required")
        .description("This application needs permission to write files for data storage and configuration")
        .platform_instructions(instructions)
        .icon("✏️")
        .button_text("Grant Write Access")
        .requires_elevation(false)
        .build()
}

/// Create an all permissions screen with platform-specific content
pub fn create_all_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Grant all permissions through System Preferences > Security & Privacy",
        Platform::Windows => "Grant all permissions through Settings > Privacy",
        Platform::Linux => "Grant all permissions through system configuration",
    };

    PermissionScreenContent::builder(PermissionType::All)
        .title("All Permissions Required")
        .description("This application needs access to all system permissions for full functionality")
        .platform_instructions(instructions)
        .icon('🔓')
        .button_text("Grant All Permissions")
        .requires_elevation(true)
        .build()
}

// =============================================================================
// SYSTEM INTEGRATION PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create a calls permission screen with platform-specific content
pub fn create_calls_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Phone calls managed by CallKit framework",
        Platform::Windows => "Phone calls managed by Windows Telephony API",
        Platform::Linux => "Phone calls managed by telephony services",
    };

    PermissionScreenContent::builder(PermissionType::Calls)
        .title("Calls Access Required")
        .description("This application needs access to make and manage phone calls")
        .platform_instructions(instructions)
        .icon('📞')
        .button_text("Grant Calls Access")
        .requires_elevation(false)
        .build()
}

/// Create a face id permission screen with platform-specific content
pub fn create_face_id_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Face ID managed by Local Authentication framework",
        Platform::Windows => "Windows Hello Face managed by Windows Biometric API",
        Platform::Linux => "Facial recognition not available",
    };

    PermissionScreenContent::builder(PermissionType::FaceID)
        .title("Face ID Access Required")
        .description("This application needs access to Face ID for biometric authentication")
        .platform_instructions(instructions)
        .icon('👤')
        .button_text("Grant Face ID Access")
        .requires_elevation(true)
        .build()
}

/// Create a file provider domain permission screen with platform-specific content
pub fn create_file_provider_domain_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "File provider domains managed by File Provider framework",
        Platform::Windows => "File provider access managed by Windows Storage API",
        Platform::Linux => "File provider access managed by filesystem services",
    };

    PermissionScreenContent::builder(PermissionType::FileProviderDomain)
        .title("File Provider Domain Access Required")
        .description("This application needs access to file provider domains for cloud storage integration")
        .platform_instructions(instructions)
        .icon('📁')
        .button_text("Grant File Provider Access")
        .requires_elevation(false)
        .build()
}

/// Create a file provider presence permission screen with platform-specific content
pub fn create_file_provider_presence_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "File provider presence managed by File Provider framework",
        Platform::Windows => "File provider presence managed by Windows Storage API",
        Platform::Linux => "File provider presence managed by filesystem monitoring",
    };

    PermissionScreenContent::builder(PermissionType::FileProviderPresence)
        .title("File Provider Presence Access Required")
        .description("This application needs access to detect file provider presence for storage management")
        .platform_instructions(instructions)
        .icon("👁️")
        .button_text("Grant Provider Presence Access")
        .requires_elevation(false)
        .build()
}

/// Create a remote desktop permission screen with platform-specific content
pub fn create_remote_desktop_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Sharing > Remote Management",
        Platform::Windows => "Settings > System > Remote Desktop",
        Platform::Linux => "Remote desktop managed by VNC/RDP services",
    };

    PermissionScreenContent::builder(PermissionType::RemoteDesktop)
        .title("Remote Desktop Access Required")
        .description("This application needs access to remote desktop functionality for screen sharing")
        .platform_instructions(instructions)
        .icon("🖥️")
        .button_text("Grant Remote Desktop")
        .requires_elevation(true)
        .build()
}

/// Create a ubiquitous file provider permission screen with platform-specific content
pub fn create_ubiquitous_file_provider_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "iCloud Drive managed by File Provider framework",
        Platform::Windows => "Cloud storage managed by Windows Cloud Files API",
        Platform::Linux => "Cloud storage managed by cloud sync services",
    };

    PermissionScreenContent::builder(PermissionType::UbiquitousFileProvider)
        .title("Cloud File Provider Access Required")
        .description("This application needs access to cloud file providers for synchronized storage")
        .platform_instructions(instructions)
        .icon("☁️")
        .button_text("Grant Cloud Access")
        .requires_elevation(false)
        .build()
}
// =============================================================================
// ADVANCED FEATURES PERMISSION SCREEN IMPLEMENTATIONS
// =============================================================================

/// Create an apple events permission screen with platform-specific content
pub fn create_apple_events_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Automation",
        Platform::Windows => "Inter-app communication managed by Windows COM API",
        Platform::Linux => "Inter-app communication managed by D-Bus service",
    };

    PermissionScreenContent::builder(PermissionType::AppleEvents)
        .title("AppleEvents Access Required")
        .description("This application needs access to automation features for inter-app communication")
        .platform_instructions(instructions)
        .icon('🔗')
        .button_text("Grant Automation Access")
        .requires_elevation(false)
        .build()
}

/// Create a siri permission screen with platform-specific content
pub fn create_siri_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Siri & Spotlight > Siri Suggestions & Privacy",
        Platform::Windows => "Cortana integration managed by Windows Voice API",
        Platform::Linux => "Voice assistant integration not available",
    };

    PermissionScreenContent::builder(PermissionType::Siri)
        .title("Siri Access Required")
        .description("This application needs access to Siri for voice control features")
        .platform_instructions(instructions)
        .icon("🗣️")
        .button_text("Grant Siri Access")
        .requires_elevation(false)
        .build()
}

/// Create a focus status permission screen with platform-specific content
pub fn create_focus_status_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Focus",
        Platform::Windows => "Settings > System > Focus assist",
        Platform::Linux => "Focus status managed by notification services",
    };

    PermissionScreenContent::builder(PermissionType::FocusStatus)
        .title("FocusStatus Access Required")
        .description("This application needs access to focus status for notification management")
        .platform_instructions(instructions)
        .icon('🎯')
        .button_text("Grant Focus Access")
        .requires_elevation(false)
        .build()
}

/// Create a media library permission screen with platform-specific content
pub fn create_media_library_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "System Preferences > Security & Privacy > Privacy > Media & Apple Music",
        Platform::Windows => "Settings > Privacy > Music Library",
        Platform::Linux => "Media library access managed by media services",
    };

    PermissionScreenContent::builder(PermissionType::MediaLibrary)
        .title("MediaLibrary Access Required")
        .description("This application needs access to media library for music and media features")
        .platform_instructions(instructions)
        .icon('🎵')
        .button_text("Grant Media Library Access")
        .requires_elevation(false)
        .build()
}

/// Create a motion permission screen with platform-specific content
pub fn create_motion_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Motion sensors managed by Core Motion framework",
        Platform::Windows => "Motion sensors managed by Windows Sensor API",
        Platform::Linux => "Motion sensors managed by IIO subsystem",
    };

    PermissionScreenContent::builder(PermissionType::Motion)
        .title("Motion Access Required")
        .description("This application needs access to motion sensors for movement detection")
        .platform_instructions(instructions)
        .icon('📱')
        .button_text("Grant Motion Access")
        .requires_elevation(false)
        .build()
}

/// Create a nearby interaction permission screen with platform-specific content
pub fn create_nearby_interaction_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Nearby interaction managed by Core Bluetooth framework",
        Platform::Windows => "Settings > Privacy > Other devices",
        Platform::Linux => "Nearby interaction managed by Bluetooth services",
    };

    PermissionScreenContent::builder(PermissionType::NearbyInteraction)
        .title("NearbyInteraction Access Required")
        .description("This application needs access to nearby interaction for device communication")
        .platform_instructions(instructions)
        .icon('📡')
        .button_text("Grant Nearby Access")
        .requires_elevation(false)
        .build()
}

/// Create a developer tools permission screen with platform-specific content
pub fn create_developer_tools_permission_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Developer tools managed by Xcode and system frameworks",
        Platform::Windows => "Developer tools managed by Windows SDK",
        Platform::Linux => "Developer tools managed by system packages",
    };

    PermissionScreenContent::builder(PermissionType::DeveloperTools)
        .title("DeveloperTools Access Required")
        .description("This application needs access to developer tools for development features")
        .platform_instructions(instructions)
        .icon("🛠️")
        .button_text("Grant Developer Access")
        .requires_elevation(true)
        .build()
}

/// Create all permissions batch request screen with platform-specific content
pub fn create_all_permissions_screen() -> PermissionScreenContent {
    let platform = detect_current_platform();
    let instructions = match platform {
        Platform::MacOS => "Grant all required permissions in System Preferences",
        Platform::Windows => "Grant all required permissions in Settings",
        Platform::Linux => "Grant all required permissions through system services",
    };

    PermissionScreenContent::builder(PermissionType::All)
        .title("All Permissions Required")
        .description("This application needs access to all system permissions for full functionality")
        .platform_instructions(instructions)
        .icon('🔓')
        .button_text("Grant All Permissions")
        .requires_elevation(true)
        .build()
}