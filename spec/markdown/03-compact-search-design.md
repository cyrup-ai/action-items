# Compact Search Bar Design

## Critical Analysis: Expanding Search Input Problem

### Current Issue (ui/src/ui/systems/setup.rs:99-139)
```rust
// Search input container
.spawn((
    Node {
        width: Val::Percent(100.0),     // ❌ PROBLEM: Full width causes expansion
        height: Val::Px(56.0),          // ❌ Too tall - Raycast uses ~40px
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: theme.spacing_rect_hv(SpacingScale::LG, SpacingScale::MD), // ❌ 16px+12px padding
        ..default()
    },
```

**Root Cause:** The current search input is designed as a large container (56px height + padding) that takes full width. When text is entered, Bevy's text layout system causes the container to expand, creating the "big ass text area" behavior instead of Raycast's compact fixed search bar.

## Target Architecture: Raycast-like Compact Search

### Design Principles
1. **Viewport-Relative Height**: 5% viewport height (responsive, no expansion)
2. **Compact Padding**: Viewport-relative spacing for responsiveness
3. **Icon Integration**: Left-aligned search icon with viewport-relative spacing
4. **Text Constraints**: Single-line input with overflow handling
5. **Focus States**: Subtle border gradients without size changes

### Raycast Search Bar Analysis

#### Dimensions (Viewport-Relative)
- **Total Height**: 5% viewport height (responsive to screen size)
- **Internal Padding**: 1% of min viewport dimension
- **Icon Size**: Responsive to viewport (0.8% VMin margin)
- **Text Area**: Remaining width after icon and padding
- **Border Radius**: 0.6% of min viewport dimension

#### Visual States
- **Default**: Search input gradient background
- **Focus**: Blue border gradient appears  
- **Typing**: Text appears inline, no expansion
- **Placeholder**: "Search..." in secondary text color

## Implementation Specification

### Phase 1: Container Restructure

**File:** `ui/src/ui/systems/setup.rs`
**Lines:** 98-139 (Search input container)

**Current Structure:**
```rust
height: Val::Px(56.0),
padding: theme.spacing_rect_hv(SpacingScale::LG, SpacingScale::MD), // 16px + 12px
```

**Target Structure (Viewport-Relative):**
```rust
height: Val::Vh(5.0),                     // 5% of viewport height (responsive)
padding: UiRect::all(Val::VMin(1.0)),    // 1% of min viewport dimension
```

### Phase 2: Icon Positioning System

**Current Icon Implementation (Lines 116-127):**
```rust
// Search icon
let (search_icon_text, search_icon_font, search_icon_color) = 
    TextBundleBuilder::search_icon(&typography_clone);
parent.spawn((
    search_icon_text,
    search_icon_font, 
    search_icon_color,
    Node {
        margin: UiRect::right(theme.spacing_px(SpacingScale::MD)),
        ..default()
    },
));
```

**Target Icon Implementation (Viewport-Relative):**
```rust
/// Compact search icon with viewport-relative positioning
parent.spawn((
    Text::new("\u{F002}"),                // FontAwesome search icon
    TextFont {
        font: fonts.icons.clone(),
        font_size: 16.0,                   // Base size (should scale with viewport in system)
        ..default()
    },
    TextColor(theme.colors.text_secondary),
    Node {
        margin: UiRect::right(Val::VMin(0.8)), // 0.8% of min viewport dimension
        flex_shrink: 0.0,                  // Prevent icon compression
        ..default()
    },
    SearchIcon, // New component for identification
));
```

### Phase 3: Text Input Constraints

**New Component:** `CompactTextInput`
```rust
#[derive(Component, Debug, Clone)]
pub struct CompactTextInput {
    placeholder: String,
    current_text: String,
    max_visible_width_vw: f32,    // Max width as % of viewport width
    cursor_position: usize,
    is_focused: bool,
}

impl Default for CompactTextInput {
    fn default() -> Self {
        Self {
            placeholder: "Search...".to_string(),
            current_text: String::new(),
            max_visible_width_vw: 55.0,   // ~55% of viewport width (container - padding)
            cursor_position: 0,
            is_focused: false,
        }
    }
}
```

**Current Text Implementation (Lines 130-138):**
```rust
// Search input text
let (input_text, input_font, input_color, input_shadow) = 
    TextBundleBuilder::search_input("Type to search...", &typography_clone);
parent.spawn((
    input_text,
    input_font,
    input_color,
    input_shadow,
    SearchInput,
));
```

**Target Text Implementation:**
```rust
/// Compact search input with overflow constraints
parent.spawn((
    Text::new("Search..."),               // Placeholder text
    TextFont {
        font: fonts.regular.clone(),
        font_size: 16.0,                   // Readable but compact
        ..default()
    },
    TextColor(theme.colors.text_secondary), // Placeholder color
    Node {
        flex_grow: 1.0,                    // Take remaining space
        overflow: Overflow::clip(),        // Clip overflowing text
        align_items: AlignItems::Center,
        ..default()
    },
    CompactTextInput::default(),
    SearchInput,
));
```

### Phase 4: Focus State Management

**New System:** `search_focus_system`
```rust
/// Handle search input focus states without expanding container
#[inline]
pub fn search_focus_system(
    mut query: Query<
        (&mut BorderGradient, &mut CompactTextInput, &Interaction),
        (With<SearchInput>, Changed<Interaction>),
    >,
    mut app_state: ResMut<NextState<AppState>>,
    theme: Res<Theme>,
) {
    for (mut border_gradient, mut input, interaction) in query.iter_mut() {
        match *interaction {
            Interaction::Pressed => {
                // Enter search mode
                input.is_focused = true;
                app_state.set(AppState::SearchMode);
                
                // Apply focus border gradient
                *border_gradient = theme.colors.search_focus_border_gradient();
            },
            Interaction::None => {
                if input.is_focused && input.current_text.is_empty() {
                    // Exit search mode if no text
                    input.is_focused = false;
                    app_state.set(AppState::Background);
                    
                    // Remove focus border
                    *border_gradient = BorderGradient::default();
                }
            },
            _ => {}
        }
    }
}
```

### Phase 5: Text Display System

**New System:** `compact_text_display_system`
```rust
/// Update displayed text in compact search input
#[inline]
pub fn compact_text_display_system(
    mut query: Query<(&mut Text, &CompactTextInput), Changed<CompactTextInput>>,
    theme: Res<Theme>,
) {
    for (mut text_component, input) in query.iter_mut() {
        // Determine what to display
        let display_text = if input.current_text.is_empty() {
            &input.placeholder
        } else {
            &input.current_text
        };
        
        // Handle text overflow with intelligent truncation
        let truncated_text = if display_text.len() > input.max_visible_chars {
            format!("{}...", &display_text[..input.max_visible_chars - 3])
        } else {
            display_text.to_string()
        };
        
        // Update text content
        text_component.0 = truncated_text;
    }
}
```

### Phase 6: Input Processing Integration

**Text Input Handler:**
```rust
/// Process keyboard input for compact search bar
#[inline]
pub fn compact_search_input_system(
    mut char_events: EventReader<ReceivedCharacter>,
    mut key_events: EventReader<KeyboardInput>,
    mut query: Query<&mut CompactTextInput, With<SearchInput>>,
    app_state: Res<State<AppState>>,
) {
    // Only process input when in search mode
    if *app_state.get() != AppState::SearchMode {
        return;
    }
    
    let Ok(mut input) = query.get_single_mut() else { return };
    
    // Handle character input
    for event in char_events.read() {
        let character = event.char;
        
        // Filter printable characters only
        if character.is_control() { continue; }
        
        // Insert character at cursor position
        input.current_text.insert(input.cursor_position, character);
        input.cursor_position += character.len_utf8();
    }
    
    // Handle special keys
    for event in key_events.read() {
        if !event.state.is_pressed() { continue; }
        
        match event.key_code {
            KeyCode::Backspace => {
                if input.cursor_position > 0 {
                    let prev_char_start = input.current_text
                        .char_indices()
                        .nth(input.cursor_position.saturating_sub(1))
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    
                    input.current_text.remove(prev_char_start);
                    input.cursor_position = prev_char_start;
                }
            },
            KeyCode::Escape => {
                // Clear search and exit search mode
                input.current_text.clear();
                input.cursor_position = 0;
                input.is_focused = false;
            },
            _ => {}
        }
    }
}
```

### Phase 7: Search Bar Styling

**Border and Shadow Configuration:**
```rust
/// Apply compact search bar styling
.spawn((
    Node {
        width: Val::Percent(100.0),
        height: Val::Px(40.0),              // Compact height
        flex_direction: FlexDirection::Row,
        align_items: AlignItems::Center,
        padding: UiRect {
            left: Val::Px(12.0),
            right: Val::Px(12.0), 
            top: Val::Px(10.0),
            bottom: Val::Px(10.0),
        },
        ..default()
    },
    theme.colors.search_input_gradient(),   // Background gradient
    BorderRadius::all(Val::Px(8.0)),        // Compact border radius
    BorderColor(Color::TRANSPARENT),         // Transparent by default
    Outline::new(
        Val::Px(1.0), 
        Val::ZERO, 
        Color::TRANSPARENT              // Focus will add border gradient
    ),
    theme.create_box_shadow(ShadowElevation::SM), // Subtle depth
    SearchInputContainer,
))
```

## Implementation Timeline

### Phase 1: Container Height Fix (High Priority)
- Change height from 56px to 40px
- Adjust padding to compact spacing
- Test visual alignment

### Phase 2: Icon Integration (High Priority)  
- Fix icon positioning and sizing
- Add SearchIcon component
- Ensure icon doesn't compress

### Phase 3: Text Constraints (High Priority)
- Add CompactTextInput component  
- Implement text overflow handling
- Test with long search queries

### Phase 4: Focus States (Medium Priority)
- Implement search_focus_system
- Add border gradient transitions
- Test focus/unfocus behavior

### Phase 5: Input Processing (Medium Priority)
- Add compact_search_input_system
- Handle character input properly
- Test keyboard interaction

### Phase 6: Display System (Low Priority)  
- Implement compact_text_display_system
- Add placeholder text handling
- Polish visual feedback

## Performance Requirements

### Zero Allocation Constraints
- Reuse existing strings where possible
- Use `String::insert` and `String::remove` for text editing
- Avoid unnecessary text recreations
- Cache font handles and computed layouts

### Benchmarking Targets
- Text input processing: < 0.1ms per keystroke
- Focus state transitions: < 0.05ms
- Text display updates: < 0.02ms per frame

## Testing Strategy

### Visual Tests
- Height comparison with Raycast (40px vs 56px+padding)
- Icon alignment and spacing verification
- Text overflow behavior testing
- Focus state border appearance

### Interaction Tests  
- Click to focus behavior
- Keyboard input responsiveness
- Text editing accuracy
- Escape key functionality

### Performance Tests
- Keystroke latency measurement
- Memory allocation tracking
- Long text input handling

## Dependencies

### Required Components
- CompactTextInput - Text state management
- SearchIcon - Icon identification
- SearchInputContainer - Container identification

### System Dependencies  
- search_focus_system - Focus state management
- compact_text_display_system - Text rendering
- compact_search_input_system - Keyboard input

## Success Criteria

1. ✅ Search bar height exactly 40px (no expansion)
2. ✅ Icon properly positioned and sized (16px)
3. ✅ Text input works inline without expanding container
4. ✅ Focus states show border gradient without size changes
5. ✅ Placeholder text displays correctly
6. ✅ Long text truncated with ellipsis
7. ✅ Performance targets met for all interactions

## Bevy Implementation Details

### ECS Architecture for Search Input

**Component Bundles:**
```rust
#[derive(Bundle)]
pub struct CompactSearchBarBundle {
    // Node components
    node: Node,
    style: BackgroundColor,
    border: BorderColor,
    border_radius: BorderRadius,
    
    // Search-specific components
    input: CompactTextInput,
    interaction: Interaction,
    focus_policy: FocusPolicy,
    
    // Visual components
    outline: Outline,
    box_shadow: BoxShadow,
    
    // Marker component
    marker: SearchInputContainer,
}

impl CompactSearchBarBundle {
    pub fn new(theme: &Theme) -> Self {
        Self {
            node: Node {
                width: Val::Percent(100.0),
                height: Val::Px(40.0),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                padding: UiRect::all(Val::Px(12.0)),
                ..default()
            },
            style: theme.colors.search_input_gradient(),
            border: BorderColor(Color::TRANSPARENT),
            border_radius: BorderRadius::all(Val::Px(8.0)),
            input: CompactTextInput::default(),
            interaction: Interaction::None,
            focus_policy: FocusPolicy::Block,
            outline: Outline::new(Val::Px(1.0), Val::ZERO, Color::TRANSPARENT),
            box_shadow: theme.create_box_shadow(ShadowElevation::SM),
            marker: SearchInputContainer,
        }
    }
}
```

**Event System Integration:**
```rust
#[derive(Event)]
pub enum SearchEvent {
    TextChanged { query: String },
    FocusGained,
    FocusLost,
    Submit { query: String },
    Clear,
}

fn search_event_handler(
    mut events: EventReader<SearchEvent>,
    mut search_inputs: Query<&mut CompactTextInput>,
    mut search_system: EventWriter<SearchSystemEvent>,
) {
    for event in events.read() {
        match event {
            SearchEvent::TextChanged { query } => {
                // Trigger search system
                search_system.send(SearchSystemEvent::Query(query.clone()));
            }
            SearchEvent::Submit { query } => {
                // Execute selected result
                search_system.send(SearchSystemEvent::Execute(query.clone()));
            }
            _ => {}
        }
    }
}
```

### Bevy Input Handling

**Using Bevy's Input System:**
```rust
fn bevy_text_input_system(
    mut events: EventReader<TextInputEvent>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut search_inputs: Query<&mut CompactTextInput, With<SearchInputContainer>>,
    mut search_events: EventWriter<SearchEvent>,
) {
    let Ok(mut input) = search_inputs.get_single_mut() else { return };
    
    // Handle text input events (Bevy 0.14+)
    for event in events.read() {
        match event {
            TextInputEvent::Char { character, .. } => {
                input.current_text.push(*character);
                input.cursor_position += 1;
                
                // Emit search event
                search_events.send(SearchEvent::TextChanged {
                    query: input.current_text.clone(),
                });
            }
            TextInputEvent::Backspace => {
                if !input.current_text.is_empty() {
                    input.current_text.pop();
                    input.cursor_position = input.cursor_position.saturating_sub(1);
                    
                    search_events.send(SearchEvent::TextChanged {
                        query: input.current_text.clone(),
                    });
                }
            }
        }
    }
    
    // Handle keyboard shortcuts
    if keyboard.just_pressed(KeyCode::Escape) {
        input.current_text.clear();
        input.cursor_position = 0;
        search_events.send(SearchEvent::Clear);
    }
    
    if keyboard.just_pressed(KeyCode::Enter) {
        search_events.send(SearchEvent::Submit {
            query: input.current_text.clone(),
        });
    }
}
```

### Animation and Transitions

**Smooth Focus Transitions:**
```rust
#[derive(Component)]
pub struct SearchBarAnimation {
    border_opacity: f32,
    target_opacity: f32,
    animation_speed: f32,
}

fn animate_search_focus(
    time: Res<Time>,
    mut query: Query<
        (&mut BorderColor, &mut SearchBarAnimation),
        With<SearchInputContainer>
    >,
) {
    for (mut border_color, mut animation) in query.iter_mut() {
        // Lerp border opacity for smooth transitions
        animation.border_opacity = animation.border_opacity.lerp(
            animation.target_opacity,
            animation.animation_speed * time.delta_secs()
        );
        
        // Apply animated border color
        let gradient_color = Color::srgba(0.3, 0.5, 1.0, animation.border_opacity);
        *border_color = BorderColor(gradient_color);
    }
}
```

### Query Optimization

**Efficient Text Rendering:**
```rust
// Use Changed<T> filter for performance
fn update_search_text_display(
    mut query: Query<
        (&mut Text, &CompactTextInput),
        Changed<CompactTextInput>
    >,
) {
    for (mut text, input) in query.iter_mut() {
        // Only update when input changes
        text.0 = if input.current_text.is_empty() {
            input.placeholder.clone()
        } else {
            truncate_text(&input.current_text, input.max_visible_chars)
        };
    }
}

#[inline]
fn truncate_text(text: &str, max_chars: usize) -> String {
    if text.len() <= max_chars {
        text.to_string()
    } else {
        format!("{}...", &text[..max_chars.saturating_sub(3)])
    }
}
```

### State Management

**Search State Resource:**
```rust
#[derive(Resource, Default)]
pub struct SearchState {
    pub is_active: bool,
    pub current_query: String,
    pub selected_index: Option<usize>,
    pub results_count: usize,
}

// State-driven system
fn search_state_system(
    search_state: Res<SearchState>,
    mut visibility: Query<&mut Visibility, With<SearchResultsContainer>>,
) {
    if search_state.is_active && search_state.results_count > 0 {
        for mut vis in visibility.iter_mut() {
            *vis = Visibility::Visible;
        }
    } else {
        for mut vis in visibility.iter_mut() {
            *vis = Visibility::Hidden;
        }
    }
}
```

### Testing Strategy with Bevy

**Unit Test Example:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bevy::app::{App, Update};
    
    #[test]
    fn test_search_bar_height() {
        let mut app = App::new();
        app.add_plugins(MinimalPlugins);
        
        let search_bar = app.world_mut().spawn(
            CompactSearchBarBundle::new(&Theme::default())
        ).id();
        
        let node = app.world().get::<Node>(search_bar).unwrap();
        assert_eq!(node.height, Val::Px(40.0));
    }
    
    #[test]
    fn test_text_truncation() {
        let long_text = "a".repeat(100);
        let truncated = truncate_text(&long_text, 50);
        assert_eq!(truncated.len(), 50); // 47 chars + "..."
        assert!(truncated.ends_with("..."));
    }
}
```

---

**Next:** [04-realtime-input-system.md](./04-realtime-input-system.md)