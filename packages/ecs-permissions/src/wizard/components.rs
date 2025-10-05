//! Wizard UI Components
//!
//! Defines all Bevy components used for wizard UI elements. Designed for
//! optimal performance with pre-allocated entities and efficient change detection.

#![allow(dead_code)] // Wizard component library - API surface area for UI systems

use bevy::prelude::*;
use crate::types::{PermissionType, PermissionStatus};
use crate::wizard::{WizardState, PermissionStatusExt};

/// Root component marking the main wizard UI container
#[derive(Component, Debug)]
pub struct WizardRoot {
    /// Timestamp when wizard was started
    pub started_at: std::time::SystemTime,
    /// Whether the wizard UI is currently visible
    pub is_visible: bool,
    /// Current wizard configuration
    pub config: WizardConfig,
}

/// Wizard configuration options
#[derive(Debug, Clone)]
pub struct WizardConfig {
    /// Whether to show advanced options
    pub show_advanced: bool,
    /// Whether to auto-advance through steps
    pub auto_advance: bool,
    /// Minimum time to spend in each step (for UX)
    pub min_step_duration: std::time::Duration,
    /// Maximum time to wait for permission responses
    pub permission_timeout: std::time::Duration,
}

impl Default for WizardConfig {
    fn default() -> Self {
        Self {
            show_advanced: false,
            auto_advance: true,
            min_step_duration: std::time::Duration::from_millis(800),
            permission_timeout: std::time::Duration::from_secs(30),
        }
    }
}

impl WizardRoot {
    /// Create a new wizard root component
    pub fn new() -> Self {
        Self {
            started_at: std::time::SystemTime::now(),
            is_visible: false,
            config: WizardConfig::default(),
        }
    }
    
    /// Get the total time the wizard has been active
    #[inline]
    pub fn active_duration(&self) -> std::time::Duration {
        self.started_at.elapsed().unwrap_or(std::time::Duration::ZERO)
    }
}

impl Default for WizardRoot {
    fn default() -> Self {
        Self::new()
    }
}

/// Component for wizard panel containers (Welcome, Permissions, etc.)
#[derive(Component, Debug)]
pub struct WizardPanel {
    /// Which wizard state this panel represents
    pub state: WizardState,
    /// Whether this panel is currently active/visible
    pub is_active: bool,
    /// Animation state for smooth transitions
    pub animation_progress: f32,
}

impl WizardPanel {
    /// Create a new wizard panel for the specified state
    #[inline]
    pub fn new(state: WizardState) -> Self {
        Self {
            state,
            is_active: false,
            animation_progress: 0.0,
        }
    }
    
    /// Check if this panel should be visible based on animation progress
    #[inline]
    pub fn is_visible(&self) -> bool {
        self.is_active && self.animation_progress > 0.0
    }
    
    /// Update animation progress (0.0 = hidden, 1.0 = fully visible)
    #[inline]
    pub fn set_animation_progress(&mut self, progress: f32) {
        self.animation_progress = progress.clamp(0.0, 1.0);
    }
}

/// Component for individual permission cards in the UI
#[derive(Component, Debug)]
pub struct PermissionCard {
    /// The permission type this card represents
    pub permission_type: PermissionType,
    /// Current status of this permission
    pub status: PermissionStatus,
    /// Whether this card is currently interactive
    pub is_interactive: bool,
    /// Whether this permission is required (vs optional)
    pub is_required: bool,
    /// Last time this permission was checked
    pub last_checked: std::time::Instant,
    /// Animation state for status changes
    pub status_animation: f32,
    /// Number of retry attempts made for this permission
    pub retry_count: u8,
    /// Maximum allowed retry attempts (default: 3)
    pub max_retries: u8,
}

impl PermissionCard {
    /// Create a new permission card
    pub fn new(permission_type: PermissionType, is_required: bool) -> Self {
        Self {
            permission_type,
            status: PermissionStatus::NotDetermined,
            is_interactive: true,
            is_required,
            last_checked: std::time::Instant::now(),
            status_animation: 0.0,
            retry_count: 0,
            max_retries: 3,
        }
    }
    
    /// Update the permission status with animation
    pub fn set_status(&mut self, new_status: PermissionStatus) {
        if new_status != self.status {
            self.status = new_status;
            self.status_animation = 0.0; // Reset animation
            self.last_checked = std::time::Instant::now();
        }
    }
    
    /// Check if this permission needs attention (required but not granted)
    #[inline]
    pub fn needs_attention(&self) -> bool {
        self.is_required && !self.status.is_granted()
    }
    
    /// Get the display priority for this card (higher = more important)
    #[inline]
    pub fn display_priority(&self) -> u8 {
        match (self.is_required, self.status) {
            (true, PermissionStatus::Denied) => 10,
            (true, PermissionStatus::Restricted) => 9,
            (true, PermissionStatus::NotDetermined) => 8,
            (true, PermissionStatus::Unknown) => 8,
            (true, PermissionStatus::Authorized) => 3,
            (false, PermissionStatus::Denied) => 5,
            (false, PermissionStatus::Restricted) => 4,
            (false, PermissionStatus::NotDetermined) => 2,
            (false, _) => 1,
        }
    }
    
    /// Get user-friendly description of this permission
    #[inline]
    pub fn description(&self) -> &'static str {
        match self.permission_type {
            PermissionType::Accessibility => "Accessibility access for global hotkeys",
            PermissionType::ScreenCapture => "Screen recording for screenshots",
            PermissionType::InputMonitoring => "Input monitoring for advanced hotkeys",
            PermissionType::Camera => "Camera access for profile pictures",
            PermissionType::Microphone => "Microphone access for voice notes",
            PermissionType::FullDiskAccess => "Full disk access for file operations",
            PermissionType::WiFi => "WiFi access for network features",
            PermissionType::Location => "Location access for location-based features",
            PermissionType::Calendar => "Calendar access for scheduling features",
            PermissionType::Reminders => "Reminders access for task management",
            PermissionType::Contacts => "Contacts access for contact management",
            PermissionType::Bluetooth => "Bluetooth access for device connectivity",
            PermissionType::Photos => "Photos access for image management",
            PermissionType::SpeechRecognition => "Speech recognition for voice commands",
            PermissionType::DesktopFolder => "Desktop folder access for file management",
            PermissionType::DocumentsFolder => "Documents folder access for file operations",
            PermissionType::DownloadsFolder => "Downloads folder access for file management",
            PermissionType::AppleEvents => "Apple Events for application automation",
            PermissionType::DeveloperTools => "Developer tools access for debugging",
            PermissionType::AdminFiles => "Administrative files access for system operations",
            PermissionType::AddressBook => "Address book access for contact management",
            PermissionType::All => "All permissions for complete application functionality",
            PermissionType::Calls => "Phone calls access for communication features",
            PermissionType::FaceID => "Face ID access for biometric authentication",
            PermissionType::FileProviderDomain => "File provider domain access for cloud integration",
            PermissionType::FileProviderPresence => "File provider presence for file system integration",
            PermissionType::FocusStatus => "Focus status access for notification management",
            PermissionType::MediaLibrary => "Media library access for content management",
            PermissionType::Motion => "Motion data access for activity tracking",
            PermissionType::NearbyInteraction => "Nearby interaction for device discovery",
            PermissionType::PhotosAdd => "Photos addition permission for content creation",
            PermissionType::PostEvent => "Post event permission for system integration",
            PermissionType::RemoteDesktop => "Remote desktop access for screen sharing",
            PermissionType::Siri => "Siri integration for voice assistant features",
            PermissionType::NetworkVolumes => "Network volumes access for remote file systems",
            PermissionType::RemovableVolumes => "Removable volumes access for external storage",
            PermissionType::UbiquitousFileProvider => "iCloud file provider for cloud storage",
            PermissionType::WillfulWrite => "Willful write permission for intentional file modifications",
            PermissionType::AccessibilityMouse => "Accessibility mouse control for assistive features",
        }
    }
    
    /// Get the reason why this permission is needed
    #[inline]
    pub fn reason(&self) -> &'static str {
        match self.permission_type {
            PermissionType::Accessibility => "Required for global hotkeys to work system-wide",
            PermissionType::ScreenCapture => "Enables taking screenshots and screen recordings",
            PermissionType::InputMonitoring => "Allows monitoring keyboard shortcuts",
            PermissionType::Camera => "Optional for adding profile pictures",
            PermissionType::Microphone => "Optional for voice note features",
            PermissionType::FullDiskAccess => "Enables working with files anywhere on disk",
            PermissionType::WiFi => "Optional for network-related features",
            PermissionType::Location => "Required for location-aware functionality",
            PermissionType::Calendar => "Required for calendar integration features",
            PermissionType::Reminders => "Required for reminder and task management",
            PermissionType::Contacts => "Required for contact management features",
            PermissionType::Bluetooth => "Required for Bluetooth device connectivity",
            PermissionType::Photos => "Required for photo and image management",
            PermissionType::SpeechRecognition => "Required for voice command functionality",
            PermissionType::DesktopFolder => "Required for desktop file management",
            PermissionType::DocumentsFolder => "Required for document file operations",
            PermissionType::DownloadsFolder => "Required for downloads folder access",
            PermissionType::AppleEvents => "Required for application automation",
            PermissionType::DeveloperTools => "Required for development and debugging features",
            PermissionType::AdminFiles => "Required for administrative file operations",
            PermissionType::AddressBook => "Required for address book integration",
            PermissionType::All => "Required for complete application functionality",
            PermissionType::Calls => "Required for phone call integration",
            PermissionType::FaceID => "Required for biometric authentication",
            PermissionType::FileProviderDomain => "Required for cloud file integration",
            PermissionType::FileProviderPresence => "Required for file system integration",
            PermissionType::FocusStatus => "Required for notification management",
            PermissionType::MediaLibrary => "Required for media content management",
            PermissionType::Motion => "Required for motion and activity tracking",
            PermissionType::NearbyInteraction => "Required for device discovery features",
            PermissionType::PhotosAdd => "Required for photo creation and editing",
            PermissionType::PostEvent => "Required for system event integration",
            PermissionType::RemoteDesktop => "Required for remote desktop functionality",
            PermissionType::Siri => "Required for voice assistant integration",
            PermissionType::NetworkVolumes => "Required for network file system access",
            PermissionType::RemovableVolumes => "Required for external storage access",
            PermissionType::UbiquitousFileProvider => "Required for iCloud storage integration",
            PermissionType::WillfulWrite => "Required for intentional file modifications",
            PermissionType::AccessibilityMouse => "Required for assistive mouse control",
        }
    }
    
    /// Check if retry is available for this permission
    ///
    /// Returns true if retry count is below max and status is Denied or Restricted
    #[inline]
    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries && 
        matches!(self.status, PermissionStatus::Denied | PermissionStatus::Restricted)
    }
    
    /// Increment retry counter
    ///
    /// Should only be called after can_retry() returns true
    #[inline]
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
    
    /// Get remaining retry attempts
    #[inline]
    pub fn remaining_retries(&self) -> u8 {
        self.max_retries.saturating_sub(self.retry_count)
    }
}

/// Semantic component for permission card title text
#[derive(Component, Debug)]
pub struct PermissionCardTitle;

/// Semantic component for permission card description text
#[derive(Component, Debug)] 
pub struct PermissionCardDescription;

/// Semantic component for permission card action button
#[derive(Component, Debug)]
pub struct PermissionCardButton;

/// Semantic component for permission card status indicator
#[derive(Component, Debug)]
pub struct PermissionCardStatus;

/// Semantic component for permission card requirement indicator
#[derive(Component, Debug)]
pub struct PermissionCardRequirement;

/// Component for wizard navigation buttons (Back, Next, Skip)
#[derive(Component, Debug)]
pub struct WizardNavigationButton {
    /// The action this button performs
    pub action: NavigationAction,
    /// Whether this button is currently enabled
    pub is_enabled: bool,
    /// Whether this button is visible in current state
    pub is_visible: bool,
    /// Button text (for accessibility)
    pub text: &'static str,
}

/// Navigation actions available in the wizard
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NavigationAction {
    /// Go to previous step
    Back,
    /// Go to next step
    Next,
    /// Skip current step or entire wizard
    Skip,
    /// Cancel and exit wizard
    Cancel,
    /// Complete the wizard
    Finish,
}

impl NavigationAction {
    /// Get a human-readable description of this action
    pub fn description(self) -> &'static str {
        match self {
            NavigationAction::Back => "Return to previous step",
            NavigationAction::Next => "Continue to next step", 
            NavigationAction::Skip => "Skip this step",
            NavigationAction::Cancel => "Cancel wizard setup",
            NavigationAction::Finish => "Complete wizard setup",
        }
    }
}

impl WizardNavigationButton {
    /// Create a new navigation button
    pub fn new(action: NavigationAction) -> Self {
        let text = match action {
            NavigationAction::Back => "Back",
            NavigationAction::Next => "Next",
            NavigationAction::Skip => "Skip",
            NavigationAction::Cancel => "Cancel",
            NavigationAction::Finish => "Finish",
        };
        
        Self {
            action,
            is_enabled: true,
            is_visible: true,
            text,
        }
    }
    
    /// Update button state based on wizard state
    pub fn update_for_state(&mut self, wizard_state: WizardState) {
        match (self.action, wizard_state) {
            (NavigationAction::Back, WizardState::Welcome) => {
                self.is_visible = false;
            },
            (NavigationAction::Next, WizardState::Complete) => {
                self.is_visible = false;
            },
            (NavigationAction::Finish, WizardState::Complete) => {
                self.is_visible = true;
                self.is_enabled = true;
            },
            (NavigationAction::Finish, _) => {
                self.is_visible = false;
            },
            _ => {
                self.is_visible = true;
                self.is_enabled = true;
            },
        }
    }
}

/// Component for wizard progress indicators
#[derive(Component, Debug)]
pub struct WizardProgressIndicator {
    /// Current progress (0.0 to 1.0)
    pub progress: f32,
    /// Current step number (1-based)
    pub current_step: u8,
    /// Total number of steps
    pub total_steps: u8,
    /// Whether to show detailed progress
    pub show_details: bool,
    /// Animation state for smooth progress updates
    pub animation_progress: f32,
}

impl WizardProgressIndicator {
    /// Create a new progress indicator
    pub fn new(total_steps: u8) -> Self {
        Self {
            progress: 0.0,
            current_step: 1,
            total_steps,
            show_details: true,
            animation_progress: 0.0,
        }
    }
    
    /// Update progress based on wizard state
    pub fn update_for_state(&mut self, state: WizardState) {
        self.current_step = match state {
            WizardState::NotStarted => 0,
            WizardState::Welcome => 1,
            WizardState::CheckingPermissions => 2,
            WizardState::RequestingPermissions => 3,
            WizardState::SettingUpHotkeys => 4,
            WizardState::Complete => self.total_steps,
        };
        
        self.progress = state.progress_percentage();
    }
    
    /// Get progress text for display
    pub fn progress_text(&self) -> String {
        if self.show_details {
            format!("Step {} of {}", self.current_step, self.total_steps)
        } else {
            format!("{}%", (self.progress * 100.0) as u8)
        }
    }
}

impl Default for WizardProgressIndicator {
    fn default() -> Self {
        Self::new(4) // Default: Welcome, Permissions, Hotkeys, Complete
    }
}

/// Component for permission status badges/icons
#[derive(Component, Debug)]
pub struct PermissionStatusBadge {
    /// The permission this badge represents
    pub permission_type: PermissionType,
    /// Current status being displayed
    pub status: PermissionStatus,
    /// Size of the badge (for responsive design)
    pub size: BadgeSize,
    /// Whether to show text label with icon
    pub show_label: bool,
}

/// Badge sizes for different contexts
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BadgeSize {
    /// Small badge for compact views
    Small,
    /// Medium badge for normal views
    Medium,
    /// Large badge for detailed views
    Large,
}

impl PermissionStatusBadge {
    /// Create a new status badge
    pub fn new(permission_type: PermissionType, size: BadgeSize) -> Self {
        Self {
            permission_type,
            status: PermissionStatus::Unknown,
            size,
            show_label: matches!(size, BadgeSize::Medium | BadgeSize::Large),
        }
    }
    
    /// Get the icon character for this status
    #[inline]
    pub fn icon_char(&self) -> char {
        match self.status {
            PermissionStatus::Unknown => '?',
            PermissionStatus::Authorized => '✓',
            PermissionStatus::Denied => '✗',
            PermissionStatus::Restricted => '!',
            PermissionStatus::NotDetermined => '◯',
        }
    }
    
    /// Get the size in pixels for this badge
    #[inline]
    pub fn pixel_size(&self) -> f32 {
        match self.size {
            BadgeSize::Small => 16.0,
            BadgeSize::Medium => 24.0,
            BadgeSize::Large => 32.0,
        }
    }
}

/// Bundle for creating a complete permission card entity
#[derive(Bundle)]
pub struct PermissionCardBundle {
    /// The permission card component
    pub card: PermissionCard,
    /// Entity name for debugging
    pub name: Name,
}

impl PermissionCardBundle {
    /// Create a new permission card bundle
    pub fn new(permission_type: PermissionType, is_required: bool) -> Self {
        let name_str = format!("PermissionCard_{:?}", permission_type);
        
        Self {
            card: PermissionCard::new(permission_type, is_required),
            name: Name::new(name_str),
        }
    }
}