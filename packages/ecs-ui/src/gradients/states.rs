//! Gradient state types for interaction tracking and component classification

/// Interaction state for gradient animations
/// 
/// Zero-allocation state tracking for smooth gradient transitions.
/// Used by GradientComponent to determine which gradient variant to apply.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientInteractionState {
    /// Default state - no interaction
    Default,
    /// Hover state - cursor over element
    Hover,
    /// Selected state - element is selected/active
    Selected,
    /// Pressed state - element is being clicked
    Pressed,
    /// Disabled state - element is non-interactive
    Disabled,
}

/// Component types for gradient theming
/// 
/// Categorizes UI components for theme-based gradient selection.
/// Each variant maps to a specific gradient in GradientTheme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GradientComponentType {
    /// Primary container - main backgrounds (e.g., launcher background)
    PrimaryContainer,
    /// Secondary container - elevated surfaces (e.g., search input)
    SecondaryContainer,
    /// List item - generic interactive list items (renamed from ResultItem)
    ListItem,
    /// List item selected - selected state variant (renamed from ResultItemSelected)
    ListItemSelected,
    /// Text overlay - text enhancement backgrounds
    TextOverlay,
    /// Border accent - subtle borders and dividers
    BorderAccent,
    /// Success state - confirmations and positive actions
    SuccessState,
    /// Warning state - cautions and attention needed
    WarningState,
    /// Error state - failures and critical issues
    ErrorState,
}
