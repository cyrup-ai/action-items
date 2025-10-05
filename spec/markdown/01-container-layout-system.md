# Container Layout System Redesign

## Critical Analysis: Expanding Layout Problem

### Current Issue (ui/src/ui/systems/setup.rs:82-95)
```rust
.spawn((
    Node {
        width: Val::Percent(80.0),        // ❌ PROBLEM: Expanding width
        max_width: Val::Px(800.0),        // ❌ Too wide for compact launcher
        min_width: Val::Px(400.0),        // ❌ Causes expansion behavior
        flex_direction: FlexDirection::Column,
        margin: UiRect::top(Val::Percent(15.0)),
        padding: theme.spacing_rect(SpacingScale::XL),
        row_gap: theme.spacing_px(SpacingScale::MD),
        ..default()
    },
```

**Root Cause:** Percentage-based widths with min/max constraints create expanding behavior when content changes. This is fundamentally different from Raycast's fixed-width compact design.

## Target Architecture: Fixed Compact Container

### Design Principles
1. **Viewport-Relative Width**: 60% viewport width (responsive to screen size)
2. **Responsive Constraints**: Intelligent scaling using viewport units
3. **Compact Spacing**: Tight margins using viewport-relative units
4. **Zero Expansion**: Content never causes container to grow

### Implementation Specification

#### Phase 1: Core Container Replacement

**File:** `ui/src/ui/systems/setup.rs`
**Lines:** 79-96 (Main container spawn)

**Current Code:**
```rust
width: Val::Percent(80.0),
max_width: Val::Px(800.0),
min_width: Val::Px(400.0),
```

**Target Code (CORRECTED - Viewport-Relative Pattern):**
```rust
width: Val::Vw(60.0),                     // 60% of viewport width (responsive)
max_width: Val::Vw(60.0),                 // Constrain to viewport percentage
min_width: Val::Vw(35.0),                 // Minimum 35% of viewport width
flex_grow: 0.0,                           // Prevent expansion beyond max
flex_shrink: 1.0,                         // Allow shrinking to min
align_self: AlignSelf::Center,            // Center in parent
```

#### Phase 2: Responsive Container System

**New Component:** `CompactContainer`
```rust
#[derive(Component, Debug, Clone)]
pub struct CompactContainer {
    base_width_vw: f32,        // 60.0 (60% viewport width)
    min_width_vw: f32,         // 35.0 (35% viewport width)  
    portrait_width_vw: f32,    // 90.0 (90% for portrait mode)
}

impl Default for CompactContainer {
    fn default() -> Self {
        Self {
            base_width_vw: 60.0,
            min_width_vw: 35.0,
            portrait_width_vw: 90.0,
        }
    }
}
```

**New System:** `update_compact_container_system`
```rust
/// Zero-allocation system for responsive container sizing
#[inline]
pub fn update_compact_container_system(
    windows: Query<&Window, With<PrimaryWindow>>,
    mut containers: Query<(&mut Node, &CompactContainer), Changed<CompactContainer>>,
) {
    let Ok(window) = windows.get_single() else { return };
    let viewport_width = window.width();
    let viewport_height = window.height();
    
    for (mut node, container) in containers.iter_mut() {
        node.width = if viewport_width < viewport_height {
            // Portrait mode: use more width
            Val::Vw(container.portrait_width_vw)
        } else {
            // Landscape mode: standard viewport-relative width
            Val::Vw(container.base_width_vw)
        };
    }
}
```

#### Phase 3: Compact Spacing System

**Current Spacing Issues:**
- `margin: UiRect::top(Val::Percent(15.0))` - Percentage margins create inconsistent spacing
- `padding: theme.spacing_rect(SpacingScale::XL)` - XL padding too large for compact design

**Target Spacing (Viewport-Relative):**
```rust
// Compact top margin: 12% of viewport height
margin: UiRect {
    top: Val::Vh(12.0),
    left: Val::Auto,
    right: Val::Auto, 
    bottom: Val::Px(0.0),
},
// Compact padding: 1% of min viewport dimension
padding: UiRect::all(Val::VMin(1.0)),
// Tighter row gap: 0.6% of min viewport dimension
row_gap: Val::VMin(0.6),
```

#### Phase 4: Overflow and Content Constraints

**New Component:** `ContentConstraints`
```rust
#[derive(Component, Debug, Clone)]
pub struct ContentConstraints {
    max_visible_results: usize,  // 8 (like Raycast)
    result_height_vh: f32,       // 6.0 (6% of viewport height per result)
    max_content_height_vh: f32,  // 8 * 6.0 = 48.0 (48% of viewport height)
}
```

**Content Height Calculation:**
```rust
/// Calculate maximum container height to prevent expansion
/// Formula: search_bar_height + (max_results * result_height) + padding
#[inline]
fn calculate_max_container_height_vh(constraints: &ContentConstraints) -> f32 {
    const SEARCH_BAR_HEIGHT_VH: f32 = 5.0;  // 5% viewport height
    const PADDING_VH: f32 = 2.0;            // 2% viewport height for padding
    
    SEARCH_BAR_HEIGHT_VH + constraints.max_content_height_vh + PADDING_VH
}
```

#### Phase 5: Text Truncation System

**Problem:** Long text in results causes horizontal expansion

**Solution:** Intelligent text truncation with ellipsis
```rust
#[derive(Component, Debug)]
pub struct TextTruncation {
    max_width_vw: f32,   // Available width as % of viewport width
    ellipsis: String,    // "..."
}

/// System to truncate text that would cause container expansion
#[inline]
pub fn text_truncation_system(
    mut query: Query<(&mut Text, &TextTruncation), Changed<Text>>,
    fonts: Res<TypographyScale>,
) {
    for (mut text, truncation) in query.iter_mut() {
        let content = &text.0;
        
        // Fast width estimation: avg char width * char count
        const AVG_CHAR_WIDTH: f32 = 8.0; // Approximation for Ubuntu font
        let estimated_width = content.len() as f32 * AVG_CHAR_WIDTH;
        
        if estimated_width > truncation.max_width {
            let max_chars = (truncation.max_width / AVG_CHAR_WIDTH) as usize;
            let truncated = if max_chars > 3 {
                format!("{}...", &content[..max_chars - 3])
            } else {
                "...".to_string()
            };
            text.0 = truncated;
        }
    }
}
```

## Implementation Timeline

### Phase 1: Core Container (High Priority)
- Replace percentage-based widths in setup.rs
- Add CompactContainer component
- Test fixed width behavior

### Phase 2: Responsive System (Medium Priority)  
- Implement update_compact_container_system
- Add window resize handling
- Test on various screen sizes

### Phase 3: Spacing Optimization (Medium Priority)
- Reduce padding from XL to MD
- Optimize margins for compact design
- Adjust row gaps for tighter layout

### Phase 4: Content Constraints (High Priority)
- Add ContentConstraints component
- Implement max height calculations
- Prevent vertical expansion

### Phase 5: Text Truncation (Low Priority)
- Implement TextTruncation component
- Add text_truncation_system
- Test with long application names

## Performance Requirements

### Zero Allocation Constraints
- All systems must use `#[inline]` for hot paths
- No heap allocations in update loops
- Reuse existing components where possible
- Use const values for fixed dimensions

### Benchmarking Targets
- Container layout: < 0.1ms per frame
- Text truncation: < 0.05ms per update
- Responsive updates: < 0.2ms per window resize

## Testing Strategy

### Unit Tests
- Container width calculations
- Responsive scaling logic
- Text truncation accuracy

### Integration Tests  
- Full layout with various content sizes
- Window resize behavior
- Performance under load

### Visual Tests
- Compare with Raycast screenshots
- Verify no expansion behavior
- Test on multiple screen sizes

## Dependencies

### Required Crates
- bevy (existing) - Core ECS system
- No additional crates required

### Internal Dependencies
- theme.rs - Spacing system
- typography.rs - Font handling
- components.rs - UI component definitions

## Bevy Implementation Details

### ECS Architecture Integration

**System Sets and Ordering:**
```rust
#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum UiLayoutSystems {
    ContainerSetup,        // Initial container spawning
    ResponsiveUpdate,      // Window-reactive updates
    ContentConstraints,    // Content size management
    TextProcessing,       // Text truncation
}

// In App::build()
app.configure_sets(
    Update,
    (
        UiLayoutSystems::ContainerSetup,
        UiLayoutSystems::ResponsiveUpdate,
        UiLayoutSystems::ContentConstraints,
        UiLayoutSystems::TextProcessing,
    ).chain(),
)
.add_systems(
    Update,
    (
        update_compact_container_system.in_set(UiLayoutSystems::ResponsiveUpdate),
        text_truncation_system.in_set(UiLayoutSystems::TextProcessing),
    ),
);
```

**Resource Management:**
```rust
#[derive(Resource)]
pub struct LayoutConfig {
    pub base_width: f32,
    pub responsive_threshold: f32,
    pub max_visible_results: usize,
}

impl Default for LayoutConfig {
    fn default() -> Self {
        Self {
            base_width: 600.0,
            responsive_threshold: 700.0,
            max_visible_results: 8,
        }
    }
}

// Initialize in plugin
app.init_resource::<LayoutConfig>();
```

**Event-Driven Updates:**
```rust
#[derive(Event)]
pub enum LayoutEvent {
    WindowResized { width: f32, height: f32 },
    ContentOverflow { excess_items: usize },
    ResponsiveModeChanged { is_responsive: bool },
}

fn handle_layout_events(
    mut events: EventReader<LayoutEvent>,
    mut containers: Query<&mut Node, With<CompactContainer>>,
) {
    for event in events.read() {
        match event {
            LayoutEvent::WindowResized { width, .. } => {
                // Trigger responsive updates
                for mut node in containers.iter_mut() {
                    node.set_changed();
                }
            }
            _ => {}
        }
    }
}
```

### Bevy UI Specific Patterns

**Using Style Sheets Pattern:**
```rust
// Define reusable styles as bundles
#[derive(Bundle, Clone)]
pub struct CompactContainerBundle {
    node: Node,
    style: BackgroundColor,
    border: BorderColor,
    container: CompactContainer,
    constraints: ContentConstraints,
    interaction: Interaction,
    focus_policy: FocusPolicy,
}

impl Default for CompactContainerBundle {
    fn default() -> Self {
        Self {
            node: Node {
                width: Val::Percent(100.0),
                max_width: Val::Px(600.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            style: BackgroundColor(Color::srgba(0.1, 0.1, 0.1, 0.98)),
            border: BorderColor(Color::srgba(0.3, 0.3, 0.3, 0.5)),
            container: CompactContainer::default(),
            constraints: ContentConstraints::default(),
            interaction: Interaction::None,
            focus_policy: FocusPolicy::Block,
        }
    }
}
```

**Query Filters for Performance:**
```rust
// Use Changed<T> filters to avoid unnecessary work
fn update_only_changed_containers(
    mut query: Query<
        (&mut Node, &CompactContainer),
        Or<(Changed<CompactContainer>, Changed<Window>)>
    >,
) {
    // Only runs when container or window changes
}
```

**Transform Hierarchy for Smooth Animations:**
```rust
// Separate transform from node for animations
commands.spawn((
    CompactContainerBundle::default(),
    Transform::from_xyz(0.0, 0.0, 0.0),
    GlobalTransform::default(),
    // Visibility components for show/hide animations
    Visibility::default(),
    ViewVisibility::default(),
));
```

### Performance Optimizations with Bevy

**Use Local<T> for System State:**
```rust
fn text_truncation_with_cache(
    mut local_cache: Local<HashMap<Entity, String>>,
    query: Query<(Entity, &Text, &TextTruncation), Changed<Text>>,
) {
    for (entity, text, truncation) in query.iter() {
        // Cache truncated text to avoid recomputation
        if let Some(cached) = local_cache.get(&entity) {
            continue;
        }
        // Truncation logic...
        local_cache.insert(entity, truncated_text);
    }
}
```

**Batch UI Updates:**
```rust
// Use CommandQueue for batched updates
fn batch_container_updates(
    mut commands: Commands,
    containers: Query<Entity, With<CompactContainer>>,
) {
    let mut command_queue = CommandQueue::default();
    
    for entity in containers.iter() {
        command_queue.push(move |world: &mut World| {
            // Batch updates here
        });
    }
    
    commands.append(&mut command_queue);
}
```

### Testing with Bevy

**Integration Test Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::App;
    
    #[test]
    fn test_compact_container_fixed_width() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins)
           .add_systems(Update, update_compact_container_system);
        
        let entity = app.world_mut().spawn(
            CompactContainerBundle::default()
        ).id();
        
        app.update();
        
        let node = app.world().get::<Node>(entity).unwrap();
        assert_eq!(node.width, Val::Percent(100.0));
        assert_eq!(node.max_width, Val::Px(600.0));
    }
}
```

## Success Criteria

1. ✅ Container maintains 600px width regardless of content
2. ✅ No horizontal expansion when typing or showing results  
3. ✅ Responsive behavior on screens < 700px width
4. ✅ Visual match with Raycast compact design
5. ✅ Zero allocations in steady state
6. ✅ Performance targets met

---

**Next:** [02-gradient-system.md](./02-gradient-system.md)