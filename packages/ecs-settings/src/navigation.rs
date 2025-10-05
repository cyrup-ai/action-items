use serde::{Deserialize, Serialize};

/// Settings tab enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SettingsTab {
    General,
    Extensions,
    AI,
    CloudSync,
    Account,
    Organizations,
    Advanced,
    About,
}

impl Default for SettingsTab {
    fn default() -> Self {
        Self::Extensions  // From original code
    }
}

impl SettingsTab {
    pub fn all() -> &'static [SettingsTab] {
        &[
            SettingsTab::General,
            SettingsTab::Extensions,
            SettingsTab::AI,
            SettingsTab::CloudSync,
            SettingsTab::Account,
            SettingsTab::Organizations,
            SettingsTab::Advanced,
            SettingsTab::About,
        ]
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            SettingsTab::General => "General",
            SettingsTab::Extensions => "Extensions",
            SettingsTab::AI => "AI",
            SettingsTab::CloudSync => "Cloud Sync",
            SettingsTab::Account => "Account",
            SettingsTab::Organizations => "Organizations",
            SettingsTab::Advanced => "Advanced",
            SettingsTab::About => "About",
        }
    }
}

/// Extension filter types (from mod.rs line 97-103)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExtensionFilter {
    All,
    Commands,
    Scripts,
    Apps,
    Quicklinks,
}

impl Default for ExtensionFilter {
    fn default() -> Self {
        Self::All
    }
}
