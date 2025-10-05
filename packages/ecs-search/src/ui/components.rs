use bevy::prelude::*;

/// Search bar component (container for input)
#[derive(Component, Debug)]
pub struct SearchBarComponent {
    pub is_focused: bool,
    pub placeholder: String,
}

impl Default for SearchBarComponent {
    fn default() -> Self {
        Self {
            is_focused: false,
            placeholder: "Search...".to_string(),
        }
    }
}

/// Search input field marker
#[derive(Component, Debug)]
pub struct SearchInputField;

/// Search results container marker
#[derive(Component, Debug)]
pub struct SearchResultsContainer;

/// Individual result item (stores index for selection)
#[derive(Component, Debug)]
pub struct SearchResultItem {
    pub index: usize,
    pub result_id: String,
    pub score: f32,
}

/// Search icon marker
#[derive(Component, Debug)]
pub struct SearchIcon;

/// Current selection state
#[derive(Resource, Debug, Default)]
pub struct SearchSelection {
    pub selected_index: usize,
    pub total_results: usize,
}

impl SearchSelection {
    pub fn select_next(&mut self) {
        if self.total_results > 0 {
            self.selected_index = (self.selected_index + 1) % self.total_results;
        }
    }
    
    pub fn select_previous(&mut self) {
        if self.total_results > 0 {
            self.selected_index = if self.selected_index == 0 {
                self.total_results - 1
            } else {
                self.selected_index - 1
            };
        }
    }
}

/// UI state from packages/ui (migrate or reference)
#[derive(Component, Debug, Default)]
pub struct SearchUIState {
    pub search_loading: bool,
    pub search_progress: f32,
    pub animating_results: usize,
}

impl SearchUIState {
    pub fn start_loading(&mut self) {
        self.search_loading = true;
        self.search_progress = 0.0;
    }
    
    pub fn complete_loading(&mut self) {
        self.search_loading = false;
        self.search_progress = 1.0;
    }
    
    pub fn update_progress(&mut self, delta_secs: f32) {
        if self.search_loading {
            self.search_progress += delta_secs * 3.0;
            self.search_progress = self.search_progress.min(0.9);
        }
    }
}
