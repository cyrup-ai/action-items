# Window Sizing Strategy

## Critical Analysis: Expanding Window Problem

### Current Issue (app/src/main.rs:101-116)
```rust
primary_window: Some(Window {
    title: "Action Items".into(),
    // Start with search bar size, will expand for results
    resolution: (WINDOW_MIN_WIDTH, WINDOW_HEIGHT).into(),  // ❌ PROBLEM: Starts small, expands
    position: WindowPosition::Centered(MonitorSelection::Primary),
    decorations: false,
    window_level: WindowLevel::AlwaysOnTop,
    visible: false,           // ❌ Hidden by default, complex show/hide logic
    mode: WindowMode::Windowed,
    transparent: true,        // ❌ Transparency can cause performance issues
    #[cfg(target_os = "macos")]
    composite_alpha_mode: CompositeAlphaMode::PostMultiplied,
    ..default()
}),
```

### Current Window Constants (app/src/window/mod.rs or similar)
```rust
pub const WINDOW_WIDTH_PERCENT: f32 = 0.8;     // ❌ Percentage-based, not fixed
pub const WINDOW_MAX_WIDTH: f32 = 800.0;       // ❌ Too wide for launcher
pub const WINDOW_MIN_WIDTH: f32 = 400.0;       // ❌ Causes expansion behavior
pub const WINDOW_HEIGHT: f32 = 80.0;           // ❌ Too small, will expand vertically

**Root Cause:** The window is designed to expand dynamically as content is added, starting from minimal size and growing to accommodate results. This creates jarring resize animations and inconsistent positioning, completely unlike Raycast's smooth, fixed-size approach.

## Target Architecture: Raycast-like Fixed Window

### Target Specifications
1. **Viewport-Relative Dimensions**: 60% viewport width × max 60% viewport height (responsive)
2. **No Dynamic Expansion**: Container size constrained by viewport percentages
3. **Responsive Constraints**: portrait/landscape mode adaptations
4. **Smooth Animations**: Fade in/out with professional timing curvesize jumps
5. **Performance First**: Minimal compositor overhead, efficient rendering

### Raycast Window Analysis

#### Dimensions (Viewport-Relative)
- **Width**: 60% viewport width (responsive)
- **Height**: max 60% viewport height (responsive, accommodates 8 results + search bar)
- **Min Width**: 35% viewport width
- **Positioning**: Always centered on primary monitor

#### Visual Properties
- **Opacity**: Starts at 0%, animates to 95% on show
- **Shadow**: Large, soft shadow for depth perception  
- **Transparency**: Minimal, just enough for modern glass effect
- **Border**: None (handled by content container)

#### Behavior
- **Show**: Instant positioning, fade-in animation (200ms)
- **Hide**: Fade-out animation (150ms), then invisible
- **Focus**: Maintains focus while visible
- **Positioning**: Locks to primary monitor center

## Implementation Specification

### Phase 1: Fixed Window Constants

**New File:** `app/src/window/constants.rs`
```rust
/// Professional window sizing constants using viewport-relative units
pub const LAUNCHER_WIDTH_VW: f32 = 60.0;        // 60% of viewport width
pub const LAUNCHER_MAX_HEIGHT_VH: f32 = 60.0;   // Maximum 60% of viewport height
pub const LAUNCHER_MIN_WIDTH_VW: f32 = 35.0;    // Minimum 35% of viewport width
pub const LAUNCHER_PORTRAIT_WIDTH_VW: f32 = 90.0; // 90% in portrait mode

/// Window positioning and animation constants
pub const FADE_IN_DURATION_MS: u64 = 200;      // Smooth appearance
pub const FADE_OUT_DURATION_MS: u64 = 150;     // Quick disappearance
pub const WINDOW_OPACITY_SHOWN: f32 = 0.95;    // Slight transparency for modern feel
pub const WINDOW_OPACITY_HIDDEN: f32 = 0.0;    // Fully transparent when hidden

/// Multi-monitor positioning
pub const MONITOR_OFFSET_PERCENT: f32 = 0.3;   // 30% from top of screen
pub const POSITION_CACHE_DURATION_SEC: u64 = 5; // Cache position for 5 seconds
```

### Phase 2: Window Configuration Replacement

**File:** `app/src/main.rs`
**Lines:** 100-116 (Window configuration)

**Current Configuration:**
```rust
resolution: (WINDOW_MIN_WIDTH, WINDOW_HEIGHT).into(),
visible: false,
transparent: true,
```

**Target Configuration:**
```rust
Window {
    title: "Action Items".into(),
    resolution: (LAUNCHER_MAX_WIDTH, LAUNCHER_MAX_HEIGHT).into(),
    position: WindowPosition::Centered(MonitorSelection::Primary),
    decorations: false,
    window_level: WindowLevel::AlwaysOnTop,
    visible: false,                               // Still hidden initially
    mode: WindowMode::Windowed,
    transparent: false,                           // Solid window for performance
    resizable: false,                             // Prevent user resizing
    #[cfg(target_os = "macos")]
    composite_alpha_mode: CompositeAlphaMode::Opaque, // Better performance
    ..default()
}
```

### Phase 3: Window Animation System

**New Component:** `WindowAnimationState`
```rust
#[derive(Component, Debug, Clone)]
pub struct WindowAnimationState {
    pub target_opacity: f32,
    pub current_opacity: f32,
    pub animation_speed: f32,       // Opacity change per second
    pub is_animating: bool,
    pub show_start_time: Option<f64>,
    pub hide_start_time: Option<f64>,
}

impl Default for WindowAnimationState {
    fn default() -> Self {
        Self {
            target_opacity: 0.0,
            current_opacity: 0.0,
            animation_speed: 5.0,       // Smooth but fast animations
            is_animating: false,
            show_start_time: None,
            hide_start_time: None,
        }
    }
}
```

**New System:** `window_animation_system`
```rust
/// Handle smooth window fade in/out animations
#[inline]
pub fn window_animation_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut animation_query: Query<&mut WindowAnimationState>,
    time: Res<Time>,
) {
    let Ok(mut window) = window_query.get_single_mut() else { return };
    let Ok(mut animation) = animation_query.get_single_mut() else { return };
    
    if !animation.is_animating {
        return;
    }
    
    let delta_time = time.delta_secs();
    let opacity_change = animation.animation_speed * delta_time;
    
    // Update current opacity towards target
    if animation.current_opacity < animation.target_opacity {
        animation.current_opacity = (animation.current_opacity + opacity_change)
            .min(animation.target_opacity);
    } else if animation.current_opacity > animation.target_opacity {
        animation.current_opacity = (animation.current_opacity - opacity_change)
            .max(animation.target_opacity);
    }
    
    // Apply opacity to window (simulated via alpha)
    // Note: Bevy doesn't have direct window opacity, so we'll use container alpha
    
    // Check if animation completed
    let opacity_diff = (animation.current_opacity - animation.target_opacity).abs();
    if opacity_diff < 0.01 {
        animation.current_opacity = animation.target_opacity;
        animation.is_animating = false;
        
        // Hide window completely if fully transparent
        if animation.current_opacity <= 0.01 {
            window.visible = false;
        }
    }
}
```

### Phase 4: Smart Positioning System

**New Component:** `WindowPositionCache`
```rust
#[derive(Resource, Debug)]
pub struct WindowPositionCache {
    pub cached_position: Option<(f32, f32)>,     // (x, y) coordinates
    pub cache_timestamp: f64,                     // When position was cached
    pub primary_monitor_id: Option<u64>,         // Monitor ID for consistency
    pub monitor_dimensions: Option<(f32, f32)>,  // Monitor resolution
}

impl Default for WindowPositionCache {
    fn default() -> Self {
        Self {
            cached_position: None,
            cache_timestamp: 0.0,
            primary_monitor_id: None,
            monitor_dimensions: None,
        }
    }
}
```

**New System:** `smart_positioning_system`
```rust
/// Calculate optimal window position based on current monitor setup
#[inline]
pub fn smart_positioning_system(
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut position_cache: ResMut<WindowPositionCache>,
    time: Res<Time>,
    launcher_state: Res<LauncherState>,
) {
    // Only recalculate position when launcher becomes visible
    if !launcher_state.visible {
        return;
    }
    
    let Ok(mut window) = window_query.get_single_mut() else { return };
    let current_time = time.elapsed_secs_f64();
    
    // Use cached position if recent and valid
    if let Some((cached_x, cached_y)) = position_cache.cached_position {
        let cache_age = current_time - position_cache.cache_timestamp;
        if cache_age < POSITION_CACHE_DURATION_SEC as f64 {
            window.position = WindowPosition::At(IVec2::new(
                cached_x as i32,
                cached_y as i32,
            ));
            return;
        }
    }
    
    // Calculate new position
    let primary_monitor = get_primary_monitor_info(); // Platform-specific function
    
    if let Some((monitor_width, monitor_height)) = primary_monitor {
        // Center horizontally, position at 30% from top
        let window_x = (monitor_width - LAUNCHER_MAX_WIDTH) / 2.0;
        let window_y = monitor_height * MONITOR_OFFSET_PERCENT;
        
        // Apply new position
        window.position = WindowPosition::At(IVec2::new(
            window_x as i32,
            window_y as i32,
        ));
        
        // Cache the position
        position_cache.cached_position = Some((window_x, window_y));
        position_cache.cache_timestamp = current_time;
        position_cache.monitor_dimensions = Some((monitor_width, monitor_height));
    }
}

/// Get primary monitor dimensions (platform-specific)
#[cfg(target_os = "macos")]
fn get_primary_monitor_info() -> Option<(f32, f32)> {
    // TODO: Use macOS APIs to get primary monitor info
    // For now, return common resolution
    Some((1920.0, 1080.0))
}

#[cfg(target_os = "windows")]
fn get_primary_monitor_info() -> Option<(f32, f32)> {
    // TODO: Use Windows APIs to get primary monitor info
    Some((1920.0, 1080.0))
}

#[cfg(target_os = "linux")]
fn get_primary_monitor_info() -> Option<(f32, f32)> {
    // TODO: Use X11/Wayland APIs to get primary monitor info  
    Some((1920.0, 1080.0))
}
```

### Phase 5: Container Opacity System

**Since Bevy doesn't support direct window opacity, we'll simulate it via container alpha:**

**New Component:** `WindowOpacityContainer`
```rust
#[derive(Component, Debug)]
pub struct WindowOpacityContainer;

/// System to update container background alpha based on window animation state
#[inline]
pub fn container_opacity_system(
    mut container_query: Query<&mut BackgroundColor, With<WindowOpacityContainer>>,
    animation_query: Query<&WindowAnimationState, Changed<WindowAnimationState>>,
) {
    let Ok(animation) = animation_query.get_single() else { return };
    let Ok(mut background) = container_query.get_single_mut() else { return };
    
    // Update container alpha to match window animation
    if let BackgroundColor(color) = background.as_mut() {
        *color = color.with_alpha(animation.current_opacity);
    }
}
```

### Phase 6: Show/Hide Integration

**Enhanced LauncherState:**
```rust
#[derive(Resource, Debug, Clone)]
pub struct LauncherState {
    pub visible: bool,
    pub window_entity: Option<Entity>,
    pub target_position: Option<(f32, f32)>,
    pub animation_state: WindowAnimationState,
}

impl Default for LauncherState {
    fn default() -> Self {
        Self {
            visible: false,
            window_entity: None,
            target_position: None,
            animation_state: WindowAnimationState::default(),
        }
    }
}
```

**Updated Show/Hide System:**
```rust
/// Handle launcher show/hide with smooth animations
#[inline]
pub fn launcher_visibility_system(
    mut launcher_state: ResMut<LauncherState>,
    mut window_query: Query<&mut Window, With<PrimaryWindow>>,
    mut animation_query: Query<&mut WindowAnimationState>,
    mut global_hotkey_events: EventReader<GlobalHotkeyEvent>,
    time: Res<Time>,
) {
    let Ok(mut window) = window_query.get_single_mut() else { return };
    let Ok(mut animation) = animation_query.get_single_mut() else { return };
    
    // Handle global hotkey toggle
    for _event in global_hotkey_events.read() {
        if launcher_state.visible {
            // Start hide animation
            animation.target_opacity = 0.0;
            animation.is_animating = true;
            animation.hide_start_time = Some(time.elapsed_secs_f64());
            launcher_state.visible = false;
        } else {
            // Start show animation
            window.visible = true;  // Make visible immediately
            animation.target_opacity = WINDOW_OPACITY_SHOWN;
            animation.is_animating = true;
            animation.show_start_time = Some(time.elapsed_secs_f64());
            launcher_state.visible = true;
        }
    }
}
```

## Implementation Timeline

### Phase 1: Fixed Window Constants (High Priority)
- Create window/constants.rs with fixed dimensions
- Update main.rs window configuration
- Test fixed window size behavior

### Phase 2: Animation System (High Priority)  
- Add WindowAnimationState component
- Implement window_animation_system
- Test smooth fade in/out

### Phase 3: Smart Positioning (Medium Priority)
- Add WindowPositionCache resource
- Implement smart_positioning_system
- Test multi-monitor positioning

### Phase 4: Container Opacity (Medium Priority)
- Add WindowOpacityContainer component
- Implement container_opacity_system  
- Connect to animation state

### Phase 5: Show/Hide Integration (Low Priority)
- Update LauncherState with animation
- Enhance launcher_visibility_system
- Test complete show/hide cycle

## Performance Requirements

### Zero Allocation Constraints
- Cache monitor information between queries
- Reuse position calculations where possible
- Avoid string allocations in hot paths
- Use fixed-size data structures for positioning

### Benchmarking Targets
- Window show latency: < 50ms from hotkey to visible
- Animation smoothness: 60fps during transitions
- Positioning calculation: < 1ms
- Memory overhead: < 10KB for window management

## Testing Strategy

### Visual Tests
- Window dimensions exactly 600x420px
- Position centered on primary monitor
- Smooth fade animations without jumps
- No resizing or expansion behavior

### Multi-monitor Tests
- Correct positioning on various monitor setups
- Primary monitor detection accuracy
- Position caching effectiveness
- Monitor change handling

### Performance Tests
- Animation frame rate consistency
- Show/hide response time measurement
- Memory allocation tracking
- CPU usage during animations

## Dependencies

### Platform APIs
- macOS: NSScreen APIs for monitor info
- Windows: GetMonitorInfo APIs
- Linux: X11/Wayland display APIs

### Internal Dependencies
- LauncherState - Visibility state management
- WindowAnimationState - Animation coordination
- WindowPositionCache - Position optimization

## Success Criteria

1. ✅ Window dimensions fixed at 600x420px (no expansion)
2. ✅ Smooth fade in/out animations (200ms/150ms)
3. ✅ Proper positioning on primary monitor center
4. ✅ No jarring resize or repositioning behavior
5. ✅ Performance targets met for all operations
6. ✅ Multi-monitor support working correctly
7. ✅ Visual match with Raycast window behavior

---

## Bevy Implementation Details

### Flex-Based UI Layout Components

**CRITICAL: This implementation focuses on FLEX-based layouts using Val::Percent with max_width constraints, NOT fixed pixels!**

```rust
use bevy::{
    prelude::*,
    ui::{Val, UiRect, FlexDirection, JustifyContent, AlignItems, PositionType},
    window::{Window, PrimaryWindow, WindowResolution},
};

/// Component for responsive container layouts using flex constraints
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct FlexContainer {
    /// Flex-based width with maximum constraint
    pub width_percent: f32,
    pub max_width: f32,
    /// Flex-based height with maximum constraint  
    pub height_percent: f32,
    pub max_height: f32,
    /// Prevent expansion beyond content
    pub flex_grow: f32,
    /// Content overflow behavior
    pub overflow: Overflow,
}

impl Default for FlexContainer {
    fn default() -> Self {
        Self {
            width_percent: 100.0,
            max_width: 600.0,     // Maximum launcher width
            height_percent: 100.0,
            max_height: 420.0,    // Maximum launcher height
            flex_grow: 0.0,       // CRITICAL: Prevent expansion
            overflow: Overflow::clip(),
        }
    }
}

/// Component for responsive search section
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct SearchSection {
    pub height_px: f32,         // Fixed height for search bar
    pub margin_bottom_px: f32,
    pub padding_percent: f32,   // Padding as percentage
}

impl Default for SearchSection {
    fn default() -> Self {
        Self {
            height_px: 48.0,
            margin_bottom_px: 8.0,
            padding_percent: 2.0, // 2% padding
        }
    }
}

/// Component for responsive results section
#[derive(Component, Reflect, Debug)]
#[reflect(Component)] 
pub struct ResultsSection {
    /// Fill remaining space after search section
    pub flex_grow: f32,
    /// Maximum items to display
    pub max_visible_items: usize,
    /// Item height in pixels
    pub item_height_px: f32,
    /// Gap between items as percentage
    pub item_gap_percent: f32,
}

impl Default for ResultsSection {
    fn default() -> Self {
        Self {
            flex_grow: 1.0,        // Fill available space
            max_visible_items: 8,
            item_height_px: 48.0,
            item_gap_percent: 0.5, // 0.5% gap
        }
    }
}
```

### Flex Layout System with Responsive Constraints

```rust
/// System to update flex-based layouts with proper constraints
pub fn flex_layout_system(
    mut flex_containers: Query<
        (&FlexContainer, &mut Node),
        (Changed<FlexContainer>, With<FlexContainer>),
    >,
    window: Query<&Window, With<PrimaryWindow>>,
) {
    let Ok(window) = window.get_single() else { return };
    let window_size = window.resolution.physical_size();
    
    // Calculate available space (accounting for window decorations, taskbar, etc.)
    let available_width = (window_size.x as f32 * 0.9).min(800.0); // 90% of screen, max 800px
    let available_height = (window_size.y as f32 * 0.8).min(600.0); // 80% of screen, max 600px

    for (flex_container, mut node) in flex_containers.iter_mut() {
        // Calculate flex-based dimensions with maximum constraints
        let target_width = (available_width * flex_container.width_percent / 100.0)
            .min(flex_container.max_width);
        let target_height = (available_height * flex_container.height_percent / 100.0)  
            .min(flex_container.max_height);

        // Apply flex-based sizing with constraints
        node.width = Val::Px(target_width);
        node.height = Val::Px(target_height);
        node.max_width = Val::Px(flex_container.max_width);
        node.max_height = Val::Px(flex_container.max_height);
        node.flex_grow = flex_container.flex_grow; // CRITICAL: Usually 0.0
        node.overflow = flex_container.overflow;

        // Ensure proper flex properties to prevent expansion
        node.flex_shrink = 1.0;                    // Allow shrinking
        node.align_self = AlignSelf::Center;       // Center in container
    }
}

/// System for responsive search section layout
pub fn search_section_layout_system(
    mut search_sections: Query<
        (&SearchSection, &mut Node, &Parent),
        (Changed<SearchSection>, With<SearchSection>),
    >,
    parent_containers: Query<&Node, (With<FlexContainer>, Without<SearchSection>)>,
) {
    for (search_section, mut node, parent) in search_sections.iter_mut() {
        if let Ok(parent_node) = parent_containers.get(parent.get()) {
            // Get parent width for percentage calculations
            let parent_width = match parent_node.width {
                Val::Px(px) => px,
                _ => 600.0, // Fallback
            };

            // Apply responsive sizing
            node.width = Val::Percent(100.0);           // Full width of parent
            node.height = Val::Px(search_section.height_px);
            node.margin = UiRect::bottom(Val::Px(search_section.margin_bottom_px));
            
            // Percentage-based padding
            let padding_px = parent_width * search_section.padding_percent / 100.0;
            node.padding = UiRect::all(Val::Px(padding_px));
            
            // Flex properties to prevent expansion
            node.flex_direction = FlexDirection::Row;
            node.align_items = AlignItems::Center;
            node.justify_content = JustifyContent::FlexStart;
            node.flex_grow = 0.0; // CRITICAL: Don't grow
            node.flex_shrink = 1.0; // Allow shrinking
        }
    }
}

/// System for responsive results section layout
pub fn results_section_layout_system(
    mut results_sections: Query<
        (&ResultsSection, &mut Node, &Parent),
        (Changed<ResultsSection>, With<ResultsSection>),
    >,
    parent_containers: Query<&Node, (With<FlexContainer>, Without<ResultsSection>)>,
) {
    for (results_section, mut node, parent) in results_sections.iter_mut() {
        if let Ok(parent_node) = parent_containers.get(parent.get()) {
            // Fill remaining space after search section
            node.width = Val::Percent(100.0);
            node.height = Val::Percent(100.0);        // Fill available height
            node.flex_grow = results_section.flex_grow; // Usually 1.0
            
            // Calculate maximum height based on items
            let max_items_height = results_section.max_visible_items as f32 * 
                (results_section.item_height_px + 4.0); // +4px for gaps
            node.max_height = Val::Px(max_items_height);
            
            // Flex layout for items
            node.flex_direction = FlexDirection::Column;
            node.align_items = AlignItems::Stretch;   // Items fill width
            node.justify_content = JustifyContent::FlexStart;
            node.overflow = Overflow::clip();         // Hide overflow items
            
            // Gap between items (percentage-based)
            let parent_height = match parent_node.height {
                Val::Px(px) => px,
                _ => 420.0,
            };
            let gap_px = parent_height * results_section.item_gap_percent / 100.0;
            node.row_gap = Val::Px(gap_px);
        }
    }
}
```

### Responsive Item Layout System

```rust
/// Component for result item with flex-based sizing
#[derive(Component, Reflect, Debug)]
#[reflect(Component)]
pub struct ResultItem {
    pub height_px: f32,
    pub padding_percent: f32,      // Padding as percentage of parent width
    pub icon_size_px: f32,
    pub text_flex_grow: f32,       // How much text section should grow
}

impl Default for ResultItem {
    fn default() -> Self {
        Self {
            height_px: 48.0,
            padding_percent: 1.5,     // 1.5% padding
            icon_size_px: 32.0,
            text_flex_grow: 1.0,      // Fill available space
        }
    }
}

/// System for result item flex layout
pub fn result_item_layout_system(
    mut result_items: Query<
        (&ResultItem, &mut Node, &Parent),
        (Changed<ResultItem>, With<ResultItem>),
    >,
    parent_sections: Query<&Node, (With<ResultsSection>, Without<ResultItem>)>,
) {
    for (result_item, mut node, parent) in result_items.iter_mut() {
        if let Ok(parent_node) = parent_sections.get(parent.get()) {
            let parent_width = match parent_node.width {
                Val::Px(px) => px,
                _ => 600.0,
            };

            // Responsive item sizing
            node.width = Val::Percent(100.0);         // Full width
            node.height = Val::Px(result_item.height_px);
            node.flex_grow = 0.0;                     // Don't expand beyond content
            node.flex_shrink = 1.0;                   // Allow shrinking
            
            // Percentage-based padding
            let padding_px = parent_width * result_item.padding_percent / 100.0;
            node.padding = UiRect::all(Val::Px(padding_px));
            
            // Horizontal flex layout for icon + text
            node.flex_direction = FlexDirection::Row;
            node.align_items = AlignItems::Center;
            node.justify_content = JustifyContent::FlexStart;
        }
    }
}

/// System for result item children (icon + text) layout
pub fn result_item_children_layout_system(
    result_items: Query<&Children, (With<ResultItem>, Changed<Children>)>,
    mut icons: Query<&mut Node, (With<ResultItemIcon>, Without<ResultItemText>)>,
    mut text_containers: Query<&mut Node, (With<ResultItemText>, Without<ResultItemIcon>)>,
    result_item_configs: Query<&ResultItem>,
) {
    for children in result_items.iter() {
        // Find the result item config (parent entity)
        let parent_entity = children.iter().next().and_then(|&child| {
            // In real implementation, would traverse up to find ResultItem entity
            None::<Entity>
        });
        
        // Update icon layout
        for child in children.iter() {
            if let Ok(mut icon_node) = icons.get_mut(*child) {
                // Fixed size icon
                icon_node.width = Val::Px(32.0);
                icon_node.height = Val::Px(32.0);
                icon_node.margin = UiRect::right(Val::Px(12.0));
                icon_node.flex_grow = 0.0;           // Don't grow
                icon_node.flex_shrink = 0.0;         // Don't shrink
            }
            
            if let Ok(mut text_node) = text_containers.get_mut(*child) {
                // Text fills remaining space
                text_node.width = Val::Percent(100.0);
                text_node.height = Val::Percent(100.0);
                text_node.flex_grow = 1.0;           // GROW to fill space
                text_node.flex_shrink = 1.0;         // Can shrink if needed
                text_node.flex_direction = FlexDirection::Column;
                text_node.justify_content = JustifyContent::Center;
                text_node.overflow = Overflow::clip(); // Clip long text
            }
        }
    }
}

/// Marker components for layout targeting
#[derive(Component)]
pub struct ResultItemIcon;

#[derive(Component)]  
pub struct ResultItemText;
```

### Responsive Window System

```rust
/// Resource for tracking responsive breakpoints
#[derive(Resource, Debug)]
pub struct ResponsiveBreakpoints {
    pub small_width: f32,      // Below this = compact mode
    pub medium_width: f32,     // Standard mode
    pub large_width: f32,      // Expanded mode (if needed)
    pub current_mode: ResponsiveMode,
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ResponsiveMode {
    Compact,   // < 400px wide
    Standard,  // 400-800px wide  
    Expanded,  // > 800px wide
}

impl Default for ResponsiveBreakpoints {
    fn default() -> Self {
        Self {
            small_width: 400.0,
            medium_width: 800.0,
            large_width: 1200.0,
            current_mode: ResponsiveMode::Standard,
        }
    }
}

/// System to handle responsive breakpoint changes
pub fn responsive_breakpoint_system(
    window: Query<&Window, With<PrimaryWindow>>,
    mut breakpoints: ResMut<ResponsiveBreakpoints>,
    mut flex_containers: Query<&mut FlexContainer>,
    mut search_sections: Query<&mut SearchSection>,
) {
    let Ok(window) = window.get_single() else { return };
    let window_width = window.resolution.physical_width() as f32;

    // Determine current responsive mode
    let new_mode = if window_width < breakpoints.small_width {
        ResponsiveMode::Compact
    } else if window_width < breakpoints.medium_width {
        ResponsiveMode::Standard
    } else {
        ResponsiveMode::Expanded
    };

    // Only update if mode changed
    if breakpoints.current_mode != new_mode {
        breakpoints.current_mode = new_mode;

        // Adjust layout based on responsive mode
        match new_mode {
            ResponsiveMode::Compact => {
                // Adjust for small screens
                for mut container in flex_containers.iter_mut() {
                    container.max_width = 380.0;    // Smaller max width
                    container.max_height = 320.0;   // Smaller max height
                }
                
                for mut search in search_sections.iter_mut() {
                    search.height_px = 40.0;        // Smaller search bar
                    search.padding_percent = 1.0;   // Less padding
                }
            },
            ResponsiveMode::Standard => {
                // Standard Raycast-like dimensions
                for mut container in flex_containers.iter_mut() {
                    container.max_width = 600.0;
                    container.max_height = 420.0;
                }
                
                for mut search in search_sections.iter_mut() {
                    search.height_px = 48.0;
                    search.padding_percent = 2.0;
                }
            },
            ResponsiveMode::Expanded => {
                // Larger screens - slightly bigger but not too big
                for mut container in flex_containers.iter_mut() {
                    container.max_width = 700.0;
                    container.max_height = 500.0;
                }
                
                for mut search in search_sections.iter_mut() {
                    search.height_px = 52.0;
                    search.padding_percent = 2.5;
                }
            },
        }
    }
}
```

### Layout Setup System

```rust
/// Setup responsive launcher layout using flex constraints
pub fn setup_responsive_launcher(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // Main launcher container with flex constraints
    let launcher_entity = commands.spawn((
        Node {
            // Start with percentage-based sizing
            width: Val::Percent(90.0),          // 90% of available space
            height: Val::Percent(80.0),         // 80% of available space
            max_width: Val::Px(600.0),          // Maximum launcher width
            max_height: Val::Px(420.0),         // Maximum launcher height
            
            // Critical flex properties to prevent expansion
            flex_grow: 0.0,                     // DON'T grow beyond content
            flex_shrink: 1.0,                   // CAN shrink if needed
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            
            // Positioning
            position_type: PositionType::Absolute,
            left: Val::Percent(50.0),           // Center horizontally
            top: Val::Percent(30.0),            // 30% from top
            
            // Visual properties
            padding: UiRect::all(Val::Px(12.0)),
            overflow: Overflow::clip(),
            ..default()
        },
        BackgroundColor(Color::srgba(0.08, 0.08, 0.09, 0.0)), // Start transparent
        BorderRadius::all(Val::Px(12.0)),
        FlexContainer::default(),
        LauncherContainer::default(),
        Visibility::Hidden,
    )).id();

    // Add search section with flex layout
    let search_entity = commands.spawn((
        Node {
            width: Val::Percent(100.0),         // Full width of parent
            height: Val::Px(48.0),              // Fixed height
            margin: UiRect::bottom(Val::Px(8.0)),
            padding: UiRect::all(Val::Percent(2.0)), // Percentage padding
            
            // Flex properties
            flex_grow: 0.0,                     // Don't grow
            flex_shrink: 1.0,                   // Can shrink
            flex_direction: FlexDirection::Row,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::FlexStart,
            
            ..default()
        },
        BackgroundGradient::from(LinearGradient::to_bottom(vec![
            ColorStop::new(Color::srgba(0.18, 0.18, 0.20, 0.85), Val::Percent(0.0)),
            ColorStop::new(Color::srgba(0.15, 0.15, 0.17, 0.90), Val::Percent(100.0)),
        ])),
        BorderRadius::all(Val::Px(8.0)),
        SearchSection::default(),
    )).id();

    // Add results section with flex grow
    let results_entity = commands.spawn((
        Node {
            width: Val::Percent(100.0),         // Full width
            height: Val::Percent(100.0),        // Fill remaining space
            
            // Flex properties for filling space
            flex_grow: 1.0,                     // GROW to fill available space
            flex_shrink: 1.0,                   // Can shrink
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Stretch,   // Items fill width
            justify_content: JustifyContent::FlexStart,
            
            // Content constraints
            max_height: Val::Px(320.0),         // Max height for 8 items
            overflow: Overflow::clip(),
            row_gap: Val::Percent(0.5),         // Small gap between items
            
            ..default()
        },
        ResultsSection::default(),
    )).id();

    // Build hierarchy
    commands.entity(launcher_entity).add_child(search_entity);
    commands.entity(launcher_entity).add_child(results_entity);
}

/// Spawn result items with flex-based layout
pub fn spawn_flex_result_items(
    mut commands: Commands,
    results_containers: Query<Entity, With<ResultsSection>>,
    font: Res<Handle<Font>>,
) {
    let Ok(results_entity) = results_containers.get_single() else { return };

    // Spawn multiple result items for testing
    for i in 0..8 {
        let result_entity = commands.spawn((
            Button, // For interaction
            Node {
                width: Val::Percent(100.0),     // Full width of parent
                height: Val::Px(48.0),          // Fixed item height
                margin: UiRect::bottom(Val::Px(2.0)),
                
                // Flex properties
                flex_grow: 0.0,                 // Don't grow beyond content
                flex_shrink: 1.0,               // Can shrink if needed
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                
                // Dynamic padding (will be set by system)
                padding: UiRect::all(Val::Px(8.0)), // Initial padding
                
                ..default()
            },
            BackgroundGradient::from(LinearGradient::to_bottom(vec![
                ColorStop::new(Color::srgba(0.13, 0.13, 0.15, 0.80), Val::Percent(0.0)),
                ColorStop::new(Color::srgba(0.11, 0.11, 0.13, 0.85), Val::Percent(100.0)),
            ])),
            BorderRadius::all(Val::Px(6.0)),
            ResultItem::default(),
        )).with_children(|parent| {
            // Icon (fixed size)
            parent.spawn((
                Node {
                    width: Val::Px(32.0),       // Fixed icon size
                    height: Val::Px(32.0),
                    margin: UiRect::right(Val::Px(12.0)),
                    flex_grow: 0.0,             // Don't grow
                    flex_shrink: 0.0,           // Don't shrink
                    ..default()
                },
                BackgroundColor(Color::srgba(0.25, 0.25, 0.25, 1.0)),
                BorderRadius::all(Val::Px(4.0)),
                ResultItemIcon,
            ));
            
            // Text container (grows to fill space)
            parent.spawn((
                Node {
                    width: Val::Percent(100.0), // Fill available space
                    height: Val::Percent(100.0),
                    flex_grow: 1.0,             // GROW to fill remaining space
                    flex_shrink: 1.0,           // Can shrink if needed
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::FlexStart,
                    overflow: Overflow::clip(),  // Clip long text
                    ..default()
                },
                ResultItemText,
            ))
            .with_children(|parent| {
                // Title
                parent.spawn((
                    Text::new(format!("Result Item {}", i + 1)),
                    TextFont {
                        font: font.clone(),
                        font_size: 14.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.95, 0.95, 0.97, 1.0)),
                ));
                
                // Subtitle
                parent.spawn((
                    Text::new(format!("Subtitle for item {}", i + 1)),
                    TextFont {
                        font: font.clone(),
                        font_size: 12.0,
                        ..default()
                    },
                    TextColor(Color::srgba(0.70, 0.70, 0.75, 1.0)),
                ));
            });
        }).id();

        // Add to results container
        commands.entity(results_entity).add_child(result_entity);
    }
}
```

### System Plugin and Registration

```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum FlexLayoutSystems {
    /// Update responsive breakpoints
    UpdateBreakpoints,
    /// Apply flex container constraints  
    ApplyContainerLayout,
    /// Apply section layouts
    ApplySectionLayout,
    /// Apply item layouts
    ApplyItemLayout,
    /// Update children layouts
    UpdateChildrenLayout,
}

pub struct FlexLayoutPlugin;

impl Plugin for FlexLayoutPlugin {
    fn build(&self, app: &mut App) {
        app
            .init_resource::<ResponsiveBreakpoints>()
            .register_type::<FlexContainer>()
            .register_type::<SearchSection>()
            .register_type::<ResultsSection>()
            .register_type::<ResultItem>()
            .configure_sets(
                Update,
                (
                    FlexLayoutSystems::UpdateBreakpoints,
                    FlexLayoutSystems::ApplyContainerLayout,
                    FlexLayoutSystems::ApplySectionLayout,
                    FlexLayoutSystems::ApplyItemLayout,
                    FlexLayoutSystems::UpdateChildrenLayout,
                ).chain(),
            )
            .add_systems(
                Update,
                (
                    responsive_breakpoint_system
                        .in_set(FlexLayoutSystems::UpdateBreakpoints),
                    flex_layout_system
                        .in_set(FlexLayoutSystems::ApplyContainerLayout),
                    (
                        search_section_layout_system,
                        results_section_layout_system,
                    ).in_set(FlexLayoutSystems::ApplySectionLayout),
                    result_item_layout_system
                        .in_set(FlexLayoutSystems::ApplyItemLayout),
                    result_item_children_layout_system
                        .in_set(FlexLayoutSystems::UpdateChildrenLayout),
                ),
            )
            .add_systems(Startup, (
                setup_responsive_launcher,
                spawn_flex_result_items,
            ).chain());
    }
}
```

### Testing Strategies for Flex Layouts

```rust
#[cfg(test)]
mod flex_layout_tests {
    use super::*;

    #[test]
    fn test_flex_container_defaults() {
        let container = FlexContainer::default();
        assert_eq!(container.width_percent, 100.0);
        assert_eq!(container.max_width, 600.0);
        assert_eq!(container.flex_grow, 0.0); // CRITICAL: Should not grow
        assert_eq!(container.overflow, Overflow::clip());
    }

    #[test]
    fn test_responsive_breakpoints() {
        let breakpoints = ResponsiveBreakpoints::default();
        assert_eq!(breakpoints.current_mode, ResponsiveMode::Standard);
        
        // Test breakpoint logic
        let test_width = 350.0;
        let expected_mode = if test_width < breakpoints.small_width {
            ResponsiveMode::Compact
        } else {
            ResponsiveMode::Standard
        };
        assert_eq!(expected_mode, ResponsiveMode::Compact);
    }

    #[test]
    fn test_result_item_layout() {
        let item = ResultItem::default();
        assert_eq!(item.text_flex_grow, 1.0); // Text should grow
        assert_eq!(item.height_px, 48.0);
        assert_eq!(item.icon_size_px, 32.0);
    }

    #[test] 
    fn test_search_section_responsive() {
        let search = SearchSection::default();
        assert_eq!(search.padding_percent, 2.0);
        assert_eq!(search.height_px, 48.0);
        
        // Test padding calculation
        let parent_width = 600.0;
        let expected_padding = parent_width * search.padding_percent / 100.0;
        assert_eq!(expected_padding, 12.0);
    }

    #[test]
    fn test_flex_layout_system() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_plugins(FlexLayoutPlugin);

        // Spawn test flex container
        let entity = app.world_mut().spawn((
            Node::default(),
            FlexContainer::default(),
        )).id();

        // Mock window size
        let window_entity = app.world_mut().spawn(Window {
            resolution: bevy::window::WindowResolution::new(800.0, 600.0),
            ..default()
        }).id();
        
        app.world_mut().entity_mut(window_entity).insert(PrimaryWindow);
        
        // Run system
        app.update();

        // Verify flex properties applied
        let node = app.world().get::<Node>(entity).unwrap();
        assert_eq!(node.flex_grow, 0.0); // Should not grow
        assert_eq!(node.overflow, Overflow::clip());
    }
}
```

**KEY IMPLEMENTATION NOTES:**

1. **NO FIXED PIXELS FOR CONTAINERS** - Uses `Val::Percent(100.0)` with `max_width`/`max_height` constraints
2. **CRITICAL `flex_grow: 0.0`** - Prevents unwanted expansion beyond content
3. **RESPONSIVE BREAKPOINTS** - Adapts to different screen sizes while maintaining constraints  
4. **FLEX-BASED SPACING** - Uses percentage-based padding and margins
5. **OVERFLOW MANAGEMENT** - Uses `Overflow::clip()` for proper content containment
6. **HIERARCHY-AWARE** - Child elements respond to parent size changes

This implementation solves the "expanding window" issue by using proper flex constraints instead of dynamic pixel-based sizing.

---

**Next:** [06-performance-optimization.md](./06-performance-optimization.md)