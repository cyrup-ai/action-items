use action_items_ecs_ui::accessibility::{AccessibleElement, LiveRegion};

/// Accessibility utility functions
impl AccessibleElement {
    /// Create accessible element for search results
    #[allow(dead_code)] // Infrastructure for future accessibility implementation
    pub fn search_result(title: &str, description: &str, index: usize) -> Self {
        Self {
            role: accesskit::Role::Button,
            name: format!("{title}: {description}"),
            description: Some("Press Enter to execute this action".to_string()),
            focusable: true,
            tab_index: Some(index as i32 + 2),
            ..Default::default()
        }
    }

    /// Create accessible element for search input
    #[allow(dead_code)] // Infrastructure for future accessibility implementation
    pub fn search_input() -> Self {
        Self {
            role: accesskit::Role::TextInput,
            name: "Search for applications and actions".to_string(),
            description: Some("Type to search for applications, files, and actions".to_string()),
            focusable: true,
            tab_index: Some(0),
            ..Default::default()
        }
    }

    /// Create accessible element for launcher container
    #[allow(dead_code)] // Infrastructure for future accessibility implementation
    pub fn launcher_container() -> Self {
        Self {
            role: accesskit::Role::Application,
            name: "Action Items Launcher".to_string(),
            description: Some("Quick application launcher with search functionality".to_string()),
            focusable: false,
            live_region: Some(LiveRegion::Polite),
            ..Default::default()
        }
    }
}
