use bevy::prelude::*;
use crate::navigation::SettingsTab;
use ecs_service_bridge::components::PluginType;

#[derive(Component)]
pub struct SettingsWindow;

#[derive(Component)]
pub struct SettingsSidebar;

#[derive(Component)]
pub struct SettingsTabButton {
    pub tab: SettingsTab,
}

#[derive(Component)]
pub struct SettingsContentArea {
    pub active_tab: SettingsTab,
}

#[derive(Component)]
pub struct SettingControl {
    pub field_name: String,
    pub table: String,  // Database table for this setting
}

#[derive(Component)]
pub struct SettingCheckbox {
    pub checked: bool,
}

#[derive(Component)]
pub struct TextInput {
    pub field_name: String,
    pub value: String,
}

#[derive(Component)]
pub struct DropdownControl {
    pub field_name: String,
    pub options: Vec<String>,
    pub selected: usize,
    pub is_open: bool,
}

#[derive(Component)]
pub struct HotkeyRecorder {
    pub field_name: String,
    pub current_combo: String,
    pub is_recording: bool,
}

#[derive(Component)]
pub struct ExtensionsTableContainer;

#[derive(Component)]
pub struct ExtensionRow {
    pub plugin_id: String,
}

#[derive(Component)]
pub struct ExtensionToggle {
    pub plugin_id: String,
    pub enabled: bool,
}

#[derive(Component)]
pub struct SettingErrorDisplay {
    pub field_name: String,
}

#[derive(Component)]
pub struct ErrorMessage {
    pub timeout: Timer,
}

#[derive(Component)]
pub struct SaveSuccessFeedback {
    pub timer: Timer,
    pub original_color: Color,
}

#[derive(Component)]
pub struct WindowModeCard {
    pub mode: WindowMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowMode {
    Default,
    Compact,
}

#[derive(Component)]
pub struct ThemeStudioButton;

#[derive(Component)]
pub struct ExtensionSearchBar;

#[derive(Component)]
pub struct ExtensionStoreButton;

#[derive(Component)]
pub struct ExtensionFilterPill {
    pub filter_type: PluginType,
    pub active: bool,
}

#[derive(Component)]
pub struct ExtensionSettingsButton {
    pub plugin_id: String,
}

/// About tab: Visit Website button
#[derive(Component)]
pub struct VisitWebsiteButton;

/// About tab: Send Feedback button
#[derive(Component)]
pub struct SendFeedbackButton;

/// About tab: Acknowledgements link
#[derive(Component)]
pub struct AcknowledgementsLink;

/// About tab: App logo placeholder
#[derive(Component)]
pub struct AboutAppLogo;

/// Marker for backdrop overlay
#[derive(Component)]
pub struct SettingsBackdrop;

/// Marker for modal root container
#[derive(Component)]
pub struct SettingsModalRoot;

/// Marker for title bar
#[derive(Component)]
pub struct SettingsTitleBar;

/// Marker for close button
#[derive(Component)]
pub struct CloseSettingsButton;

/// Marker for the profile sidebar container (left 25%)
#[derive(Component)]
pub struct UserProfileSidebar;

/// Profile photo component with user initials and optional avatar URL
#[derive(Component)]
pub struct ProfilePhoto {
    pub initials: String,
    pub avatar_url: Option<String>,
}

/// Subscription status display box
#[derive(Component)]
pub struct SubscriptionStatusBox;

/// Individual feature row in Pro/Organizations/Developer sections
#[derive(Component)]
pub struct FeatureRow {
    pub feature_id: String,
    pub section: String,  // "pro", "organizations", "developer"
}

/// Blue "Pro" badge component
#[derive(Component)]
pub struct ProBadge;

/// Info icon "ⓘ" component
#[derive(Component)]
pub struct InfoIcon;

/// Red "Log Out" button
#[derive(Component)]
pub struct LogOutButton;

/// Gray "Manage Subscription" button
#[derive(Component)]
pub struct ManageSubscriptionButton;

/// Organizations tab components

#[derive(Component)]
pub struct OrganizationsScreen;

#[derive(Component)]
pub struct OrganizationsSidebar;

#[derive(Component)]
pub struct OrganizationListItem {
    pub org_id: String,
    pub selected: bool,
}

#[derive(Component)]
pub struct CreateOrgButton;

#[derive(Component)]
pub struct OrganizationLogo {
    pub org_id: String,
}

#[derive(Component)]
pub struct PlanBadge {
    pub plan_type: String,  // "Free", "Pro", "Enterprise"
}

#[derive(Component)]
pub struct InfoCard;

#[derive(Component)]
pub struct ManageOrgButton {
    pub org_id: String,
}

#[derive(Component)]
pub struct EditOrgButton {
    pub org_id: String,
}

#[derive(Component)]
pub struct OpenStoreButton {
    pub org_id: String,
}

#[derive(Component)]
pub struct LeaveOrgButton {
    pub org_id: String,
}

#[derive(Component)]
pub struct ManageOrgSubscriptionButton {
    pub org_id: String,
}
