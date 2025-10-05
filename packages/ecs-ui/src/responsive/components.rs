use bevy::prelude::*;

/// Default maximum iterations for text truncation convergence
const DEFAULT_MAX_ITERATIONS: u8 = 10;

/// ViewportResponsiveContainer component for viewport-responsive sizing
///
/// Uses Bevy's viewport units (Vw/Vh) for optimal positioning across screen sizes.
/// Perfect for centered modal/dialog layouts that need consistent sizing on different displays.
///
/// # Viewport Units
/// - `Val::Vw(60.0)` = 60% of viewport width
/// - `Val::Vh(50.0)` = 50% of viewport height
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::responsive::ViewportResponsiveContainer;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         Node {
///             ..ViewportResponsiveContainer::default().to_node_style()
///         },
///         ViewportResponsiveContainer::default(),
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct ViewportResponsiveContainer {
    /// Primary width as percentage of screen width (60% default)
    pub width_vw: f32,
    /// Primary height as percentage of screen height (50% default)
    pub height_vh: f32,
    /// Maximum width constraint as percentage of screen width (90% max)
    pub max_width_vw: f32,
    /// Minimum width constraint as percentage of screen width (30% min)
    pub min_width_vw: f32,
    /// Maximum height constraint as percentage of screen height (80% max)
    pub max_height_vh: f32,
    /// Minimum height constraint as percentage of screen height (20% min)
    pub min_height_vh: f32,
    /// Flex direction for container layout (Column for vertical stacking)
    pub flex_direction: FlexDirection,
    /// Horizontal alignment for container positioning
    pub align_items: AlignItems,
    /// Vertical alignment for container positioning
    pub justify_content: JustifyContent,
    /// Self alignment for container centering
    pub align_self: AlignSelf,
}

impl Default for ViewportResponsiveContainer {
    fn default() -> Self {
        Self {
            width_vw: 60.0,                             // 60% of screen width
            height_vh: 50.0,                            // 50% of screen height
            max_width_vw: 90.0,                         // Max 90% of screen width
            min_width_vw: 30.0,                         // Min 30% of screen width
            max_height_vh: 80.0,                        // Max 80% of screen height
            min_height_vh: 20.0,                        // Min 20% of screen height
            flex_direction: FlexDirection::Column,      // Vertical stacking
            align_items: AlignItems::Center,            // Center horizontally
            justify_content: JustifyContent::FlexStart, // Start from top
            align_self: AlignSelf::Center,              // Center container itself
        }
    }
}

impl ViewportResponsiveContainer {
    /// Generate Bevy Node style with viewport-responsive sizing
    ///
    /// Zero-allocation style generation with optimized viewport unit usage.
    /// Converts percentage values to Bevy's Val::Vw/Vh types.
    #[inline]
    pub fn to_node_style(&self) -> Node {
        Node {
            width: Val::Vw(self.width_vw),
            height: Val::Vh(self.height_vh),
            max_width: Val::Vw(self.max_width_vw),
            min_width: Val::Vw(self.min_width_vw),
            max_height: Val::Vh(self.max_height_vh),
            min_height: Val::Vh(self.min_height_vh),
            flex_direction: self.flex_direction,
            align_items: self.align_items,
            justify_content: self.justify_content,
            align_self: self.align_self,
            padding: UiRect::all(Val::Px(12.0)),
            margin: UiRect::all(Val::Px(0.0)),
            border: UiRect::all(Val::Px(0.0)),
            position_type: PositionType::Relative,
            overflow: Overflow::clip(),
            ..default()
        }
    }

    /// Create compact variant for smaller screens or constrained layouts
    #[inline]
    pub fn compact() -> Self {
        Self {
            width_vw: 45.0,      // Smaller width for compact layout
            height_vh: 40.0,     // Smaller height for compact layout
            max_width_vw: 70.0,  // Reduced max width
            min_width_vw: 25.0,  // Reduced min width
            max_height_vh: 60.0, // Reduced max height
            min_height_vh: 15.0, // Reduced min height
            ..Default::default()
        }
    }

    /// Create expanded variant for larger screens or full-feature mode
    #[inline]
    pub fn expanded() -> Self {
        Self {
            width_vw: 75.0,      // Larger width for expanded layout
            height_vh: 65.0,     // Larger height for expanded layout
            max_width_vw: 95.0,  // Increased max width
            min_width_vw: 40.0,  // Increased min width
            max_height_vh: 90.0, // Increased max height
            min_height_vh: 30.0, // Increased min height
            ..Default::default()
        }
    }

    /// Validate viewport percentages for safety
    ///
    /// Returns true if all viewport values are within reasonable bounds (0-100%)
    /// and min/max constraints are properly ordered.
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.width_vw > 0.0
            && self.width_vw <= 100.0
            && self.height_vh > 0.0
            && self.height_vh <= 100.0
            && self.max_width_vw >= self.min_width_vw
            && self.max_height_vh >= self.min_height_vh
            && self.min_width_vw > 0.0
            && self.min_width_vw <= 100.0
            && self.min_height_vh > 0.0
            && self.min_height_vh <= 100.0
            && self.max_width_vw > 0.0
            && self.max_width_vw <= 100.0
            && self.max_height_vh > 0.0
            && self.max_height_vh <= 100.0
    }
}

/// ContentConstraints component for managing content overflow
///
/// Defines visible item limits and maximum content height for scrollable areas.
/// Commonly used with list UIs, search results, or any bounded content display.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::responsive::ContentConstraints;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         ContentConstraints {
///             max_visible_results: 8,
///             result_height: 48.0,
///             max_content_height: 384.0,
///         },
///         // ... other components
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct ContentConstraints {
    /// Maximum number of items visible without scrolling (8 default like Raycast)
    pub max_visible_results: usize,
    /// Height of each individual item in pixels (48.0 default)
    pub result_height: f32,
    /// Maximum total height for content area (max_visible_results * result_height)
    pub max_content_height: f32,
}

impl Default for ContentConstraints {
    fn default() -> Self {
        Self {
            max_visible_results: 8,
            result_height: 48.0,
            max_content_height: 8.0 * 48.0, // 384.0
        }
    }
}

impl ContentConstraints {
    /// Create constraints with custom item count and height
    pub fn new(max_visible_results: usize, result_height: f32) -> Self {
        Self {
            max_visible_results,
            result_height,
            max_content_height: max_visible_results as f32 * result_height,
        }
    }

    /// Calculate total height needed for given item count
    pub fn height_for_items(&self, item_count: usize) -> f32 {
        (item_count.min(self.max_visible_results) as f32) * self.result_height
    }

    /// Check if content will overflow (requires scrolling)
    pub fn will_overflow(&self, item_count: usize) -> bool {
        item_count > self.max_visible_results
    }
}

/// TextTruncation component for managing text overflow
///
/// Prevents horizontal expansion by defining maximum text width and ellipsis behavior.
/// Used with text entities that should truncate rather than wrap or expand containers.
///
/// # Example
/// ```rust
/// use bevy::prelude::*;
/// use action_items_ecs_ui::responsive::TextTruncation;
///
/// fn setup(mut commands: Commands) {
///     commands.spawn((
///         Text::from_section("Very long text that might overflow...", TextStyle::default()),
///         TextTruncation {
///             max_width: 500.0,
///             ellipsis: "...".to_string(),
///         },
///     ));
/// }
/// ```
#[derive(Component, Debug, Clone)]
pub struct TextTruncation {
    /// Maximum width for text before truncation (in pixels, 500.0 default)
    pub max_width: f32,
    /// Ellipsis string to append when text is truncated ("..." default)
    pub ellipsis: String,
}

impl Default for TextTruncation {
    fn default() -> Self {
        Self {
            max_width: 500.0,
            ellipsis: "...".to_string(),
        }
    }
}

impl TextTruncation {
    /// Create truncation with custom max width
    pub fn with_max_width(max_width: f32) -> Self {
        assert!(
            max_width > 0.0 && max_width.is_finite(),
            "TextTruncation max_width must be finite and > 0.0, got: {}",
            max_width
        );
        Self {
            max_width,
            ..Default::default()
        }
    }

    /// Create truncation with custom ellipsis
    pub fn with_ellipsis(ellipsis: impl Into<String>) -> Self {
        Self {
            ellipsis: ellipsis.into(),
            ..Default::default()
        }
    }

    /// Check if text needs truncation based on measured width
    /// 
    /// This is a utility method for manual truncation logic.
    /// The text_truncation_system automatically handles truncation using TextLayoutInfo.
    /// 
    /// # Arguments
    /// * `text_width` - Actual measured pixel width from TextLayoutInfo.size.x
    /// 
    /// # Example
    /// ```rust
    /// let layout_info: TextLayoutInfo = /* from query */;
    /// let truncation = TextTruncation::default();
    /// if truncation.needs_truncation(layout_info.size.x) {
    ///     // Manual truncation logic
    /// }
    /// ```
    pub fn needs_truncation(&self, text_width: f32) -> bool {
        text_width > self.max_width
    }

    /// Validate that configuration is valid for use
    pub fn is_valid(&self) -> bool {
        self.max_width > 0.0 && self.max_width.is_finite()
    }
}

/// State tracking for iterative text truncation with real measurements
///
/// Tracks original text and iteration count to prevent infinite loops
/// during the truncation-measure-truncate cycle.
///
/// Automatically added by text_truncation_system when truncation is needed.
/// Automatically removed when text fits within constraints.
#[derive(Component, Debug, Clone)]
pub struct TruncationState {
    /// Original untruncated text for restoration if needed
    pub original_text: String,
    /// Current truncation iteration count
    pub iteration_count: u8,
    /// Maximum iterations before giving up (prevents infinite loops)
    pub max_iterations: u8,
}

impl Default for TruncationState {
    fn default() -> Self {
        Self {
            original_text: String::new(),
            iteration_count: 0,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }
}

impl TruncationState {
    /// Create new state with original text
    pub fn new(original_text: String) -> Self {
        Self {
            original_text,
            iteration_count: 0,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }
    
    /// Check if we've exhausted our iteration budget
    pub fn is_exhausted(&self) -> bool {
        self.iteration_count >= self.max_iterations
    }
    
    /// Increment iteration count
    pub fn increment(&mut self) {
        self.iteration_count += 1;
    }
}
