use bevy::prelude::*;

#[derive(Component)]
pub struct ActionResultsContainer;

#[derive(Component)]
pub struct ResultsContainer;

#[derive(Component)]
pub struct ActionResultItem {
    pub index: usize,
}

#[derive(Component)]
pub struct ResultIcon {
    pub result_id: String,
    pub loading: bool,
    pub image_handle: Option<Handle<Image>>,
}

#[derive(Component)]
pub struct ImageComponent(pub Handle<Image>);

/// Action Items search result item with enhanced styling and interaction states
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultItem {
    pub action_id: String,
    pub is_selected: bool,
    pub index: usize,
}

/// Component marker for search result item icons with enhanced loading states
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultIcon {
    pub result_id: String,
    pub loading: bool,
    pub image_handle: Option<Handle<Image>>,
    pub fallback_text: Option<String>, // For text-based icons like file extensions
}
/// Component marker for search result item title with enhanced typography
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultTitle;

/// Component marker for search result item subtitle/description with secondary styling
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultSubtitle;

/// Component marker for keyboard shortcut indicators with badge styling
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultShortcut;

/// Component marker for interactive search result backgrounds with hover states
#[derive(Component, Debug)]
pub struct ActionItemsSearchResultBackground;

/// Enhanced search result display data with Action Items metadata
#[derive(Debug, Clone)]
pub struct ActionItemsSearchResultData {
    pub title: String,
    pub subtitle: Option<String>,
    pub icon_path: Option<String>,
    pub shortcut: Option<String>,
    pub action_id: String,
    pub category: Option<String>,
    pub score: f32,     // Search relevance score for sorting
    pub ranking: usize, // Display ranking position (0-based index)
}

impl ActionItemsSearchResultData {
    /// Create new search result with minimum required fields
    pub fn new(title: String, action_id: String) -> Self {
        Self {
            title,
            subtitle: None,
            icon_path: None,
            shortcut: None,
            action_id,
            category: None,
            score: 1.0,
            ranking: 0,
        }
    }

    /// Add subtitle for additional context
    pub fn with_subtitle(mut self, subtitle: String) -> Self {
        self.subtitle = Some(subtitle);
        self
    }
    /// Add icon path for visual identification
    pub fn with_icon(mut self, icon_path: String) -> Self {
        self.icon_path = Some(icon_path);
        self
    }

    /// Add keyboard shortcut for power users
    pub fn with_shortcut(mut self, shortcut: String) -> Self {
        self.shortcut = Some(shortcut);
        self
    }

    /// Add category for organization and filtering
    pub fn with_category(mut self, category: String) -> Self {
        self.category = Some(category);
        self
    }

    /// Set relevance score for result ranking
    pub fn with_score(mut self, score: f32) -> Self {
        self.score = score;
        self
    }

    /// Set ranking position for display ordering
    pub fn with_ranking(mut self, ranking: usize) -> Self {
        self.ranking = ranking;
        self
    }
}

/// Animation components
pub type HoverEffectQuery<'world, 'state> =
    Query<'world, 'state, (&'world mut Transform, &'world ActionResultItem)>;

/// Resource for fallback icons
#[derive(Resource, Default)]
pub struct FallbackIcon(pub Option<Handle<Image>>);
